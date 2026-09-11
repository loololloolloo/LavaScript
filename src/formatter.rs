use std::fmt::Write;
use crate::parser::{Expr, LogicalOp, Op, Program, Stmt, CompareOp};

pub fn format_program(program: &Program) -> String {
    let mut out = String::new();
    for f in &program.functions {
        writeln!(&mut out, "function {}({})", f.name, f.params.join(", ")).unwrap();
        write_stmts(&mut out, &f.body, 1);
        writeln!(&mut out, "end").unwrap();
        writeln!(&mut out).unwrap();
    }
    write_stmts(&mut out, &program.main, 0);
    while out.ends_with("\n\n") { out.pop(); }
    out
}

fn write_stmts(out: &mut String, stmts: &[Stmt], depth: usize) {
    for s in stmts {
        let pad = "    ".repeat(depth);
        match s {
            Stmt::Let(n,e) => writeln!(out, "{pad}let {n} = {}", expr(e)).unwrap(),
            Stmt::Const(n,e) => writeln!(out, "{pad}const {n} = {}", expr(e)).unwrap(),
            Stmt::Assign(n,e) => writeln!(out, "{pad}{n} = {}", expr(e)).unwrap(),
            Stmt::Print(e) => writeln!(out, "{pad}print {}", expr(e)).unwrap(),
            Stmt::Return(e) => writeln!(out, "{pad}return {}", expr(e)).unwrap(),
            Stmt::Break => writeln!(out, "{pad}break").unwrap(),
            Stmt::Continue => writeln!(out, "{pad}continue").unwrap(),
            Stmt::If{condition,then_body,else_body} => { writeln!(out, "{pad}if {} then",expr(condition)).unwrap(); write_stmts(out,then_body,depth+1); if !else_body.is_empty(){writeln!(out,"{pad}else").unwrap();write_stmts(out,else_body,depth+1);} writeln!(out,"{pad}end").unwrap(); },
            Stmt::While{condition,body} => { writeln!(out,"{pad}while {}",expr(condition)).unwrap();write_stmts(out,body,depth+1);writeln!(out,"{pad}end").unwrap(); },
            Stmt::For{name,start,end,step,body} => { writeln!(out,"{pad}for {name} = {}, {}, {}",expr(start),expr(end),expr(step)).unwrap();write_stmts(out,body,depth+1);writeln!(out,"{pad}end").unwrap(); },
            Stmt::Repeat{body,condition} => { writeln!(out,"{pad}repeat").unwrap();write_stmts(out,body,depth+1);writeln!(out,"{pad}until {}",expr(condition)).unwrap(); },
            Stmt::Do{body} => { writeln!(out,"{pad}do").unwrap();write_stmts(out,body,depth+1);writeln!(out,"{pad}end").unwrap(); },
        }
    }
}

fn expr(e: &Expr) -> String { match e {
    Expr::Number(n)=>n.to_string(), Expr::Bool(v)=>v.to_string(), Expr::String(s)=>format!("\"{}\"",s.replace('\\','\\\\').replace('"','\\"').replace('\n','\\n')),
    Expr::Variable(n)=>n.clone(), Expr::Binary(a,o,b)=>format!("({} {} {})",expr(a),op(o),expr(b)),
    Expr::Compare(a,o,b)=>format!("({} {} {})",expr(a),cmp(o),expr(b)), Expr::Logical(a,o,b)=>format!("({} {} {})",expr(a),if matches!(o,LogicalOp::And){"and"}else{"or"},expr(b)),
    Expr::Not(a)=>format!("not {}",expr(a)), Expr::Call(n,args)=>format!("{}({})",n,args.iter().map(expr).collect::<Vec<_>>().join(", ")),
}}
fn op(o:&Op)->&'static str{match o{Op::Add=>"+",Op::Sub=>"-",Op::Mul=>"*",Op::Div=>"/",Op::Mod=>"%"}}
fn cmp(o:&CompareOp)->&'static str{match o{CompareOp::Eq=>"==",CompareOp::Ne=>"!=",CompareOp::Lt=>"<",CompareOp::Le=>"<=",CompareOp::Gt=>">",CompareOp::Ge=>">="}}
