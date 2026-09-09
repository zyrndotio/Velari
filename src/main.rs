mod lexer;mod ast;mod parser;mod semantic;mod interpreter;
use std::{env,fs,path::Path};use lexer::Lexer;use parser::Parser;use semantic::SemanticAnalyzer;use interpreter::Interpreter;
fn compile(source:&str,run:bool)->Result<(),String>{
 let tokens=Lexer::new(source).tokenize().map_err(|es|es.into_iter().map(|e|format!("error at {}:{}: {}",e.span.line,e.span.column,e.message)).collect::<Vec<_>>().join("\n"))?;
 let ast=Parser::new(tokens).parse_program().map_err(|es|es.into_iter().map(|e|format!("error at {}:{}: {}",e.span.line,e.span.column,e.message)).collect::<Vec<_>>().join("\n"))?;
 let mut sem=SemanticAnalyzer::new();sem.analyze(&ast).map_err(|es|es.into_iter().map(|e|format!("error at {}:{}: {}",e.span.line,e.span.column,e.message)).collect::<Vec<_>>().join("\n"))?;
 if run { Interpreter::new().execute(&ast).map_err(|e|e.0)?; } else { println!("check succeeded"); } Ok(())
}
fn usage(){eprintln!("VelaRi 0.2\nusage: velari <check|run|new> [file|directory]");}
fn main(){let mut a=env::args().skip(1);match a.next().as_deref(){Some("version")=>println!("velari 0.2.0"),Some("check")|Some("run")=>{let run=env::args().nth(1).as_deref()==Some("run");let file=a.next().unwrap_or_else(||"main.vr".into());match fs::read_to_string(&file).map_err(|e|e.to_string()).and_then(|s|compile(&s,run)){Ok(())=>{},Err(e)=>{eprintln!("{e}");std::process::exit(1)}}},Some("new")=>{let dir=a.next().unwrap_or_else(||"velari-app".into());fs::create_dir_all(Path::new(&dir).join("src")).expect("create project");fs::write(Path::new(&dir).join("src/main.vr"),"begin\n print \"hello, VelaRi\"\nend\n").expect("write project");fs::write(Path::new(&dir).join("vela.toml"),"[package]\nname = \"velari-app\"\nversion = \"0.1.0\"\nentry = \"src/main.vr\"\n").expect("write manifest");println!("created {dir}")},_=>usage()}}
