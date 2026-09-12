use crate::{parser::{CompareOp, Expr, LogicalOp, Op, Program, Stmt}, stdlib};
use crate::value::ValueType;
use std::collections::{HashMap,HashSet};
pub type Type=ValueType;
type Sig=(Vec<Type>,Type);

pub fn check(p:&Program)->Result<(),String>{let mut sigs=HashMap::new();for f in &p.functions{if sigs.insert(f.name.clone(),(vec![Type::Number;f.params.len()],Type::Number)).is_some(){return Err(format!("function `{}` is already declared",f.name));}}for (n,s) in stdlib::signatures(){sigs.insert(n.to_string(),(s.params,s.result));}for f in &p.functions{let mut e=HashMap::new();for x in &f.params{e.insert(x.clone(),Type::Number);}check_stmts(&f.body,&mut e,&mut HashSet::new(),true,&sigs)?;}let mut e=HashMap::new();check_stmts(&p.main,&mut e,&mut HashSet::new(),false,&sigs)}
fn check_stmts(ss:&[Stmt],e:&mut HashMap<String,Type>,c:&mut HashSet<String>,inf:bool,s:&HashMap<String,Sig>)->Result<(),String>{for x in ss{match x{
 Stmt::Let(n,v)|Stmt::Const(n,v)=>{if e.contains_key(n){return Err(format!("variable `{n}` is already declared"))}let t=expr_type(v,e,s)?;e.insert(n.clone(),t);if matches!(x,Stmt::Const(..)){c.insert(n.clone());}},
 Stmt::Assign(n,v)=>{if c.contains(n){return Err(format!("cannot assign to constant `{n}`"))}let old=*e.get(n).ok_or_else(||format!("undefined variable `{n}`"))?;let new=expr_type(v,e,s)?;compatible(old,new)?;},
 Stmt::SetIndex(a,i,v)=>{let t=expr_type(a,e,s)?;if t!=Type::Array{return Err("indexed assignment requires an array".into())}require_number(i,e,s,"array index")?;expr_type(v,e,s)?;},
 Stmt::SetProperty(a,_,v)=>{let t=expr_type(a,e,s)?;if t!=Type::Table{return Err("property assignment requires a table".into())}expr_type(v,e,s)?;},
 Stmt::Print(v)=>{expr_type(v,e,s)?},Stmt::Return(v)=>{if !inf{return Err("`return` is only valid inside a function".into())}expr_type(v,e,s)?},
 Stmt::If{condition,then_body,else_body}=>{require_bool(condition,e,s,"if")?;let mut a=e.clone();let mut ac=c.clone();check_stmts(then_body,&mut a,&mut ac,inf,s)?;let mut b=e.clone();let mut bc=c.clone();check_stmts(else_body,&mut b,&mut bc,inf,s)?},
 Stmt::While{condition,body}=>{require_bool(condition,e,s,"while")?;let mut a=e.clone();let mut ac=c.clone();check_stmts(body,&mut a,&mut ac,inf,s)?},
 Stmt::For{name,start,end,step,body}=>{require_number(start,e,s,"for start")?;require_number(end,e,s,"for end")?;require_number(step,e,s,"for step")?;e.insert(name.clone(),Type::Number);check_stmts(body,e,c,inf,s)?},
 Stmt::Repeat{body,condition}=>{check_stmts(body,e,c,inf,s)?;require_bool(condition,e,s,"repeat condition")?},Stmt::Do{body}=>check_stmts(body,e,c,inf,s)?,Stmt::Break|Stmt::Continue=>{}
}}Ok(())}
fn compatible(a:Type,b:Type)->Result<(),String>{if a!=Type::Unknown&&b!=Type::Unknown&&a!=b{Err(format!("type mismatch: expected {}, got {}",a.name(),b.name()))}else{Ok(())}}
fn expr_type(x:&Expr,e:&HashMap<String,Type>,s:&HashMap<String,Sig>)->Result<Type,String>{match x{
 Expr::Number(_)=>Ok(Type::Number),Expr::Bool(_)=>Ok(Type::Bool),Expr::String(_)=>Ok(Type::String),Expr::Nil=>Ok(Type::Nil),Expr::Variable(n)=>e.get(n).copied().ok_or_else(||format!("undefined variable `{n}`")),
 Expr::Array(v)=>{for x in v{expr_type(x,e,s)?;}Ok(Type::Array)},Expr::Table(v)=>{for(_,x)in v{expr_type(x,e,s)?;}Ok(Type::Table)},
 Expr::Binary(a,_,b)=>{require_number(a,e,s,"left operand")?;require_number(b,e,s,"right operand")?;Ok(Type::Number)},
 Expr::Compare(a,o,b)=>{let x=expr_type(a,e,s)?;let y=expr_type(b,e,s)?;if x!=y{return Err("comparison operands must have the same type".into())}if !matches!(o,CompareOp::Eq|CompareOp::Ne)&&x!=Type::Number{return Err("ordering comparisons require numbers".into())}Ok(Type::Bool)},
 Expr::Logical(a,_,b)=>{require_bool(a,e,s,"logical operand")?;require_bool(b,e,s,"logical operand")?;Ok(Type::Bool)},Expr::Not(a)=>{require_bool(a,e,s,"not operand")?;Ok(Type::Bool)},
 Expr::Call(n,a)=>call_type(n,a,e,s),Expr::Index(a,i)=>{let t=expr_type(a,e,s)?;require_number(i,e,s,"array index")?;match t{Type::Array=>Ok(Type::Unknown),Type::Table=>Ok(Type::Unknown),_=>(Err("indexing requires an array or table".into()))}},
 Expr::Property(a,_)=>{if expr_type(a,e,s)?!=Type::Table{return Err("property access requires a table".into())}Ok(Type::Unknown)},
 Expr::Method(a,n,args)=>{let t=expr_type(a,e,s)?;for x in args{expr_type(x,e,s)?;}match(t,n.as_str()){(Type::Array,"push")|(Type::Array,"pop")|(Type::Array,"contains")|(Type::String,"contains")|(Type::String,"upper")|(Type::String,"lower")|(Type::Table,"get")|(Type::Table,"set")=>Ok(if n=="contains"{Type::Bool}else{Type::Unknown}),_=>Err(format!("unknown method `{n}` for {}",t.name()))}}
}}
fn call_type(n:&str,a:&[Expr],e:&HashMap<String,Type>,s:&HashMap<String,Sig>)->Result<Type,String>{let(sig,result)=s.get(n).ok_or_else(||format!("undefined function `{n}`"))?;if a.len()!=sig.len(){return Err(format!("function `{n}` expects {} argument(s), got {}",sig.len(),a.len()))}for(i,x)in a.iter().enumerate(){let t=expr_type(x,e,s)?;if sig[i]!=Type::Unknown&&t!=sig[i]{return Err(format!("argument {} to `{n}` must be {}",i+1,sig[i].name()))}}Ok(*result)}
fn require_number(x:&Expr,e:&HashMap<String,Type>,s:&HashMap<String,Sig>,w:&str)->Result<(),String>{if expr_type(x,e,s)?!=Type::Number{Err(format!("{w} must be a number"))}else{Ok(())}}
fn require_bool(x:&Expr,e:&HashMap<String,Type>,s:&HashMap<String,Sig>,w:&str)->Result<(),String>{if expr_type(x,e,s)?!=Type::Bool{Err(format!("{w} must be a boolean"))}else{Ok(())}}
