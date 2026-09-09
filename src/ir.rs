use std::collections::HashMap;

use crate::ast::{Expr, Op, Stmt, Type, Value};

#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    Const {
        dest: String,
        value: Value,
        ty: Type,
    },
    Load {
        dest: String,
        name: String,
        ty: Type,
    },
    Store {
        name: String,
        value: String,
        ty: Type,
    },
    Binary {
        dest: String,
        op: Op,
        left: String,
        right: String,
        ty: Type,
    },
    Call {
        dest: String,
        name: String,
        args: Vec<String>,
        ty: Type,
    },
    Index {
        dest: String,
        target: String,
        index: String,
        ty: Type,
    },
    Aggregate {
        dest: String,
        elements: Vec<String>,
        ty: Type,
    },
    Print {
        value: String,
    },
    Assert {
        value: String,
    },
    Return(Option<String>),
    Label(String),
    Branch {
        condition: String,
        then_label: String,
        else_label: String,
    },
    Jump(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct IrFunction {
    pub name: String,
    pub parameters: Vec<(String, Type)>,
    pub return_type: Type,
    pub instructions: Vec<Instruction>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IrProgram {
    pub functions: Vec<IrFunction>,
}

#[derive(Default)]
pub struct Lowerer {
    next: usize,
    functions: HashMap<String, (Vec<(String, Type)>, Type)>,
}

struct FunctionLowerer<'a> {
    parent: &'a mut Lowerer,
    variables: HashMap<String, Type>,
    instructions: Vec<Instruction>,
}

impl Lowerer {
    pub fn lower(root: &Stmt) -> IrProgram {
        let mut lowerer = Self::default();
        lowerer.collect_functions(root);

        let mut functions = Vec::new();
        if let Stmt::Block(statements) = root {
            let main_body = statements
                .iter()
                .filter(|statement| !matches!(statement, Stmt::Function { .. }))
                .cloned()
                .collect::<Vec<_>>();
            functions.push(lowerer.lower_function("main", &[], Type::Null, &main_body));

            for statement in statements {
                if let Stmt::Function {
                    name,
                    parameters,
                    return_type,
                    body,
                    ..
                } = statement
                {
                    let parameter_types = parameters
                        .iter()
                        .map(|(parameter, annotation)| {
                            (
                                parameter.clone(),
                                annotation.clone().unwrap_or(Type::Unknown),
                            )
                        })
                        .collect::<Vec<_>>();
                    functions.push(lowerer.lower_function(
                        name,
                        &parameter_types,
                        return_type.clone().unwrap_or(Type::Unknown),
                        body,
                    ));
                }
            }
        }

        IrProgram { functions }
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
                let parameter_types = parameters
                    .iter()
                    .map(|(parameter, annotation)| {
                        (
                            parameter.clone(),
                            annotation.clone().unwrap_or(Type::Unknown),
                        )
                    })
                    .collect::<Vec<_>>();
                self.functions.insert(
                    name.clone(),
                    (
                        parameter_types,
                        return_type.clone().unwrap_or(Type::Unknown),
                    ),
                );
            }
            _ => {}
        }
    }

    fn lower_function(
        &mut self,
        name: &str,
        parameters: &[(String, Type)],
        return_type: Type,
        body: &[Stmt],
    ) -> IrFunction {
        let mut function = FunctionLowerer {
            parent: self,
            variables: parameters.iter().cloned().collect(),
            instructions: Vec::new(),
        };
        for statement in body {
            function.stmt(statement);
        }
        IrFunction {
            name: name.to_string(),
            parameters: parameters.to_vec(),
            return_type,
            instructions: function.instructions,
        }
    }

    fn temp(&mut self) -> String {
        let name = format!("%{}", self.next);
        self.next += 1;
        name
    }

    fn label(&mut self, prefix: &str) -> String {
        let name = format!("{prefix}{}", self.next);
        self.next += 1;
        name
    }
}

