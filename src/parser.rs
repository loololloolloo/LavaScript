use crate::lexer::Token;

#[derive(Debug, Clone)]
pub enum Expr { Number(i32), Bool(bool), String(String), Variable(String), Binary(Box<Expr>, Op, Box<Expr>), Compare(Box<Expr>, CompareOp, Box<Expr>), Logical(Box<Expr>, LogicalOp, Box<Expr>), Not(Box<Expr>), Call(String, Vec<Expr>) }
#[derive(Debug, Clone)]
pub enum Op { Add, Sub, Mul, Div, Mod }
#[derive(Debug, Clone)]
pub enum CompareOp { Eq, Ne, Lt, Le, Gt, Ge }
#[derive(Debug, Clone)]
pub enum LogicalOp { And, Or }
#[derive(Debug, Clone)]
pub enum Stmt {
    Let(String, Expr), Const(String, Expr), Assign(String, Expr), Print(Expr), Return(Expr), Break, Continue,
    If { condition: Expr, then_body: Vec<Stmt>, else_body: Vec<Stmt> },
    While { condition: Expr, body: Vec<Stmt> },
    For { name: String, start: Expr, end: Expr, step: Expr, body: Vec<Stmt> },
    Repeat { body: Vec<Stmt>, condition: Expr },
    Do { body: Vec<Stmt> },
}
#[derive(Debug, Clone)]
pub struct FunctionDecl { pub name: String, pub params: Vec<String>, pub body: Vec<Stmt> }
#[derive(Debug, Clone)]
pub struct Program { pub functions: Vec<FunctionDecl>, pub main: Vec<Stmt> }

pub fn parse(tokens: &[Token]) -> Result<Program, String> {
    let mut p = Parser { tokens, pos: 0 }; p.skip_separators();
    let mut functions = Vec::new(); let mut main = Vec::new();
    while !p.at(&Token::Eof) { p.skip_separators(); if p.at(&Token::Eof) { break; } if p.at(&Token::Function) { functions.push(p.parse_function()?); } else { main.push(p.statement(false)?); } p.skip_separators(); }
    Ok(Program { functions, main })
}

