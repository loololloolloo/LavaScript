use std::collections::{HashMap, HashSet};
use crate::parser::{CompareOp, Expr, LogicalOp, Op, Program, Stmt};
use crate::value::ValueType;

pub type Type = ValueType;

pub fn check(program: &Program) -> Result<(), String> {
    let mut signatures = HashMap::new();
    for f in &program.functions {
        if signatures.insert(f.name.clone(), (f.params.len(), Type::Number)).is_some() {
            return Err(format!("function `{}` is already declared", f.name));
        }
    }
    for f in &program.functions {
        let mut env = HashMap::new();
        for p in &f.params { env.insert(p.clone(), Type::Number); }
        check_stmts(&f.body, &mut env, &mut HashSet::new(), true, &signatures)?;
    }
    let mut env = HashMap::new();
    check_stmts(&program.main, &mut env, &mut HashSet::new(), false, &signatures)
}

fn check_stmts(stmts: &[Stmt], env: &mut HashMap<String, Type>, consts: &mut HashSet<String>, in_function: bool, signatures: &HashMap<String,(usize,Type)>) -> Result<(), String> {
    for s in stmts { match s {
        Stmt::Let(n,e) => { if env.contains_key(n) { return Err(format!("variable `{n}` is already declared")); } env.insert(n.clone(), expr_type(e, env, signatures)?); }
        Stmt::Const(n,e) => { if env.contains_key(n) { return Err(format!("variable `{n}` is already declared")); } env.insert(n.clone(), expr_type(e, env, signatures)?); consts.insert(n.clone()); }
        Stmt::Assign(n,e) => { if consts.contains(n) { return Err(format!("cannot assign to constant `{n}`")); } let old=*env.get(n).ok_or_else(||format!("undefined variable `{n}`"))?; let new=expr_type(e,env,signatures)?; if old!=Type::Unknown && new!=Type::Unknown && old!=new { return Err(format!("type mismatch assigning `{n}`: expected {}, got {}",old.name(),new.name())); } }
        Stmt::Print(e) => { expr_type(e,env,signatures)?; }
        Stmt::Return(e) => { if !in_function{return Err("`return` is only valid inside a function".into());} expr_type(e,env,signatures)?; }
        Stmt::If{condition,then_body,else_body} => { require_bool(condition,env,signatures,"if")?; let mut a=env.clone();let mut ac=consts.clone();check_stmts(then_body,&mut a,&mut ac,in_function,signatures)?;let mut b=env.clone();let mut bc=consts.clone();check_stmts(else_body,&mut b,&mut bc,in_function,signatures)?; }
        Stmt::While{condition,body} => { require_bool(condition,env,signatures,"while")?; let mut a=env.clone();let mut ac=consts.clone();check_stmts(body,&mut a,&mut ac,in_function,signatures)?; }
        Stmt::For{name,start,end,step,body} => { require_number(start,env,signatures,"for start")?;require_number(end,env,signatures,"for end")?;require_number(step,env,signatures,"for step")?;env.insert(name.clone(),Type::Number);let mut a=env.clone();let mut ac=consts.clone();check_stmts(body,&mut a,&mut ac,in_function,signatures)?; }
        Stmt::Repeat{body,condition} => { let mut a=env.clone();let mut ac=consts.clone();check_stmts(body,&mut a,&mut ac,in_function,signatures)?;require_bool(condition,&a,signatures,"repeat condition")?; }
        Stmt::Do{body} => { let mut a=env.clone();let mut ac=consts.clone();check_stmts(body,&mut a,&mut ac,in_function,signatures)?; }
        Stmt::Break|Stmt::Continue => {}
    }} Ok(())
}

fn expr_type(e:&Expr,env:&HashMap<String,Type>,signatures:&HashMap<String,(usize,Type)>)->Result<Type,String>{match e {
    Expr::Number(_)=>Ok(Type::Number),Expr::Bool(_)=>Ok(Type::Bool),Expr::String(_)=>Ok(Type::String),
    Expr::Variable(n)=>env.get(n).copied().ok_or_else(||format!("undefined variable `{n}`")),
    Expr::Binary(a,op,b)=>{require_number(a,env,signatures,"left operand")?;require_number(b,env,signatures,"right operand")?;match op{Op::Add|Op::Sub|Op::Mul|Op::Div|Op::Mod=>Ok(Type::Number)}},
    Expr::Compare(a,op,b)=>{let x=expr_type(a,env,signatures)?;let y=expr_type(b,env,signatures)?;if x!=y{return Err("comparison operands must have the same type".into())}if !matches!(op,CompareOp::Eq|CompareOp::Ne)&&x!=Type::Number{return Err("ordering comparisons require numbers".into())}Ok(Type::Bool)},
    Expr::Logical(a,_,b)=>{require_bool(a,env,signatures,"logical operand")?;require_bool(b,env,signatures,"logical operand")?;Ok(Type::Bool)},
    Expr::Not(a)=>{require_bool(a,env,signatures,"not operand")?;Ok(Type::Bool)},
    Expr::Call(name,args)=>{let (arity,result)=signatures.get(name).copied().ok_or_else(||format!("undefined function `{name}`"))?;if args.len()!=arity{return Err(format!("function `{name}` expects {arity} argument(s), got {}",args.len()));}for arg in args{if expr_type(arg,env,signatures)?!=Type::Number{return Err(format!("function `{name}` currently accepts only number arguments"));}}Ok(result)}
}}
fn require_number(e:&Expr,env:&HashMap<String,Type>,s:&HashMap<String,(usize,Type)>,w:&str)->Result<(),String>{if expr_type(e,env,s)?!=Type::Number{return Err(format!("{w} must be a number"))}Ok(())}
fn require_bool(e:&Expr,env:&HashMap<String,Type>,s:&HashMap<String,(usize,Type)>,w:&str)->Result<(),String>{if expr_type(e,env,s)?!=Type::Bool{return Err(format!("{w} must be a boolean"))}Ok(())}