impl FunctionLowerer<'_> {
    fn temp(&mut self) -> String {
        self.parent.temp()
    }

    fn label(&mut self, prefix: &str) -> String {
        self.parent.label(prefix)
    }

    fn expr(&mut self, expression: &Expr) -> (String, Type) {
        match expression {
            Expr::Literal(value, _) => {
                let dest = self.temp();
                let ty = infer_value(value);
                self.instructions.push(Instruction::Const {
                    dest: dest.clone(),
                    value: value.clone(),
                    ty: ty.clone(),
                });
                (dest, ty)
            }
            Expr::Variable(name, _) => {
                let dest = self.temp();
                let ty = self.variables.get(name).cloned().unwrap_or(Type::Unknown);
                self.instructions.push(Instruction::Load {
                    dest: dest.clone(),
                    name: name.clone(),
                    ty: ty.clone(),
                });
                (dest, ty)
            }
            Expr::Array(elements, _) => {
                let mut values = Vec::new();
                let mut element_type = Type::Unknown;
                for element in elements {
                    let (value, ty) = self.expr(element);
                    values.push(value);
                    element_type = merge_types(&element_type, &ty);
                }
                let ty = Type::Array(Box::new(element_type));
                let dest = self.temp();
                self.instructions.push(Instruction::Aggregate {
                    dest: dest.clone(),
                    elements: values,
                    ty: ty.clone(),
                });
                (dest, ty)
            }
            Expr::Map(entries, _) => {
                let mut values = Vec::new();
                let mut value_type = Type::Unknown;
                for (_, expression) in entries {
                    let (value, ty) = self.expr(expression);
                    values.push(value);
                    value_type = merge_types(&value_type, &ty);
                }
                let ty = Type::Map(Box::new(Type::String), Box::new(value_type));
                let dest = self.temp();
                self.instructions.push(Instruction::Aggregate {
                    dest: dest.clone(),
                    elements: values,
                    ty: ty.clone(),
                });
                (dest, ty)
            }
            Expr::Index { target, index, .. } => {
                let (target_value, target_type) = self.expr(target);
                let (index_value, _) = self.expr(index);
                let ty = match target_type {
                    Type::Array(element_type) => *element_type,
                    Type::Map(_, value_type) => *value_type,
                    _ => Type::Unknown,
                };
                let dest = self.temp();
                self.instructions.push(Instruction::Index {
                    dest: dest.clone(),
                    target: target_value,
                    index: index_value,
                    ty: ty.clone(),
                });
                (dest, ty)
            }
            Expr::Binary {
                left, op, right, ..
            } => {
                let (left_value, left_type) = self.expr(left);
                let (right_value, right_type) = self.expr(right);
                let ty = binary_type(*op, &left_type, &right_type);
                let dest = self.temp();
                self.instructions.push(Instruction::Binary {
                    dest: dest.clone(),
                    op: *op,
                    left: left_value,
                    right: right_value,
                    ty: ty.clone(),
                });
                (dest, ty)
            }
            Expr::Unary { expr, .. } => self.expr(expr),
            Expr::Call {
                name, arguments, ..
            } => {
                let mut args = Vec::new();
                for argument in arguments {
                    args.push(self.expr(argument).0);
                }
                let ty = self
                    .parent
                    .functions
                    .get(name)
                    .map(|(_, result)| result.clone())
                    .unwrap_or(Type::Unknown);
                let dest = self.temp();
                self.instructions.push(Instruction::Call {
                    dest: dest.clone(),
                    name: name.clone(),
                    args,
                    ty: ty.clone(),
                });
                (dest, ty)
            }
        }
    }

    fn stmt(&mut self, statement: &Stmt) {
        match statement {
            Stmt::Block(statements) => {
                for statement in statements {
                    self.stmt(statement);
                }
            }
            Stmt::Let {
                name,
                annotation,
                value,
                ..
            } => {
                let (value, inferred_type) = self.expr(value);
                let ty = annotation.clone().unwrap_or(inferred_type);
                self.variables.insert(name.clone(), ty.clone());
                self.instructions.push(Instruction::Store {
                    name: name.clone(),
                    value,
                    ty,
                });
            }
            Stmt::Assign { name, value, .. } => {
                let (value, inferred_type) = self.expr(value);
                let ty = self.variables.get(name).cloned().unwrap_or(inferred_type);
                self.instructions.push(Instruction::Store {
                    name: name.clone(),
                    value,
                    ty,
                });
            }
            Stmt::Print(expression) => {
                let (value, _) = self.expr(expression);
                self.instructions.push(Instruction::Print { value });
            }
            Stmt::Assert(expression) => {
                let (value, _) = self.expr(expression);
                self.instructions.push(Instruction::Assert { value });
            }
            Stmt::Return { value, .. } => {
                let value = value.as_ref().map(|expression| self.expr(expression).0);
                self.instructions.push(Instruction::Return(value));
            }
            Stmt::Conditional {
                condition,
                then_branch,
                otherwise_branch,
                ..
            } => {
                let (condition, _) = self.expr(condition);
                let then_label = self.label("then");
                let else_label = self.label("else");
                let join_label = self.label("join");
                self.instructions.push(Instruction::Branch {
                    condition,
                    then_label: then_label.clone(),
                    else_label: else_label.clone(),
                });
                self.instructions.push(Instruction::Label(then_label));
                for statement in then_branch {
                    self.stmt(statement);
                }
                self.instructions
                    .push(Instruction::Jump(join_label.clone()));
                self.instructions.push(Instruction::Label(else_label));
                if let Some(otherwise_branch) = otherwise_branch {
                    for statement in otherwise_branch {
                        self.stmt(statement);
                    }
                }
                self.instructions
                    .push(Instruction::Jump(join_label.clone()));
                self.instructions.push(Instruction::Label(join_label));
            }
            Stmt::RepeatLoop {
                iterations, body, ..
            } => {
                let (iterations, _) = self.expr(iterations);
                let body_label = self.label("repeat_body");
                let exit_label = self.label("repeat_exit");
                self.instructions.push(Instruction::Branch {
                    condition: iterations,
                    then_label: body_label.clone(),
                    else_label: exit_label.clone(),
                });
                self.instructions.push(Instruction::Label(body_label));
                for statement in body {
                    self.stmt(statement);
                }
                self.instructions
                    .push(Instruction::Jump(exit_label.clone()));
                self.instructions.push(Instruction::Label(exit_label));
            }
            Stmt::Function { .. } | Stmt::CreateWindow { .. } => {}
        }
    }
}