struct Parser<'a> { tokens: &'a [Token], pos: usize }
impl<'a> Parser<'a> {
    fn at(&self,t:&Token)->bool{self.tokens.get(self.pos)==Some(t)}
    fn skip_separators(&mut self){while self.at(&Token::Newline)||self.at(&Token::Semicolon){self.pos+=1;}}
    fn require_line_end(&mut self,message:&str)->Result<(),String>{if self.at(&Token::Newline)||self.at(&Token::Semicolon)||self.at(&Token::End)||self.at(&Token::Else)||self.at(&Token::ElseIf)||self.at(&Token::Eof){self.skip_separators();Ok(())}else{Err(message.into())}}
    fn statement(&mut self,in_function:bool)->Result<Stmt,String>{
        match self.tokens.get(self.pos) {
            Some(Token::Let)|Some(Token::Local)=>{self.pos+=1;let name=self.identifier()?;self.expect(Token::Equal,"expected `=`")?;Ok(Stmt::Let(name,self.expr()?))}
            Some(Token::Const)=>{self.pos+=1;let name=self.identifier()?;self.expect(Token::Equal,"expected `=`")?;Ok(Stmt::Const(name,self.expr()?))}
            Some(Token::Identifier(_))=>{let name=self.identifier()?;self.expect(Token::Equal,"expected `=` after variable name")?;Ok(Stmt::Assign(name,self.expr()?))}
            Some(Token::Print)=>{self.pos+=1;Ok(Stmt::Print(self.expr()?))}
            Some(Token::Return)=>{if !in_function{return Err("`return` is only valid inside a function".into());}self.pos+=1;Ok(Stmt::Return(self.expr()?))}
            Some(Token::If)=>self.parse_if(in_function),
            Some(Token::While)=>self.parse_while(in_function),
            Some(Token::For)=>self.parse_for(in_function),
            Some(Token::Repeat)=>self.parse_repeat(in_function),
            Some(Token::Do)=>self.parse_do(in_function),
            Some(Token::Break)=>{self.pos+=1;Ok(Stmt::Break)}
            Some(Token::Continue)=>{self.pos+=1;Ok(Stmt::Continue)}
            _=>Err("expected statement".into())
        }
    }
    fn parse_function(&mut self)->Result<FunctionDecl,String>{
        self.pos+=1;let name=self.identifier()?;self.expect(Token::LeftParen,"expected `(` after function name")?;let mut params=Vec::new();
        if !self.at(&Token::RightParen){loop{let param=self.identifier()?;if params.contains(&param){return Err(format!("duplicate parameter `{param}`"));}params.push(param);if self.at(&Token::Comma){self.pos+=1;}else{break;}}}
        self.expect(Token::RightParen,"expected `)` after function parameters")?;self.skip_separators();let body=self.block(&[Token::End],true)?;
        self.expect(Token::End,"expected `end` after function")?;if !body.iter().any(|s|matches!(s,Stmt::Return(_))){return Err(format!("function `{name}` must contain a return"));}Ok(FunctionDecl{name,params,body})
    }
    fn block(&mut self,stops:&[Token],in_function:bool)->Result<Vec<Stmt>,String>{let mut out=Vec::new();loop{self.skip_separators();if self.at(&Token::Eof)||stops.iter().any(|t|self.at(t)){break;}if self.at(&Token::Function){return Err("nested functions are not supported".into());}out.push(self.statement(in_function)?);self.skip_separators();}Ok(out)}
    fn parse_if(&mut self,in_function:bool)->Result<Stmt,String>{self.pos+=1;let condition=self.expr()?;if self.at(&Token::Then){self.pos+=1;}self.require_line_end("expected newline after `if` condition")?;let then_body=self.block(&[Token::Else,Token::ElseIf,Token::End],in_function)?;let else_body=if self.at(&Token::ElseIf){self.pos+=1;vec![self.parse_if_after_keyword(in_function)?]}else if self.at(&Token::Else){self.pos+=1;self.require_line_end("expected newline after `else`")?;self.block(&[Token::End],in_function)?}else{Vec::new()};self.expect(Token::End,"expected `end` after `if`")?;Ok(Stmt::If{condition,then_body,else_body})}
    fn parse_if_after_keyword(&mut self,in_function:bool)->Result<Stmt,String>{let condition=self.expr()?;if self.at(&Token::Then){self.pos+=1;}self.require_line_end("expected newline after `elseif` condition")?;let then_body=self.block(&[Token::Else,Token::ElseIf,Token::End],in_function)?;let else_body=if self.at(&Token::ElseIf){self.pos+=1;vec![self.parse_if_after_keyword(in_function)?]}else if self.at(&Token::Else){self.pos+=1;self.require_line_end("expected newline after `else`")?;self.block(&[Token::End],in_function)?}else{Vec::new()};Ok(Stmt::If{condition,then_body,else_body})}
    fn parse_while(&mut self,in_function:bool)->Result<Stmt,String>{self.pos+=1;let condition=self.expr()?;if self.at(&Token::Then){self.pos+=1;}self.require_line_end("expected newline after `while` condition")?;let body=self.block(&[Token::End],in_function)?;self.expect(Token::End,"expected `end` after `while`")?;Ok(Stmt::While{condition,body})}
    fn parse_for(&mut self,in_function:bool)->Result<Stmt,String>{self.pos+=1;let name=self.identifier()?;self.expect(Token::Equal,"expected `=` after for variable")?;let start=self.expr()?;self.expect(Token::Comma,"expected `,` after for start")?;let end=self.expr()?;let step=if self.at(&Token::Comma){self.pos+=1;self.expr()?}else{Expr::Number(1)};self.require_line_end("expected newline after `for` header")?;let body=self.block(&[Token::End],in_function)?;self.expect(Token::End,"expected `end` after `for`")?;Ok(Stmt::For{name,start,end,step,body})}
    fn parse_repeat(&mut self,in_function:bool)->Result<Stmt,String>{self.pos+=1;self.require_line_end("expected newline after `repeat`")?;let body=self.block(&[Token::Until],in_function)?;self.expect(Token::Until,"expected `until` after `repeat` body")?;let condition=self.expr()?;Ok(Stmt::Repeat{body,condition})}
    fn parse_do(&mut self,in_function:bool)->Result<Stmt,String>{self.pos+=1;self.require_line_end("expected newline after `do`")?;let body=self.block(&[Token::End],in_function)?;self.expect(Token::End,"expected `end` after `do`")?;Ok(Stmt::Do{body})}
    fn identifier(&mut self)->Result<String,String>{match self.tokens.get(self.pos){Some(Token::Identifier(s))=>{let n=s.clone();self.pos+=1;Ok(n)},_=>Err("expected identifier".into())}}
    fn expect(&mut self,t:Token,message:&str)->Result<(),String>{if self.at(&t){self.pos+=1;Ok(())}else{Err(message.into())}}
    fn expr(&mut self)->Result<Expr,String>{self.logic_or()}
    fn logic_or(&mut self)->Result<Expr,String>{let mut left=self.logic_and()?;loop{let op=match self.tokens.get(self.pos){Some(Token::Or)|Some(Token::OrOr)=>LogicalOp::Or,_=>break};self.pos+=1;left=Expr::Logical(Box::new(left),op,Box::new(self.logic_and()?));}Ok(left)}
    fn logic_and(&mut self)->Result<Expr,String>{let mut left=self.compare_expr()?;loop{let op=match self.tokens.get(self.pos){Some(Token::And)|Some(Token::AndAnd)=>LogicalOp::And,_=>break};self.pos+=1;left=Expr::Logical(Box::new(left),op,Box::new(self.compare_expr()?));}Ok(left)}
    fn compare_expr(&mut self)->Result<Expr,String>{let left=self.additive()?;match self.tokens.get(self.pos){Some(Token::EqualEqual)=>self.compare(left,CompareOp::Eq),Some(Token::NotEqual)=>self.compare(left,CompareOp::Ne),Some(Token::Less)=>self.compare(left,CompareOp::Lt),Some(Token::LessEqual)=>self.compare(left,CompareOp::Le),Some(Token::Greater)=>self.compare(left,CompareOp::Gt),Some(Token::GreaterEqual)=>self.compare(left,CompareOp::Ge),_=>Ok(left)}}
    fn compare(&mut self,left:Expr,op:CompareOp)->Result<Expr,String>{self.pos+=1;Ok(Expr::Compare(Box::new(left),op,Box::new(self.additive()?)))}
    fn additive(&mut self)->Result<Expr,String>{let mut left=self.term()?;loop{let op=match self.tokens.get(self.pos){Some(Token::Plus)=>Op::Add,Some(Token::Minus)=>Op::Sub,_=>break};self.pos+=1;left=Expr::Binary(Box::new(left),op,Box::new(self.term()?));}Ok(left)}
    fn term(&mut self)->Result<Expr,String>{let mut left=self.unary()?;loop{let op=match self.tokens.get(self.pos){Some(Token::Star)=>Op::Mul,Some(Token::Slash)=>Op::Div,Some(Token::Percent)=>Op::Mod,_=>break};self.pos+=1;left=Expr::Binary(Box::new(left),op,Box::new(self.unary()?));}Ok(left)}
    fn unary(&mut self)->Result<Expr,String>{if self.at(&Token::Minus){self.pos+=1;return Ok(Expr::Binary(Box::new(Expr::Number(0)),Op::Sub,Box::new(self.unary()?)));}if self.at(&Token::Not)||self.at(&Token::Bang){self.pos+=1;return Ok(Expr::Not(Box::new(self.unary()?)));}self.primary()}
    fn primary(&mut self)->Result<Expr,String>{match self.tokens.get(self.pos){Some(Token::Number(n))=>{let n=*n;self.pos+=1;Ok(Expr::Number(n))},Some(Token::True)=>{self.pos+=1;Ok(Expr::Bool(true))},Some(Token::False)=>{self.pos+=1;Ok(Expr::Bool(false))},Some(Token::String(s))=>{let s=s.clone();self.pos+=1;Ok(Expr::String(s))},Some(Token::Identifier(s))=>{let name=s.clone();self.pos+=1;if self.at(&Token::LeftParen){self.pos+=1;let mut args=Vec::new();if !self.at(&Token::RightParen){loop{args.push(self.expr()?);if self.at(&Token::Comma){self.pos+=1;}else{break;}}}self.expect(Token::RightParen,"expected `)` after function arguments")?;Ok(Expr::Call(name,args))}else{Ok(Expr::Variable(name))}},Some(Token::LeftParen)=>{self.pos+=1;let e=self.expr()?;self.expect(Token::RightParen,"expected `)`")?;Ok(e)},_=>Err("expected expression".into())}}
}
