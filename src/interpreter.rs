use crate::ast::{Expr, Op, Stmt, Value};
use std::collections::HashMap;
#[derive(Debug)]
pub struct RuntimeError(pub String);
enum Flow {
    Continue,
    Return(Value),
}
pub struct Interpreter {
    scopes: Vec<HashMap<String, Value>>,
    functions: HashMap<String, (Vec<String>, Vec<Stmt>)>,
}
impl Interpreter {
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
            functions: HashMap::new(),
        }
    }
    pub fn execute(&mut self, s: &Stmt) -> Result<(), RuntimeError> {
        self.collect(s);
        match self.exec(s)? {
            Flow::Continue => Ok(()),
            Flow::Return(_) => Err(RuntimeError("return outside function".into())),
        }
    }
    fn collect(&mut self, s: &Stmt) {
        match s {
            Stmt::Block(xs) => {
                for x in xs {
                    self.collect(x)
                }
            }
            Stmt::Function {
                name,
                parameters,
                body,
                ..
            } => {
                self.functions.insert(
                    name.clone(),
                    (
                        parameters.iter().map(|(n, _)| n.clone()).collect(),
                        body.clone(),
                    ),
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
    fn lookup(&self, n: &str) -> Option<Value> {
        self.scopes.iter().rev().find_map(|s| s.get(n).cloned())
    }
    fn define(&mut self, n: String, v: Value) {
        self.scopes.last_mut().unwrap().insert(n, v);
    }
    fn assign(&mut self, n: &str, v: Value) -> bool {
        for s in self.scopes.iter_mut().rev() {
            if s.contains_key(n) {
                s.insert(n.to_string(), v);
                return true;
            }
        }
        false
    }
    fn exec(&mut self, s: &Stmt) -> Result<Flow, RuntimeError> {
        match s {
            Stmt::Block(xs) => {
                for x in xs {
                    if let Flow::Return(v) = self.exec(x)? {
                        return Ok(Flow::Return(v));
                    }
                }
                Ok(Flow::Continue)
            }
            Stmt::Let { name, value, .. } => {
                let v = self.eval(value)?;
                self.define(name.clone(), v);
                Ok(Flow::Continue)
            }
            Stmt::Assign { name, value, .. } => {
                let v = self.eval(value)?;
                if !self.assign(name, v) {
                    return Err(RuntimeError(format!("undefined variable `{name}`")));
                }
                Ok(Flow::Continue)
            }
            Stmt::Print(x) => {
                println!("{}", self.eval(x)?.display());
                Ok(Flow::Continue)
            }
            Stmt::Assert(x) => match self.eval(x)? {
                Value::Boolean(true) => Ok(Flow::Continue),
                Value::Boolean(false) => Err(RuntimeError("assertion failed".into())),
                _ => Err(RuntimeError("assert condition must be Boolean".into())),
            },
            Stmt::Return { value, .. } => Ok(Flow::Return(
                value
                    .as_ref()
                    .map(|x| self.eval(x))
                    .transpose()?
                    .unwrap_or(Value::Null),
            )),
            Stmt::Function { .. } => Ok(Flow::Continue),
            Stmt::Conditional {
                condition,
                then_branch,
                otherwise_branch,
                ..
            } => {
                let yes = matches!(self.eval(condition)?, Value::Boolean(true));
                let xs = if yes {
                    Some(then_branch)
                } else {
                    otherwise_branch.as_ref()
                };
                if let Some(xs) = xs {
                    self.push();
                    let r = self.exec(&Stmt::Block(xs.clone()))?;
                    self.pop();
                    Ok(r)
                } else {
                    Ok(Flow::Continue)
                }
            }
            Stmt::RepeatLoop {
                iterations, body, ..
            } => {
                let n = match self.eval(iterations)? {
                    Value::Number(n) if n >= 0.0 => n as usize,
                    _ => {
                        return Err(RuntimeError(
                            "repeat count must be a non-negative Number".into(),
                        ))
                    }
                };
                for _ in 0..n {
                    self.push();
                    let r = self.exec(&Stmt::Block(body.clone()))?;
                    self.pop();
                    if let Flow::Return(v) = r {
                        return Ok(Flow::Return(v));
                    }
                }
                Ok(Flow::Continue)
            }
            Stmt::CreateWindow {
                title,
                width,
                height,
                ..
            } => {
                println!(
                    "[VelaRi] window {:?} {}x{}",
                    title,
                    self.number(width)?,
                    self.number(height)?
                );
                Ok(Flow::Continue)
            }
        }
    }
    fn number(&mut self, x: &Expr) -> Result<f64, RuntimeError> {
        match self.eval(x)? {
            Value::Number(n) => Ok(n),
            _ => Err(RuntimeError("expected Number".into())),
        }
    }
    fn eval(&mut self, x: &Expr) -> Result<Value, RuntimeError> {
        match x {
            Expr::Literal(v, _) => Ok(v.clone()),
            Expr::Variable(n, _) => self
                .lookup(n)
                .ok_or_else(|| RuntimeError(format!("undefined variable `{n}`"))),
            Expr::Array(xs, _) => Ok(Value::Array(
                xs.iter()
                    .map(|x| self.eval(x))
                    .collect::<Result<Vec<_>, _>>()?,
            )),
            Expr::Map(xs, _) => {
                let mut m = HashMap::new();
                for (k, v) in xs {
                    m.insert(k.clone(), self.eval(v)?);
                }
                Ok(Value::Map(m))
            }
            Expr::Index { target, index, .. } => {
                let collection = self.eval(target)?;
                let key = self.eval(index)?;
                match (collection, key) {
                    (Value::Array(xs), Value::Number(n)) => xs
                        .get(n as usize)
                        .cloned()
                        .ok_or_else(|| RuntimeError("array index out of bounds".into())),
                    (Value::Map(m), Value::String(k)) => m
                        .get(&k)
                        .cloned()
                        .ok_or_else(|| RuntimeError(format!("missing map key `{k}`"))),
                    _ => Err(RuntimeError(
                        "indexing requires an array and number or map and string".into(),
                    )),
                }
            }
            Expr::Call {
                name, arguments, ..
            } => {
                if name == "length" {
                    if arguments.len() != 1 {
                        return Err(RuntimeError("length expects one argument".into()));
                    }
                    return match self.eval(&arguments[0])? {
                        Value::Array(xs) => Ok(Value::Number(xs.len() as f64)),
                        Value::Map(m) => Ok(Value::Number(m.len() as f64)),
                        Value::String(s) => Ok(Value::Number(s.chars().count() as f64)),
                        _ => Err(RuntimeError(
                            "length requires a string, array, or map".into(),
                        )),
                    };
                }
                let (params, body) = self
                    .functions
                    .get(name)
                    .cloned()
                    .ok_or_else(|| RuntimeError(format!("unknown function `{name}`")))?;
                let args = arguments
                    .iter()
                    .map(|a| self.eval(a))
                    .collect::<Result<Vec<_>, _>>()?;
                self.push();
                for (p, v) in params.into_iter().zip(args) {
                    self.define(p, v);
                }
                let r = self.exec(&Stmt::Block(body))?;
                self.pop();
                Ok(match r {
                    Flow::Continue => Value::Null,
                    Flow::Return(v) => v,
                })
            }
            Expr::Unary { op, expr, .. } => match (op, self.eval(expr)?) {
                (Op::Neg, Value::Number(n)) => Ok(Value::Number(-n)),
                _ => Err(RuntimeError("invalid unary operation".into())),
            },
            Expr::Binary {
                left, op, right, ..
            } => {
                let a = self.eval(left)?;
                let b = self.eval(right)?;
                match (op, a, b) {
                    (Op::Add, Value::Number(a), Value::Number(b)) => Ok(Value::Number(a + b)),
                    (Op::Add, Value::String(a), Value::String(b)) => Ok(Value::String(a + &b)),
                    (Op::Sub, Value::Number(a), Value::Number(b)) => Ok(Value::Number(a - b)),
                    (Op::Mul, Value::Number(a), Value::Number(b)) => Ok(Value::Number(a * b)),
                    (Op::Div, Value::Number(a), Value::Number(b)) => Ok(Value::Number(a / b)),
                    (Op::Eq, a, b) => Ok(Value::Boolean(a == b)),
                    (Op::And, Value::Boolean(a), Value::Boolean(b)) => Ok(Value::Boolean(a && b)),
                    (Op::Or, Value::Boolean(a), Value::Boolean(b)) => Ok(Value::Boolean(a || b)),
                    _ => Err(RuntimeError("invalid binary operation".into())),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{lexer::Lexer, parser::Parser};
    #[test]
    fn executes_function_program() {
        let src = "begin\nfunction add(a,b)\n return a+b\nend\nlet value be add(2,3)\nend";
        let ast = Parser::new(Lexer::new(src).tokenize().unwrap())
            .parse_program()
            .unwrap();
        assert!(Interpreter::new().execute(&ast).is_ok());
    }
}

#[cfg(test)]
mod collection_tests {
    use super::*;
    use crate::{lexer::Lexer, parser::Parser};
    #[test]
    fn executes_collection_program() {
        let src = "begin\nlet xs be [1,2,3]\nprint xs[1]\nprint length(xs)\nend";
        let ast = Parser::new(Lexer::new(src).tokenize().unwrap())
            .parse_program()
            .unwrap();
        assert!(Interpreter::new().execute(&ast).is_ok());
    }
}
