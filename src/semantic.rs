use crate::{
    ast::{Expr, Op, Stmt, Value},
    lexer::Span,
};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VelaRiType {
    Int,
    Number,
    Boolean,
    String,
    Null,
    Unknown,
}
#[derive(Debug, Clone, PartialEq)]
pub struct SemanticError {
    pub message: String,
    pub span: Span,
}
#[derive(Clone)]
struct FunctionSig {
    parameters: usize,
    result: VelaRiType,
}

pub struct SemanticAnalyzer {
    scopes: Vec<HashMap<String, VelaRiType>>,
    functions: HashMap<String, FunctionSig>,
    in_function: bool,
}
fn type_from_ast(t: &crate::ast::Type) -> VelaRiType {
    match t {
        crate::ast::Type::Int => VelaRiType::Int,
        crate::ast::Type::Float => VelaRiType::Number,
        crate::ast::Type::Bool => VelaRiType::Boolean,
        crate::ast::Type::String => VelaRiType::String,
        crate::ast::Type::Null => VelaRiType::Null,
        _ => VelaRiType::Unknown,
    }
}
fn expected_name(t: VelaRiType) -> &'static str {
    match t {
        VelaRiType::Int => "Int",
        VelaRiType::Number => "Float",
        VelaRiType::Boolean => "Boolean",
        VelaRiType::String => "String",
        VelaRiType::Null => "Null",
        VelaRiType::Unknown => "Unknown",
    }
}
impl SemanticAnalyzer {
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
            functions: HashMap::new(),
            in_function: false,
        }
    }
    pub fn analyze(&mut self, root: &Stmt) -> Result<(), Vec<SemanticError>> {
        let mut e = Vec::new();
        self.collect_functions(root);
        self.stmt(root, &mut e);
        if e.is_empty() {
            Ok(())
        } else {
            Err(e)
        }
    }
    fn collect_functions(&mut self, s: &Stmt) {
        match s {
            Stmt::Block(xs) => {
                for x in xs {
                    self.collect_functions(x)
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
                        parameters: parameters.len(),
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
        self.scopes.push(HashMap::new())
    }
    fn pop(&mut self) {
        self.scopes.pop();
    }
    fn define(&mut self, n: String, t: VelaRiType) {
        self.scopes.last_mut().unwrap().insert(n, t);
    }
    fn lookup(&self, n: &str) -> Option<VelaRiType> {
        self.scopes.iter().rev().find_map(|s| s.get(n).copied())
    }
    fn stmt(&mut self, s: &Stmt, e: &mut Vec<SemanticError>) {
        match s {
            Stmt::Block(xs) => {
                for x in xs {
                    self.stmt(x, e)
                }
            }
            Stmt::Let {
                name,
                annotation,
                value,
                span,
            } => {
                if let Ok(t) = self.expr(value, e) {
                    if let Some(expected) = annotation {
                        let expected = match expected {
                            crate::ast::Type::Int => VelaRiType::Int,
                            crate::ast::Type::Float => VelaRiType::Number,
                            crate::ast::Type::Bool => VelaRiType::Boolean,
                            crate::ast::Type::String => VelaRiType::String,
                            crate::ast::Type::Null => VelaRiType::Null,
                            _ => VelaRiType::Unknown,
                        };
                        if expected != VelaRiType::Unknown && t != expected {
                            e.push(SemanticError {
                                message: format!(
                                    "type mismatch: expected {}, found {:?}",
                                    expected_name(expected),
                                    t
                                ),
                                span: *span,
                            });
                        }
                    }
                    if self.scopes.last().unwrap().contains_key(name) {
                        e.push(SemanticError {
                            message: format!("variable `{name}` is already declared in this scope"),
                            span: *span,
                        })
                    } else {
                        self.define(name.clone(), t)
                    }
                }
            }
            Stmt::Assign { name, value, span } => {
                if self.lookup(name).is_none() {
                    e.push(SemanticError {
                        message: format!("variable `{name}` used before initialization"),
                        span: *span,
                    })
                }
                let _ = self.expr(value, e);
            }
            Stmt::Print(x) => {
                let _ = self.expr(x, e);
            }
            Stmt::Assert(x) => {
                if self.expr(x, e) != Ok(VelaRiType::Boolean) {
                    e.push(SemanticError {
                        message: "assert condition must be Boolean".into(),
                        span: x.span(),
                    });
                }
            }
            Stmt::Return { value, span } => {
                if !self.in_function {
                    e.push(SemanticError {
                        message: "return is only valid inside a function".into(),
                        span: *span,
                    })
                }
                if let Some(x) = value {
                    let _ = self.expr(x, e);
                }
            }
            Stmt::Function {
                name,
                parameters,
                body,
                span,
                ..
            } => {
                self.push();
                for (p, t) in parameters {
                    self.define(
                        p.clone(),
                        t.as_ref()
                            .map(|x| match x {
                                crate::ast::Type::Int => VelaRiType::Int,
                                crate::ast::Type::Float => VelaRiType::Number,
                                crate::ast::Type::Bool => VelaRiType::Boolean,
                                crate::ast::Type::String => VelaRiType::String,
                                crate::ast::Type::Null => VelaRiType::Null,
                                _ => VelaRiType::Unknown,
                            })
                            .unwrap_or(VelaRiType::Unknown),
                    );
                }
                let old = self.in_function;
                self.in_function = true;
                for x in body {
                    self.stmt(x, e);
                }
                if !self.block_returns(body) {
                    e.push(SemanticError {
                        message: format!("function `{name}` can fall through without returning"),
                        span: *span,
                    });
                }
                self.in_function = old;
                self.pop();
            }
            Stmt::Conditional {
                condition,
                then_branch,
                otherwise_branch,
                ..
            } => {
                if self.expr(condition, e) != Ok(VelaRiType::Boolean) {
                    e.push(SemanticError {
                        message: "if condition must be Boolean".into(),
                        span: condition.span(),
                    });
                }
                self.push();
                for x in then_branch {
                    self.stmt(x, e);
                }
                self.pop();
                if let Some(xs) = otherwise_branch {
                    self.push();
                    for x in xs {
                        self.stmt(x, e);
                    }
                    self.pop();
                }
            }
            Stmt::RepeatLoop {
                iterations, body, ..
            } => {
                if !matches!(
                    self.expr(iterations, e),
                    Ok(VelaRiType::Int | VelaRiType::Number)
                ) {
                    e.push(SemanticError {
                        message: "repeat count must be Number".into(),
                        span: iterations.span(),
                    });
                }
                self.push();
                for x in body {
                    self.stmt(x, e);
                }
                self.pop();
            }
            Stmt::CreateWindow { width, height, .. } => {
                for x in [width, height] {
                    if self.expr(x, e) != Ok(VelaRiType::Number) {
                        e.push(SemanticError {
                            message: "window dimensions must be Number".into(),
                            span: x.span(),
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
    fn expr(&self, x: &Expr, e: &mut Vec<SemanticError>) -> Result<VelaRiType, ()> {
        match x {
            Expr::Literal(v, _) => Ok(match v {
                Value::Number(n) => {
                    if n.fract() == 0.0 {
                        VelaRiType::Int
                    } else {
                        VelaRiType::Number
                    }
                }
                Value::Boolean(_) => VelaRiType::Boolean,
                Value::String(_) => VelaRiType::String,
                Value::Null => VelaRiType::Null,
                Value::Array(_) | Value::Map(_) => VelaRiType::Unknown,
            }),
            Expr::Variable(n, sp) => self.lookup(n).ok_or_else(|| {
                e.push(SemanticError {
                    message: format!("variable `{n}` used before initialization"),
                    span: *sp,
                })
            }),
            Expr::Array(xs, _) => {
                for x in xs {
                    let _ = self.expr(x, e);
                }
                Ok(VelaRiType::Unknown)
            }
            Expr::Map(xs, _) => {
                for (_, x) in xs {
                    let _ = self.expr(x, e);
                }
                Ok(VelaRiType::Unknown)
            }
            Expr::Index {
                target,
                index,
                span: _,
            } => {
                let _ = self.expr(target, e);
                let _ = self.expr(index, e);
                Ok(VelaRiType::Unknown)
            }
            Expr::Call {
                name,
                arguments,
                span,
            } => {
                if name == "length" {
                    if arguments.len() != 1 {
                        e.push(SemanticError {
                            message: "length expects one argument".into(),
                            span: *span,
                        });
                    }
                    for a in arguments {
                        let _ = self.expr(a, e);
                    }
                    return Ok(VelaRiType::Number);
                }
                match self.functions.get(name) {
                    Some(sig) if sig.parameters == arguments.len() => {
                        for a in arguments {
                            let _ = self.expr(a, e);
                        }
                        return Ok(sig.result);
                    }
                    Some(sig) => e.push(SemanticError {
                        message: format!(
                            "function `{name}` expects {} arguments, got {}",
                            sig.parameters,
                            arguments.len()
                        ),
                        span: *span,
                    }),
                    None => e.push(SemanticError {
                        message: format!("unknown function `{name}`"),
                        span: *span,
                    }),
                }
                for a in arguments {
                    let _ = self.expr(a, e);
                }
                Ok(VelaRiType::Unknown)
            }
            Expr::Unary { op, expr, span } => {
                let t = self.expr(expr, e).map_err(|_| ())?;
                if *op == Op::Neg && matches!(t, VelaRiType::Int | VelaRiType::Number) {
                    Ok(t)
                } else {
                    e.push(SemanticError {
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
                let a = self.expr(left, e).map_err(|_| ())?;
                let b = self.expr(right, e).map_err(|_| ())?;
                match op {
                    Op::Add | Op::Sub | Op::Mul | Op::Div
                        if (matches!(a, VelaRiType::Int | VelaRiType::Number)
                            && matches!(b, VelaRiType::Int | VelaRiType::Number))
                            || (a == VelaRiType::Unknown || b == VelaRiType::Unknown) =>
                    {
                        Ok(if a == VelaRiType::Unknown || b == VelaRiType::Unknown {
                            VelaRiType::Unknown
                        } else {
                            a
                        })
                    }
                    Op::Add if a == VelaRiType::String && b == a => Ok(a),
                    Op::Eq if a == b || a == VelaRiType::Unknown || b == VelaRiType::Unknown => {
                        Ok(VelaRiType::Boolean)
                    }
                    Op::And | Op::Or if a == VelaRiType::Boolean && b == a => {
                        Ok(VelaRiType::Boolean)
                    }
                    _ => {
                        e.push(SemanticError {
                            message: format!("invalid operation {op:?} for {a:?} and {b:?}"),
                            span: *span,
                        });
                        Err(())
                    }
                }
            }
        }
    }
}
