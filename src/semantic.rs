use std::collections::HashMap;
use crate::{ast::{Expr, Op, Stmt, Value}, lexer::Span};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VelaRiType { Number, Boolean, String, Null, Unknown }
#[derive(Debug, Clone, PartialEq)]
pub struct SemanticError { pub message: String, pub span: Span }
#[derive(Clone)] struct FunctionSig { parameters: usize }

pub struct SemanticAnalyzer { scopes: Vec<HashMap<String, VelaRiType>>, functions: HashMap<String, FunctionSig>, in_function: bool, return_seen: bool }
impl SemanticAnalyzer {
 pub fn new() -> Self { Self { scopes: vec![HashMap::new()], functions: HashMap::new(), in_function: false, return_seen: false } }
 pub fn analyze(&mut self, root: &Stmt) -> Result<(), Vec<SemanticError>> { let mut e=Vec::new(); self.collect_functions(root); self.stmt(root,&mut e); if e.is_empty(){Ok(())}else{Err(e)} }
 fn collect_functions(&mut self,s:&Stmt){match s{Stmt::Block(xs)=>for x in xs{self.collect_functions(x)},Stmt::Function{name,parameters,..}=>{self.functions.insert(name.clone(),FunctionSig{parameters:parameters.len()});},_=>{}}}
 fn push(&mut self){self.scopes.push(HashMap::new())} fn pop(&mut self){self.scopes.pop();}
 fn define(&mut self,n:String,t:VelaRiType){self.scopes.last_mut().unwrap().insert(n,t);}
 fn lookup(&self,n:&str)->Option<VelaRiType>{self.scopes.iter().rev().find_map(|s|s.get(n).copied())}
 fn stmt(&mut self,s:&Stmt,e:&mut Vec<SemanticError>){match s{
  Stmt::Block(xs)=>for x in xs{self.stmt(x,e)},
  Stmt::Let{name,value,span}=>{if let Ok(t)=self.expr(value,e){if self.scopes.last().unwrap().contains_key(name){e.push(SemanticError{message:format!("variable `{name}` is already declared in this scope"),span:*span})}else{self.define(name.clone(),t)}}},
  Stmt::Assign{name,value,span}=>{if self.lookup(name).is_none(){e.push(SemanticError{message:format!("variable `{name}` used before initialization"),span:*span})}let _=self.expr(value,e);},
  Stmt::Print(x)=>{let _=self.expr(x,e);},
  Stmt::Return{value,span}=>{if !self.in_function{e.push(SemanticError{message:"return is only valid inside a function".into(),span:*span})}if let Some(x)=value{let _=self.expr(x,e);}self.return_seen=true;},
  Stmt::Function{name,parameters,body,span}=>{self.push();for p in parameters{self.define(p.clone(),VelaRiType::Unknown);}let old=self.in_function;let old_return=self.return_seen;self.in_function=true;self.return_seen=false;for x in body{self.stmt(x,e);}if !self.return_seen{e.push(SemanticError{message:format!("function `{name}` has no return statement"),span:*span});}self.in_function=old;self.return_seen=old_return;self.pop();},
  Stmt::Conditional{condition,then_branch,otherwise_branch,..}=>{if self.expr(condition,e)!=Ok(VelaRiType::Boolean){e.push(SemanticError{message:"if condition must be Boolean".into(),span:condition.span()});}self.push();for x in then_branch{self.stmt(x,e);}self.pop();if let Some(xs)=otherwise_branch{self.push();for x in xs{self.stmt(x,e);}self.pop();}},
  Stmt::RepeatLoop{iterations,body,..}=>{if self.expr(iterations,e)!=Ok(VelaRiType::Number){e.push(SemanticError{message:"repeat count must be Number".into(),span:iterations.span()});}self.push();for x in body{self.stmt(x,e);}self.pop();},
  Stmt::CreateWindow{width,height,..}=>for x in [width,height]{if self.expr(x,e)!=Ok(VelaRiType::Number){e.push(SemanticError{message:"window dimensions must be Number".into(),span:x.span()});}},
 }}
 fn expr(&self,x:&Expr,e:&mut Vec<SemanticError>)->Result<VelaRiType,()>{match x{
  Expr::Literal(v,_)=>Ok(match v{Value::Number(_)=>VelaRiType::Number,Value::Boolean(_)=>VelaRiType::Boolean,Value::String(_)=>VelaRiType::String,Value::Null=>VelaRiType::Null}),
  Expr::Variable(n,sp)=>self.lookup(n).ok_or_else(||{e.push(SemanticError{message:format!("variable `{n}` used before initialization"),span:*sp});()}),
  Expr::Call{name,arguments,span}=>{match self.functions.get(name){Some(sig) if sig.parameters==arguments.len()=>{},Some(sig)=>e.push(SemanticError{message:format!("function `{name}` expects {} arguments, got {}",sig.parameters,arguments.len()),span:*span}),None=>e.push(SemanticError{message:format!("unknown function `{name}`"),span:*span})}for a in arguments{let _=self.expr(a,e);}Ok(VelaRiType::Unknown)},
  Expr::Unary{op,expr,span}=>{let t=self.expr(expr,e).map_err(|_|())?;if *op==Op::Neg&&t==VelaRiType::Number{Ok(t)}else{e.push(SemanticError{message:"unary minus requires Number".into(),span:*span});Err(())}},
  Expr::Binary{left,op,right,span}=>{let a=self.expr(left,e).map_err(|_|())?;let b=self.expr(right,e).map_err(|_|())?;match op{Op::Add|Op::Sub|Op::Mul|Op::Div if (a==VelaRiType::Number&&b==a)||(a==VelaRiType::Unknown||b==VelaRiType::Unknown)=>Ok(if a==VelaRiType::Unknown||b==VelaRiType::Unknown{VelaRiType::Unknown}else{a}),Op::Add if a==VelaRiType::String&&b==a=>Ok(a),Op::Eq if a==b||a==VelaRiType::Unknown||b==VelaRiType::Unknown=>Ok(VelaRiType::Boolean),Op::And|Op::Or if a==VelaRiType::Boolean&&b==a=>Ok(VelaRiType::Boolean),_=>{e.push(SemanticError{message:format!("invalid operation {op:?} for {a:?} and {b:?}"),span:*span});Err(())}}}}
 }}
