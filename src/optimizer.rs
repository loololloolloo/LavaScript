use crate::parser::{CompareOp, Expr, LogicalOp, Op, Program, Stmt};

pub fn optimize(mut program: Program) -> Program {
    for function in &mut program.functions { optimize_statements(&mut function.body); }
    optimize_statements(&mut program.main);
    program
}

fn optimize_statements(statements: &mut [Stmt]) {
    for stmt in statements {
        match stmt {
            Stmt::Let(_, e) | Stmt::Const(_, e) | Stmt::Assign(_, e) | Stmt::Print(e) | Stmt::Return(e) => *e = fold(e.clone()),
            Stmt::If { condition, then_body, else_body } => { *condition = fold(condition.clone()); optimize_statements(then_body); optimize_statements(else_body); }
            Stmt::While { condition, body } => { *condition = fold(condition.clone()); optimize_statements(body); }
            Stmt::For { start, end, step, body, .. } => { *start = fold(start.clone()); *end = fold(end.clone()); *step = fold(step.clone()); optimize_statements(body); }
            Stmt::Repeat { body, condition } => { optimize_statements(body); *condition = fold(condition.clone()); }
            Stmt::Do { body } => optimize_statements(body),
            Stmt::Break | Stmt::Continue => {}
        }
    }
}

fn fold(expr: Expr) -> Expr {
    let expr = match expr {
        Expr::Binary(a, op, b) => Expr::Binary(Box::new(fold(*a)), op, Box::new(fold(*b))),
        Expr::Compare(a, op, b) => Expr::Compare(Box::new(fold(*a)), op, Box::new(fold(*b))),
        Expr::Logical(a, op, b) => Expr::Logical(Box::new(fold(*a)), op, Box::new(fold(*b))),
        Expr::Not(a) => Expr::Not(Box::new(fold(*a))),
        Expr::Call(name, args) => Expr::Call(name, args.into_iter().map(fold).collect()),
        other => other,
    };
    match expr {
        Expr::Binary(a, op, b) => match (*a, *b) {
            (Expr::Number(x), Expr::Number(y)) => match op { Op::Add=>Expr::Number(x.wrapping_add(y)), Op::Sub=>Expr::Number(x.wrapping_sub(y)), Op::Mul=>Expr::Number(x.wrapping_mul(y)), Op::Div=>if y==0{Expr::Binary(Box::new(Expr::Number(x)),Op::Div,Box::new(Expr::Number(y)))}else{Expr::Number(x/y)}, Op::Mod=>if y==0{Expr::Binary(Box::new(Expr::Number(x)),Op::Mod,Box::new(Expr::Number(y)))}else{Expr::Number(x%y)} },
            (x, y) => Expr::Binary(Box::new(x), op, Box::new(y)),
        },
        Expr::Compare(a, op, b) => match (*a, *b) {
            (Expr::Number(x), Expr::Number(y)) => Expr::Bool(match op { CompareOp::Eq=>x==y, CompareOp::Ne=>x!=y, CompareOp::Lt=>x<y, CompareOp::Le=>x<=y, CompareOp::Gt=>x>y, CompareOp::Ge=>x>=y }),
            (Expr::Bool(x), Expr::Bool(y)) => Expr::Bool(match op { CompareOp::Eq=>x==y, CompareOp::Ne=>x!=y, _=>false }),
            (x, y) => Expr::Compare(Box::new(x), op, Box::new(y)),
        },
        Expr::Logical(a, op, b) => match (*a, *b) {
            (Expr::Bool(x), Expr::Bool(y)) => Expr::Bool(match op { LogicalOp::And=>x&&y, LogicalOp::Or=>x||y }),
            (x, y) => Expr::Logical(Box::new(x), op, Box::new(y)),
        },
        Expr::Not(a) => match *a { Expr::Bool(v)=>Expr::Bool(!v), x=>Expr::Not(Box::new(x)) },
        other => other,
    }
}