fn infer_value(value: &Value) -> Type {
    match value {
        Value::Number(number) if number.fract() == 0.0 => Type::Int,
        Value::Number(_) => Type::Float,
        Value::Boolean(_) => Type::Bool,
        Value::String(_) => Type::String,
        Value::Null => Type::Null,
        Value::Array(values) => {
            let element_type = values
                .iter()
                .map(infer_value)
                .fold(Type::Unknown, |a, b| merge_types(&a, &b));
            Type::Array(Box::new(element_type))
        }
        Value::Map(values) => {
            let value_type = values
                .values()
                .map(infer_value)
                .fold(Type::Unknown, |a, b| merge_types(&a, &b));
            Type::Map(Box::new(Type::String), Box::new(value_type))
        }
    }
}

fn merge_types(left: &Type, right: &Type) -> Type {
    if left == &Type::Unknown {
        return right.clone();
    }
    if right == &Type::Unknown || left == right {
        return left.clone();
    }
    match (left, right) {
        (Type::Array(left), Type::Array(right)) => Type::Array(Box::new(merge_types(left, right))),
        (Type::Map(left_key, left_value), Type::Map(right_key, right_value)) => Type::Map(
            Box::new(merge_types(left_key, right_key)),
            Box::new(merge_types(left_value, right_value)),
        ),
        (Type::Int, Type::Float) | (Type::Float, Type::Int) => Type::Float,
        _ => Type::Unknown,
    }
}

fn binary_type(op: Op, left: &Type, right: &Type) -> Type {
    match op {
        Op::Eq | Op::And | Op::Or => Type::Bool,
        Op::Add | Op::Sub | Op::Mul | Op::Div => merge_types(left, right),
        _ => Type::Unknown,
    }
}

impl IrProgram {
    pub fn text(&self) -> String {
        let mut output = String::new();
        for function in &self.functions {
            output.push_str(&format!(
                "function {}({}) -> {}:\n",
                function.name,
                function
                    .parameters
                    .iter()
                    .map(|(name, ty)| format!("{name}: {}", ty.display()))
                    .collect::<Vec<_>>()
                    .join(", "),
                function.return_type.display()
            ));
            for instruction in &function.instructions {
                output.push_str(&format!("  {instruction:?}\n"));
            }
        }
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{lexer::Lexer, parser::Parser};

    #[test]
    fn infers_typed_collections_and_lowers_functions() {
        let source = "begin\nfunction make(): Array<Int>\n return [1, 2, 3]\nend\nlet xs be make()\nprint xs[1]\nend";
        let tokens = Lexer::new(source).tokenize().expect("tokens");
        let ast = Parser::new(tokens).parse_program().expect("ast");
        let ir = Lowerer::lower(&ast);
        assert_eq!(ir.functions.len(), 2);
        assert!(ir.functions[0].instructions.iter().any(|instruction| {
            matches!(instruction, Instruction::Call { ty: Type::Array(element), .. } if **element == Type::Int)
        }));
        assert!(ir.functions[1].instructions.iter().any(|instruction| {
            matches!(instruction, Instruction::Aggregate { ty: Type::Array(element), .. } if **element == Type::Int)
        }));
    }
}
