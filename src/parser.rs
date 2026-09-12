use crate::lexer::Token;

#[derive(Debug, Clone)]
pub enum Expr {
    Number(i32), Bool(bool), String(String), Nil, Variable(String),
    Array(Vec<Expr>), Table(Vec<(String, Expr)>),
    Binary(Box<Expr>, Op, Box<Expr>), Compare(Box<Expr>, CompareOp, Box<Expr>),
    Logical(Box<Expr>, LogicalOp, Box<Expr>), Not(Box<Expr>),
    Call(String, Vec<Expr>), Index(Box<Expr>, Box<Expr>), Property(Box<Expr>, String),
    Method(Box<Expr>, String, Vec<Expr>),
}
#[derive(Debug, Clone)] pub enum Op { Add, Sub, Mul, Div, Mod }
#[derive(Debug, Clone)] pub enum CompareOp { Eq, Ne, Lt, Le, Gt, Ge }
#[derive(Debug, Clone)] pub enum LogicalOp { And, Or }
#[derive(Debug, Clone)] pub enum Stmt {
    Let(String, Expr), Const(String, Expr), Assign(String, Expr), SetIndex(Expr, Expr, Expr), SetProperty(Expr, String, Expr),
    Print(Expr), Return(Expr), Break, Continue,
    If { condition: Expr, then_body: Vec<Stmt>, else_body: Vec<Stmt> }, While { condition: Expr, body: Vec<Stmt> },
    For { name: String, start: Expr, end: Expr, step: Expr, body: Vec<Stmt> }, Repeat { body: Vec<Stmt>, condition: Expr }, Do { body: Vec<Stmt> },
}
#[derive(Debug, Clone)] pub struct FunctionDecl { pub name: String, pub params: Vec<String>, pub body: Vec<Stmt> }
#[derive(Debug, Clone)] pub struct Program { pub functions: Vec<FunctionDecl>, pub main: Vec<Stmt> }

