use std::collections::{HashMap, HashSet};
use crate::parser::{CompareOp, Expr, LogicalOp, Op, Program, Stmt};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type { Number, Bool, String, Unknown }

pub fn check(program: &Program) -> Result<(), String> {
    for f in &program.functions {
        let mut env = HashMap::new();
        for p in &f.params { env.insert(p.clone(), Type::Number); }
        check_stmts(&f.body, &mut env, &mut HashSet::new(), true)?;
    }
    let mut env = HashMap::new();
    check_stmts(&program.main, &mut env, &mut HashSet::new(), false)
}

fn check_stmts(stmts: &[Stmt], env: &mut HashMap<String, Type>, consts: &mut HashSet<String>, in_function: bool) -> Result<(), String> {
    for s in stmts { match s {
        Stmt::Let(n,e) => { if env.contains_key(n) { return Err(format!("variable `{n}` is already declared")); } env.insert(n.clone(), expr_type(e, env)?); }
        Stmt::Const(n,e) => { if env.contains_key(n) { return Err(format!("variable `{n}` is already declared")); } env.insert(n.clone(), expr_type(e, env)?); consts.insert(n.clone()); }
        Stmt::Assign(n,e) => { if consts.contains(n) { return Err(format!("cannot assign to constant `{n}`")); } let old=*env.get(n).ok_or_else(||format!("undefined variable `{n}`"))?; let new=expr_type(e,env)?; if old!=Type::Unknown && new!=Type::Unknown && old!=new { return Err(format!("type mismatch assigning `{n}`: expected {:?}, got {:?}",old,new)); } }
        Stmt::Print(e) => { expr_type(e,env)?; }
        Stmt::Return(e) => { if !in_function{return Err("`return` is only valid inside a function".into());} if expr_type(e,env)?==Type::Unknown{return Err("cannot infer return type".into());} }
        Stmt::If{condition,then_body,else_body} => { require_bool(condition,env,"if")?; let mut a=env.clone();let mut ac=consts.clone();check_stmts(then_body,&mut a,&mut ac,in_function)?;let mut b=env.clone();let mut bc=consts.clone();check_stmts(else_body,&mut b,&mut bc,in_function)?; }
        Stmt::While{condition,body} => { require_bool(condition,env,"while")?; let mut a=env.clone();let mut ac=consts.clone();check_stmts(body,&mut a,&mut ac,in_function)?; }
        Stmt::For{name,start,end,step,body} => { require_number(start,env,"for start")?;require_number(end,env,"for end")?;require_number(step,env,"for step")?;env.insert(name.clone(),Type::Number);let mut a=env.clone();let mut ac=consts.clone();check_stmts(body,&mut a,&mut ac,in_function)?; }
        Stmt::Repeat{body,condition} => { let mut a=env.clone();let mut ac=consts.clone();check_stmts(body,&mut a,&mut ac,in_function)?;require_bool(condition,&a,"repeat condition")?; }
        Stmt::Do{body} => { let mut a=env.clone();let mut ac=consts.clone();check_stmts(body,&mut a,&mut ac,in_function)?; }
        Stmt::Break|Stmt::Continue => {}
    }} Ok(())
}
fn expr_type(e:&Expr,env:&HashMap<String,Type>)->Result<Type,String>{match e {
    Expr::Number(_)=>Ok(Type::Number),Expr::Bool(_)=>Ok(Type::Bool),Expr::String(_)=>Ok(Type::String),
    Expr::Variable(n)=>env.get(n).copied().ok_or_else(||format!("undefined variable `{n}`")),
    Expr::Binary(a,op,b)=>{require_number(a,env,"left operand")?;require_number(b,env,"right operand")?;match op{Op::Add|Op::Sub|Op::Mul|Op::Div|Op::Mod=>Ok(Type::Number)}},
    Expr::Compare(a,op,b)=>{let x=expr_type(a,env)?;let y=expr_type(b,env)?;if x!=y{return Err("comparison operands must have the same type".into())}if !matches!(op,CompareOp::Eq|CompareOp::Ne)&&x!=Type::Number{return Err("ordering comparisons require numbers".into())}Ok(Type::Bool)},
    Expr::Logical(a,_,b)=>{require_bool(a,env,"logical operand")?;require_bool(b,env,"logical operand")?;Ok(Type::Bool)},
    Expr::Not(a)=>{require_bool(a,env,"not operand")?;Ok(Type::Bool)},
    Expr::Call(_,_)=>Ok(Type::Number),
}}
fn require_number(e:&Expr,env:&HashMap<String,Type>,where_:&str)->Result<(),String>{if expr_type(e,env)?!=Type::Number{return Err(format!("{where_} must be a number"))}Ok(())}
fn require_bool(e:&Expr,env:&HashMap<String,Type>,where_:&str)->Result<(),String>{if expr_type(e,env)?!=Type::Bool{return Err(format!("{where_} must be a boolean"))}Ok(())}
