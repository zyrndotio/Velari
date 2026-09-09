use std::collections::HashMap;

use crate::{
    ast::{Expr, Op, Stmt, Type, Value},
    lexer::Span,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VelaRiType {
    Int,
    Number,
    Boolean,
    String,
    Null,
    Array(Box<VelaRiType>),
    Map(Box<VelaRiType>, Box<VelaRiType>),
    Unknown,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SemanticError {
    pub message: String,
    pub span: Span,
}

#[derive(Clone)]
struct FunctionSig {
    parameters: Vec<VelaRiType>,
    result: VelaRiType,
}

pub struct SemanticAnalyzer {
    scopes: Vec<HashMap<String, VelaRiType>>,
    functions: HashMap<String, FunctionSig>,
    in_function: bool,
    return_type: Option<VelaRiType>,
}

fn type_from_ast(ty: &Type) -> VelaRiType {
    match ty {
        Type::Int => VelaRiType::Int,
        Type::Float => VelaRiType::Number,
        Type::Bool => VelaRiType::Boolean,
        Type::String => VelaRiType::String,
        Type::Null => VelaRiType::Null,
        Type::Array(element) => VelaRiType::Array(Box::new(type_from_ast(element))),
        Type::Map(key, value) => {
            VelaRiType::Map(Box::new(type_from_ast(key)), Box::new(type_from_ast(value)))
        }
        Type::Unknown => VelaRiType::Unknown,
    }
}

fn type_name(ty: &VelaRiType) -> String {
    match ty {
        VelaRiType::Int => "Int".into(),
        VelaRiType::Number => "Float".into(),
        VelaRiType::Boolean => "Boolean".into(),
        VelaRiType::String => "String".into(),
        VelaRiType::Null => "Null".into(),
        VelaRiType::Array(element) => format!("Array<{}>", type_name(element)),
        VelaRiType::Map(key, value) => format!("Map<{}, {}>", type_name(key), type_name(value)),
        VelaRiType::Unknown => "Unknown".into(),
    }
}

fn compatible(expected: &VelaRiType, actual: &VelaRiType) -> bool {
    expected == &VelaRiType::Unknown
        || actual == &VelaRiType::Unknown
        || expected == actual
        || matches!((expected, actual), (VelaRiType::Number, VelaRiType::Int))
}

fn merge_types(left: &VelaRiType, right: &VelaRiType) -> VelaRiType {
    if left == &VelaRiType::Unknown {
        return right.clone();
    }
    if right == &VelaRiType::Unknown || left == right {
        return left.clone();
    }
    match (left, right) {
        (VelaRiType::Int, VelaRiType::Number) | (VelaRiType::Number, VelaRiType::Int) => {
            VelaRiType::Number
        }
        (VelaRiType::Array(left), VelaRiType::Array(right)) => {
            VelaRiType::Array(Box::new(merge_types(left, right)))
        }
        (VelaRiType::Map(left_key, left_value), VelaRiType::Map(right_key, right_value)) => {
            VelaRiType::Map(
                Box::new(merge_types(left_key, right_key)),
                Box::new(merge_types(left_value, right_value)),
            )
        }
        _ => VelaRiType::Unknown,
    }
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
            functions: HashMap::new(),
            in_function: false,
            return_type: None,
        }
    }

    pub fn analyze(&mut self, root: &Stmt) -> Result<(), Vec<SemanticError>> {
        let mut errors = Vec::new();
        self.collect_functions(root);
        self.stmt(root, &mut errors);
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    fn collect_functions(&mut self, statement: &Stmt) {
        match statement {
            Stmt::Block(statements) => {
                for statement in statements {
                    self.collect_functions(statement);
                }
            }
            Stmt::Function {
                name,
                parameters,
                return_type,
                ..
            } => {
                self.functions.insert(
                    name.clone(),
                    FunctionSig {
                        parameters: parameters
                            .iter()
                            .map(|(_, ty)| {
                                ty.as_ref()
                                    .map(type_from_ast)
                                    .unwrap_or(VelaRiType::Unknown)
                            })
                            .collect(),
                        result: return_type
                            .as_ref()
                            .map(type_from_ast)
                            .unwrap_or(VelaRiType::Unknown),
                    },
                );
            }
            _ => {}
        }
    }

    fn push(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn pop(&mut self) {
        self.scopes.pop();
    }

    fn define(&mut self, name: String, ty: VelaRiType) {
        self.scopes.last_mut().unwrap().insert(name, ty);
    }

    fn lookup(&self, name: &str) -> Option<VelaRiType> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(name).cloned())
    }

    fn report_mismatch(
        &self,
        expected: &VelaRiType,
        actual: &VelaRiType,
        span: Span,
        errors: &mut Vec<SemanticError>,
    ) {
        if !compatible(expected, actual) {
            errors.push(SemanticError {
                message: format!(
                    "type mismatch: expected {}, found {}",
                    type_name(expected),
                    type_name(actual)
                ),
                span,
            });
        }
    }

    fn stmt(&mut self, statement: &Stmt, errors: &mut Vec<SemanticError>) {
        match statement {
            Stmt::Block(statements) => {
                for statement in statements {
                    self.stmt(statement, errors);
                }
            }
            Stmt::Let {
                name,
                annotation,
                value,
                span,
            } => {
                if let Ok(actual) = self.expr(value, errors) {
                    if let Some(annotation) = annotation {
                        self.report_mismatch(&type_from_ast(annotation), &actual, *span, errors);
                    }
                    if self.scopes.last().unwrap().contains_key(name) {
                        errors.push(SemanticError {
                            message: format!("variable `{name}` is already declared in this scope"),
                            span: *span,
                        });
                    } else {
                        self.define(
                            name.clone(),
                            annotation.as_ref().map(type_from_ast).unwrap_or(actual),
                        );
                    }
                }
            }
            Stmt::Assign { name, value, span } => {
                let expected = self.lookup(name);
                if expected.is_none() {
                    errors.push(SemanticError {
                        message: format!("variable `{name}` used before initialization"),
                        span: *span,
                    });
                }
                if let Ok(actual) = self.expr(value, errors) {
                    if let Some(expected) = expected {
                        self.report_mismatch(&expected, &actual, *span, errors);
                    }
                }
            }
            Stmt::Print(expression) => {
                let _ = self.expr(expression, errors);
            }
            Stmt::Assert(expression) => {
                if self.expr(expression, errors) != Ok(VelaRiType::Boolean) {
                    errors.push(SemanticError {
                        message: "assert condition must be Boolean".into(),
                        span: expression.span(),
                    });
                }
            }
            Stmt::Return { value, span } => {
                if !self.in_function {
                    errors.push(SemanticError {
                        message: "return is only valid inside a function".into(),
                        span: *span,
                    });
                }
                let actual = value
                    .as_ref()
                    .map(|expression| self.expr(expression, errors).unwrap_or(VelaRiType::Unknown))
                    .unwrap_or(VelaRiType::Null);
                if let Some(expected) = &self.return_type {
                    self.report_mismatch(expected, &actual, *span, errors);
                }
            }
            Stmt::Function {
                name,
                parameters,
                body,
                return_type,
                span,
            } => {
                self.push();
                for (parameter, ty) in parameters {
                    self.define(
                        parameter.clone(),
                        ty.as_ref()
                            .map(type_from_ast)
                            .unwrap_or(VelaRiType::Unknown),
                    );
                }
                let old_in_function = self.in_function;
                let old_return_type = self.return_type.clone();
                self.in_function = true;
                self.return_type = return_type.as_ref().map(type_from_ast);
                for statement in body {
                    self.stmt(statement, errors);
                }
                if !self.block_returns(body) {
                    errors.push(SemanticError {
                        message: format!("function `{name}` can fall through without returning"),
                        span: *span,
                    });
                }
                self.in_function = old_in_function;
                self.return_type = old_return_type;
                self.pop();
            }
            Stmt::Conditional {
                condition,
                then_branch,
                otherwise_branch,
                ..
            } => {
                if self.expr(condition, errors) != Ok(VelaRiType::Boolean) {
                    errors.push(SemanticError {
                        message: "if condition must be Boolean".into(),
                        span: condition.span(),
                    });
                }
                self.push();
                for statement in then_branch {
                    self.stmt(statement, errors);
                }
                self.pop();
                if let Some(otherwise_branch) = otherwise_branch {
                    self.push();
                    for statement in otherwise_branch {
                        self.stmt(statement, errors);
                    }
                    self.pop();
                }
            }
            Stmt::RepeatLoop {
                iterations, body, ..
            } => {
                if !matches!(
                    self.expr(iterations, errors),
                    Ok(VelaRiType::Int | VelaRiType::Number)
                ) {
                    errors.push(SemanticError {
                        message: "repeat count must be Number".into(),
                        span: iterations.span(),
                    });
                }
                self.push();
                for statement in body {
                    self.stmt(statement, errors);
                }
                self.pop();
            }
            Stmt::CreateWindow { width, height, .. } => {
                for expression in [width, height] {
                    if !matches!(
                        self.expr(expression, errors),
                        Ok(VelaRiType::Int | VelaRiType::Number)
                    ) {
                        errors.push(SemanticError {
                            message: "window dimensions must be Number".into(),
                            span: expression.span(),
                        });
                    }
                }
            }
        }
    }

    fn block_returns(&self, statements: &[Stmt]) -> bool {
        statements
            .iter()
            .any(|statement| self.statement_returns(statement))
    }

    fn statement_returns(&self, statement: &Stmt) -> bool {
        match statement {
            Stmt::Return { .. } => true,
            Stmt::Block(statements) => self.block_returns(statements),
            Stmt::Conditional {
                then_branch,
                otherwise_branch: Some(otherwise_branch),
                ..
            } => self.block_returns(then_branch) && self.block_returns(otherwise_branch),
            _ => false,
        }
    }

    fn expr(&self, expression: &Expr, errors: &mut Vec<SemanticError>) -> Result<VelaRiType, ()> {
        match expression {
            Expr::Literal(value, _) => Ok(match value {
                Value::Number(number) if number.fract() == 0.0 => VelaRiType::Int,
                Value::Number(_) => VelaRiType::Number,
                Value::Boolean(_) => VelaRiType::Boolean,
                Value::String(_) => VelaRiType::String,
                Value::Null => VelaRiType::Null,
                Value::Array(values) => {
                    let element = values
                        .iter()
                        .map(value_type)
                        .fold(VelaRiType::Unknown, |a, b| merge_types(&a, &b));
                    VelaRiType::Array(Box::new(element))
                }
                Value::Map(values) => {
                    let value = values
                        .values()
                        .map(value_type)
                        .fold(VelaRiType::Unknown, |a, b| merge_types(&a, &b));
                    VelaRiType::Map(Box::new(VelaRiType::String), Box::new(value))
                }
            }),
            Expr::Variable(name, span) => self.lookup(name).ok_or_else(|| {
                errors.push(SemanticError {
                    message: format!("variable `{name}` used before initialization"),
                    span: *span,
                });
            }),
            Expr::Array(elements, _) => {
                let mut element_type = VelaRiType::Unknown;
                for element in elements {
                    if let Ok(ty) = self.expr(element, errors) {
                        element_type = merge_types(&element_type, &ty);
                    }
                }
                Ok(VelaRiType::Array(Box::new(element_type)))
            }
            Expr::Map(entries, span) => {
                let mut value_type = VelaRiType::Unknown;
                for (_, value) in entries {
                    if let Ok(ty) = self.expr(value, errors) {
                        value_type = merge_types(&value_type, &ty);
                    }
                }
                let _ = span;
                Ok(VelaRiType::Map(
                    Box::new(VelaRiType::String),
                    Box::new(value_type),
                ))
            }
            Expr::Index {
                target,
                index,
                span,
            } => {
                let target_type = self.expr(target, errors)?;
                let index_type = self.expr(index, errors)?;
                match target_type {
                    VelaRiType::Array(element) => {
                        self.report_mismatch(&VelaRiType::Int, &index_type, *span, errors);
                        Ok(*element)
                    }
                    VelaRiType::Map(key, value) => {
                        self.report_mismatch(&key, &index_type, *span, errors);
                        Ok(*value)
                    }
                    VelaRiType::Unknown => Ok(VelaRiType::Unknown),
                    actual => {
                        errors.push(SemanticError {
                            message: format!("cannot index value of type {}", type_name(&actual)),
                            span: *span,
                        });
                        Err(())
                    }
                }
            }
            Expr::Call {
                name,
                arguments,
                span,
            } => {
                if name == "length" {
                    if arguments.len() != 1 {
                        errors.push(SemanticError {
                            message: "length expects one argument".into(),
                            span: *span,
                        });
                    }
                    for argument in arguments {
                        let _ = self.expr(argument, errors);
                    }
                    return Ok(VelaRiType::Int);
                }
                let Some(signature) = self.functions.get(name) else {
                    errors.push(SemanticError {
                        message: format!("unknown function `{name}`"),
                        span: *span,
                    });
                    for argument in arguments {
                        let _ = self.expr(argument, errors);
                    }
                    return Ok(VelaRiType::Unknown);
                };
                if signature.parameters.len() != arguments.len() {
                    errors.push(SemanticError {
                        message: format!(
                            "function `{name}` expects {} arguments, got {}",
                            signature.parameters.len(),
                            arguments.len()
                        ),
                        span: *span,
                    });
                }
                for (position, argument) in arguments.iter().enumerate() {
                    if let Ok(actual) = self.expr(argument, errors) {
                        if let Some(expected) = signature.parameters.get(position) {
                            self.report_mismatch(expected, &actual, argument.span(), errors);
                        }
                    }
                }
                Ok(signature.result.clone())
            }
            Expr::Unary { op, expr, span } => {
                let ty = self.expr(expr, errors)?;
                if *op == Op::Neg && matches!(ty, VelaRiType::Int | VelaRiType::Number) {
                    Ok(ty)
                } else {
                    errors.push(SemanticError {
                        message: "unary minus requires Number".into(),
                        span: *span,
                    });
                    Err(())
                }
            }
            Expr::Binary {
                left,
                op,
                right,
                span,
            } => {
                let left_type = self.expr(left, errors)?;
                let right_type = self.expr(right, errors)?;
                match op {
                    Op::Add | Op::Sub | Op::Mul | Op::Div
                        if (matches!(left_type, VelaRiType::Int | VelaRiType::Number)
                            && matches!(right_type, VelaRiType::Int | VelaRiType::Number))
                            || left_type == VelaRiType::Unknown
                            || right_type == VelaRiType::Unknown =>
                    {
                        Ok(merge_types(&left_type, &right_type))
                    }
                    Op::Add if left_type == VelaRiType::String && right_type == left_type => {
                        Ok(VelaRiType::String)
                    }
                    Op::Eq if compatible(&left_type, &right_type) => Ok(VelaRiType::Boolean),
                    Op::And | Op::Or
                        if left_type == VelaRiType::Boolean && right_type == left_type =>
                    {
                        Ok(VelaRiType::Boolean)
                    }
                    _ => {
                        errors.push(SemanticError {
                            message: format!(
                                "invalid operation {op:?} for {} and {}",
                                type_name(&left_type),
                                type_name(&right_type)
                            ),
                            span: *span,
                        });
                        Err(())
                    }
                }
            }
        }
    }
}

