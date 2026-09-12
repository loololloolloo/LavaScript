use crate::parser::{CompareOp, Expr, LogicalOp, Op, Program, Stmt};

pub fn optimize(mut program: Program) -> Program { for f in &mut program.functions { optimize_statements(&mut f.body); } optimize_statements(&mut program.main); program }
fn optimize_statements(ss:&mut[Stmt]){for s in ss{match s{
 Stmt::Let(_,e)|Stmt::Const(_,e)|Stmt::Assign(_,e)|Stmt::Print(e)|Stmt::Return(e)=>*e=fold(e.clone()),
 Stmt::SetIndex(a,b,e)=>{*a=fold(a.clone());*b=fold(b.clone());*e=fold(e.clone())}, Stmt::SetProperty(a,_,e)=>{*a=fold(a.clone());*e=fold(e.clone())},
 Stmt::If{condition,then_body,else_body}=>{*condition=fold(condition.clone());optimize_statements(then_body);optimize_statements(else_body)},
 Stmt::While{condition,body}=>{*condition=fold(condition.clone());optimize_statements(body)}, Stmt::For{start,end,step,body,..}=>{*start=fold(start.clone());*end=fold(end.clone());*step=fold(step.clone());optimize_statements(body)},
 Stmt::Repeat{body,condition}=>{optimize_statements(body);*condition=fold(condition.clone())}, Stmt::Do{body}=>optimize_statements(body), Stmt::Break|Stmt::Continue=>{}
}}}
fn fold(e:Expr)->Expr{let e=match e{
 Expr::Binary(a,o,b)=>Expr::Binary(Box::new(fold(*a)),o,Box::new(fold(*b))),Expr::Compare(a,o,b)=>Expr::Compare(Box::new(fold(*a)),o,Box::new(fold(*b))),Expr::Logical(a,o,b)=>Expr::Logical(Box::new(fold(*a)),o,Box::new(fold(*b))),Expr::Not(a)=>Expr::Not(Box::new(fold(*a))),
 Expr::Call(n,a)=>Expr::Call(n,a.into_iter().map(fold).collect()),Expr::Array(a)=>Expr::Array(a.into_iter().map(fold).collect()),Expr::Table(a)=>Expr::Table(a.into_iter().map(|(k,v)|(k,fold(v))).collect()),Expr::Index(a,b)=>Expr::Index(Box::new(fold(*a)),Box::new(fold(*b))),Expr::Property(a,n)=>Expr::Property(Box::new(fold(*a)),n),Expr::Method(a,n,args)=>Expr::Method(Box::new(fold(*a)),n,args.into_iter().map(fold).collect()),o=>o};
 match e{Expr::Binary(a,o,b)=>match(*a,*b){(Expr::Number(x),Expr::Number(y))=>match o{Op::Add=>Expr::Number(x.wrapping_add(y)),Op::Sub=>Expr::Number(x.wrapping_sub(y)),Op::Mul=>Expr::Number(x.wrapping_mul(y)),Op::Div=>if y==0{Expr::Binary(Box::new(Expr::Number(x)),Op::Div,Box::new(Expr::Number(y)))}else{Expr::Number(x/y)},Op::Mod=>if y==0{Expr::Binary(Box::new(Expr::Number(x)),Op::Mod,Box::new(Expr::Number(y)))}else{Expr::Number(x%y)}},(x,y)=>Expr::Binary(Box::new(x),o,Box::new(y))},
 Expr::Compare(a,o,b)=>match(*a,*b){(Expr::Number(x),Expr::Number(y))=>Expr::Bool(match o{CompareOp::Eq=>x==y,CompareOp::Ne=>x!=y,CompareOp::Lt=>x<y,CompareOp::Le=>x<=y,CompareOp::Gt=>x>y,CompareOp::Ge=>x>=y}),(Expr::Bool(x),Expr::Bool(y))=>Expr::Bool(match o{CompareOp::Eq=>x==y,CompareOp::Ne=>x!=y,_=>false}),(Expr::String(x),Expr::String(y))=>Expr::Bool(match o{CompareOp::Eq=>x==y,CompareOp::Ne=>x!=y,_=>false}),(x,y)=>Expr::Compare(Box::new(x),o,Box::new(y))},
 Expr::Logical(a,o,b)=>match(*a,*b){(Expr::Bool(x),Expr::Bool(y))=>Expr::Bool(match o{LogicalOp::And=>x&&y,LogicalOp::Or=>x||y}),(x,y)=>Expr::Logical(Box::new(x),o,Box::new(y))},Expr::Not(a)=>match*a{Expr::Bool(v)=>Expr::Bool(!v),x=>Expr::Not(Box::new(x))},o=>o}}