pub fn parse(tokens: &[Token]) -> Result<Program, String> {
    let mut p=Parser{tokens,pos:0}; p.skip(); let mut functions=Vec::new(); let mut main=Vec::new();
    while !p.at(&Token::Eof){p.skip();if p.at(&Token::Eof){break}if p.at(&Token::Function){functions.push(p.parse_function()?)}else{main.push(p.statement(false)?)}p.skip()}
    Ok(Program{functions,main})
}
struct Parser<'a>{tokens:&'a[Token],pos:usize}
impl<'a> Parser<'a>{
 fn at(&self,t:&Token)->bool{self.tokens.get(self.pos)==Some(t)}
 fn skip(&mut self){while self.at(&Token::Newline)||self.at(&Token::Semicolon){self.pos+=1}}
 fn line_end(&mut self,m:&str)->Result<(),String>{if self.at(&Token::Newline)||self.at(&Token::Semicolon)||self.at(&Token::End)||self.at(&Token::Else)||self.at(&Token::ElseIf)||self.at(&Token::Eof){self.skip();Ok(())}else{Err(m.into())}}
 fn identifier(&mut self)->Result<String,String>{match self.tokens.get(self.pos){Some(Token::Identifier(s))=>{let n=s.clone();self.pos+=1;Ok(n)},_=>(Err("expected identifier".into()))}}
 fn expect(&mut self,t:Token,m:&str)->Result<(),String>{if self.at(&t){self.pos+=1;Ok(())}else{Err(m.into())}}
 fn statement(&mut self,inf:bool)->Result<Stmt,String>{match self.tokens.get(self.pos){
  Some(Token::Let)|Some(Token::Local)=>{self.pos+=1;let n=self.identifier()?;self.expect(Token::Equal,"expected `=`")?;Ok(Stmt::Let(n,self.expr()?))}
  Some(Token::Const)=>{self.pos+=1;let n=self.identifier()?;self.expect(Token::Equal,"expected `=`")?;Ok(Stmt::Const(n,self.expr()?))}
  Some(Token::Identifier(_))=>{let n=self.identifier()?;if self.at(&Token::Equal){self.pos+=1;Ok(Stmt::Assign(n,self.expr()?))}else{let mut target=Expr::Variable(n);loop{if self.at(&Token::LeftBracket){self.pos+=1;let i=self.expr()?;self.expect(Token::RightBracket,"expected `]`")?;target=Expr::Index(Box::new(target),Box::new(i))}else if self.at(&Token::Dot){self.pos+=1;let k=self.identifier()?;target=Expr::Property(Box::new(target),k)}else{break}}self.expect(Token::Equal,"expected `=` after assignment target")?;let v=self.expr()?;match target{Expr::Index(a,b)=>Ok(Stmt::SetIndex(*a,*b,v)),Expr::Property(a,k)=>Ok(Stmt::SetProperty(*a,k,v)),_=>(Err("invalid assignment target".into()))}}}
  Some(Token::Print)=>{self.pos+=1;Ok(Stmt::Print(self.expr()?))}
  Some(Token::Return)=>{if !inf{return Err("`return` is only valid inside a function".into())}self.pos+=1;Ok(Stmt::Return(self.expr()?))}
  Some(Token::If)=>self.parse_if(inf),Some(Token::While)=>self.parse_while(inf),Some(Token::For)=>self.parse_for(inf),Some(Token::Repeat)=>self.parse_repeat(inf),Some(Token::Do)=>self.parse_do(inf),
  Some(Token::Break)=>{self.pos+=1;Ok(Stmt::Break)},Some(Token::Continue)=>{self.pos+=1;Ok(Stmt::Continue)},_=>(Err("expected statement".into()))}}
 fn parse_function(&mut self)->Result<FunctionDecl,String>{self.pos+=1;let name=self.identifier()?;self.expect(Token::LeftParen,"expected `(`")?;let mut ps=Vec::new();if !self.at(&Token::RightParen){loop{let p=self.identifier()?;if ps.contains(&p){return Err(format!("duplicate parameter `{p}`"))}ps.push(p);if self.at(&Token::Comma){self.pos+=1}else{break}}}self.expect(Token::RightParen,"expected `)`")?;self.skip();let body=self.block(&[Token::End],true)?;self.expect(Token::End,"expected `end`")?;if !body.iter().any(|s|matches!(s,Stmt::Return(_))){return Err(format!("function `{name}` must contain a return"))}Ok(FunctionDecl{name,params:ps,body})}
 fn block(&mut self,stops:&[Token],inf:bool)->Result<Vec<Stmt>,String>{let mut o=Vec::new();loop{self.skip();if self.at(&Token::Eof)||stops.iter().any(|t|self.at(t)){break}if self.at(&Token::Function){return Err("nested functions are not supported".into())}o.push(self.statement(inf)?)}Ok(o)}
 fn parse_if(&mut self,inf:bool)->Result<Stmt,String>{self.pos+=1;let c=self.expr()?;if self.at(&Token::Then){self.pos+=1}self.line_end("expected newline after `if`")?;let t=self.block(&[Token::Else,Token::ElseIf,Token::End],inf)?;let e=if self.at(&Token::ElseIf){self.pos+=1;vec![self.parse_if_after(inf)?]}else if self.at(&Token::Else){self.pos+=1;self.line_end("expected newline after `else`")?;self.block(&[Token::End],inf)?}else{Vec::new()};self.expect(Token::End,"expected `end`")?;Ok(Stmt::If{condition:c,then_body:t,else_body:e})}
 fn parse_if_after(&mut self,inf:bool)->Result<Stmt,String>{let c=self.expr()?;if self.at(&Token::Then){self.pos+=1}self.line_end("expected newline")?;let t=self.block(&[Token::Else,Token::ElseIf,Token::End],inf)?;let e=if self.at(&Token::ElseIf){self.pos+=1;vec![self.parse_if_after(inf)?]}else if self.at(&Token::Else){self.pos+=1;self.line_end("expected newline")?;self.block(&[Token::End],inf)?}else{Vec::new()};Ok(Stmt::If{condition:c,then_body:t,else_body:e})}
 fn parse_while(&mut self,inf:bool)->Result<Stmt,String>{self.pos+=1;let c=self.expr()?;if self.at(&Token::Then){self.pos+=1}self.line_end("expected newline")?;let b=self.block(&[Token::End],inf)?;self.expect(Token::End,"expected `end`")?;Ok(Stmt::While{condition:c,body:b})}
 fn parse_for(&mut self,inf:bool)->Result<Stmt,String>{self.pos+=1;let n=self.identifier()?;self.expect(Token::Equal,"expected `=`")?;let s=self.expr()?;self.expect(Token::Comma,"expected `,`")?;let e=self.expr()?;let st=if self.at(&Token::Comma){self.pos+=1;self.expr()?}else{Expr::Number(1)};self.line_end("expected newline")?;let b=self.block(&[Token::End],inf)?;self.expect(Token::End,"expected `end`")?;Ok(Stmt::For{name:n,start:s,end:e,step:st,body:b})}
 fn parse_repeat(&mut self,inf:bool)->Result<Stmt,String>{self.pos+=1;self.line_end("expected newline")?;let b=self.block(&[Token::Until],inf)?;self.expect(Token::Until,"expected `until`")?;Ok(Stmt::Repeat{body:b,condition:self.expr()?})}
 fn parse_do(&mut self,inf:bool)->Result<Stmt,String>{self.pos+=1;self.line_end("expected newline")?;let b=self.block(&[Token::End],inf)?;self.expect(Token::End,"expected `end`")?;Ok(Stmt::Do{body:b})}
 fn expr(&mut self)->Result<Expr,String>{self.logic_or()}
 fn logic_or(&mut self)->Result<Expr,String>{let mut l=self.logic_and()?;loop{let o=match self.tokens.get(self.pos){Some(Token::Or)|Some(Token::OrOr)=>LogicalOp::Or,_=>break};self.pos+=1;l=Expr::Logical(Box::new(l),o,Box::new(self.logic_and()?))}Ok(l)}
 fn logic_and(&mut self)->Result<Expr,String>{let mut l=self.compare()?;loop{let o=match self.tokens.get(self.pos){Some(Token::And)|Some(Token::AndAnd)=>LogicalOp::And,_=>break};self.pos+=1;l=Expr::Logical(Box::new(l),o,Box::new(self.compare()?))}Ok(l)}
 fn compare(&mut self)->Result<Expr,String>{let l=self.additive()?;let o=match self.tokens.get(self.pos){Some(Token::EqualEqual)=>Some(CompareOp::Eq),Some(Token::NotEqual)=>Some(CompareOp::Ne),Some(Token::Less)=>Some(CompareOp::Lt),Some(Token::LessEqual)=>Some(CompareOp::Le),Some(Token::Greater)=>Some(CompareOp::Gt),Some(Token::GreaterEqual)=>Some(CompareOp::Ge),_=>(None)};if let Some(o)=o{self.pos+=1;Ok(Expr::Compare(Box::new(l),o,Box::new(self.additive()?)))}else{Ok(l)}}
 fn additive(&mut self)->Result<Expr,String>{let mut l=self.term()?;loop{let o=match self.tokens.get(self.pos){Some(Token::Plus)=>Op::Add,Some(Token::Minus)=>Op::Sub,_=>break};self.pos+=1;l=Expr::Binary(Box::new(l),o,Box::new(self.term()?))}Ok(l)}
 fn term(&mut self)->Result<Expr,String>{let mut l=self.unary()?;loop{let o=match self.tokens.get(self.pos){Some(Token::Star)=>Op::Mul,Some(Token::Slash)=>Op::Div,Some(Token::Percent)=>Op::Mod,_=>break};self.pos+=1;l=Expr::Binary(Box::new(l),o,Box::new(self.unary()?))}Ok(l)}
 fn unary(&mut self)->Result<Expr,String>{if self.at(&Token::Minus){self.pos+=1;return Ok(Expr::Binary(Box::new(Expr::Number(0)),Op::Sub,Box::new(self.unary()?)))}if self.at(&Token::Not)||self.at(&Token::Bang){self.pos+=1;return Ok(Expr::Not(Box::new(self.unary()?)))}self.primary()}
 fn primary(&mut self)->Result<Expr,String>{let mut e=match self.tokens.get(self.pos){Some(Token::Number(n))=>{let n=*n;self.pos+=1;Expr::Number(n)},Some(Token::True)=>{self.pos+=1;Expr::Bool(true)},Some(Token::False)=>{self.pos+=1;Expr::Bool(false)},Some(Token::String(s))=>{let s=s.clone();self.pos+=1;Expr::String(s)},Some(Token::Identifier(s))=>{let n=s.clone();self.pos+=1;Expr::Variable(n)},Some(Token::LeftParen)=>{self.pos+=1;let x=self.expr()?;self.expect(Token::RightParen,"expected `)`")?;x},Some(Token::LeftBracket)=>{self.pos+=1;let mut v=Vec::new();if !self.at(&Token::RightBracket){loop{v.push(self.expr()?);if self.at(&Token::Comma){self.pos+=1}else{break}}}self.expect(Token::RightBracket,"expected `]`")?;Expr::Array(v)},Some(Token::LeftBrace)=>{self.pos+=1;let mut v=Vec::new();if !self.at(&Token::RightBrace){loop{let k=match self.tokens.get(self.pos){Some(Token::Identifier(s))|Some(Token::String(s))=>{let x=s.clone();self.pos+=1;x},_=>(return Err("expected table key".into()))};self.expect(Token::Colon,"expected `:`")?;v.push((k,self.expr()?));if self.at(&Token::Comma){self.pos+=1}else{break}}}self.expect(Token::RightBrace,"expected `}`")?;Expr::Table(v)},_=>(return Err("expected expression".into()))};
  loop{if self.at(&Token::LeftParen){self.pos+=1;let mut a=Vec::new();if !self.at(&Token::RightParen){loop{a.push(self.expr()?);if self.at(&Token::Comma){self.pos+=1}else{break}}}self.expect(Token::RightParen,"expected `)`")?;e=match e{Expr::Variable(n)=>Expr::Call(n,a),Expr::Property(o,n)=>Expr::Method(o,n,a),_=>(return Err("only functions and methods can be called".into()))}}else if self.at(&Token::LeftBracket){self.pos+=1;let i=self.expr()?;self.expect(Token::RightBracket,"expected `]`")?;e=Expr::Index(Box::new(e),Box::new(i))}else if self.at(&Token::Dot){self.pos+=1;let n=self.identifier()?;e=Expr::Property(Box::new(e),n)}else{break}}Ok(e)}
}