fn value_type(value: &Value) -> VelaRiType {
    match value {
        Value::Number(number) if number.fract() == 0.0 => VelaRiType::Int,
        Value::Number(_) => VelaRiType::Number,
        Value::Boolean(_) => VelaRiType::Boolean,
        Value::String(_) => VelaRiType::String,
        Value::Null => VelaRiType::Null,
        Value::Array(values) => VelaRiType::Array(Box::new(
            values
                .iter()
                .map(value_type)
                .fold(VelaRiType::Unknown, |a, b| merge_types(&a, &b)),
        )),
        Value::Map(values) => VelaRiType::Map(
            Box::new(VelaRiType::String),
            Box::new(
                values
                    .values()
                    .map(value_type)
                    .fold(VelaRiType::Unknown, |a, b| merge_types(&a, &b)),
            ),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{lexer::Lexer, parser::Parser};

    fn analyze(source: &str) -> Result<(), Vec<SemanticError>> {
        let tokens = Lexer::new(source).tokenize().expect("tokens");
        let ast = Parser::new(tokens).parse_program().expect("ast");
        SemanticAnalyzer::new().analyze(&ast)
    }

    #[test]
    fn infers_collection_element_types() {
        let source = "begin\nlet values be [1, 2, 3]\nlet names be {\"first\": \"VelaRi\"}\nprint values[1]\nprint names[\"first\"]\nend";
        assert!(analyze(source).is_ok());
    }

    #[test]
    fn rejects_invalid_collection_annotation() {
        let source = "begin\nlet values: Array<Int> be [1, \"wrong\"]\nend";
        let errors = analyze(source).expect_err("mixed array should fail");
        assert!(errors
            .iter()
            .any(|error| error.message.contains("Array<Int>")));
    }

    #[test]
    fn validates_function_arguments_and_returns() {
        let source = "begin\nfunction add(a: Int, b: Int): Int\nreturn \"wrong\"\nend\nprint add(\"wrong\", true)\nend";
        let errors = analyze(source).expect_err("invalid call should fail");
        assert!(errors
            .iter()
            .any(|error| error.message.contains("expected Int")));
    }
}
