use crate::lexer::Token;

#[derive(Debug, Clone)]
pub enum Expr { Number(i32), Variable(String), Binary(Box<Expr>, Op, Box<Expr>), Compare(Box<Expr>, CompareOp, Box<Expr>), Call(String, Vec<Expr>) }
#[derive(Debug, Clone)]
pub enum Op { Add, Sub, Mul, Div }
#[derive(Debug, Clone)]
pub enum CompareOp { Eq, Ne, Lt, Le, Gt, Ge }
#[derive(Debug, Clone)]
pub enum Stmt { Let(String, Expr), Assign(String, Expr), Print(Expr), Return(Expr), If { condition: Expr, then_body: Vec<Stmt>, else_body: Vec<Stmt> }, While { condition: Expr, body: Vec<Stmt> } }
#[derive(Debug, Clone)]
pub struct FunctionDecl { pub name: String, pub params: Vec<String>, pub body: Vec<Stmt> }
#[derive(Debug, Clone)]
pub struct Program { pub functions: Vec<FunctionDecl>, pub main: Vec<Stmt> }

pub fn parse(tokens: &[Token]) -> Result<Program, String> {
    let mut p = Parser { tokens, pos: 0 }; p.skip_newlines();
    let mut functions = Vec::new(); let mut main = Vec::new();
    while !p.at(&Token::Eof) { p.skip_newlines(); if p.at(&Token::Eof) { break; } if p.at(&Token::Function) { functions.push(p.parse_function()?); } else { main.push(p.statement(false)?); } p.skip_newlines(); }
    Ok(Program { functions, main })
}

struct Parser<'a> { tokens: &'a [Token], pos: usize }
impl<'a> Parser<'a> {
    fn at(&self,t:&Token)->bool{self.tokens.get(self.pos)==Some(t)}
    fn skip_newlines(&mut self){while self.at(&Token::Newline){self.pos+=1;}}
    fn statement(&mut self,in_function:bool)->Result<Stmt,String>{
        match self.tokens.get(self.pos) {
            Some(Token::Let)=>{self.pos+=1;let name=self.identifier()?;self.expect(Token::Equal,"expected `=`")?;Ok(Stmt::Let(name,self.expr()?))}
            Some(Token::Identifier(_))=>{let name=self.identifier()?;self.expect(Token::Equal,"expected `=` after variable name")?;Ok(Stmt::Assign(name,self.expr()?))}
            Some(Token::Print)=>{self.pos+=1;Ok(Stmt::Print(self.expr()?))}
            Some(Token::Return)=>{if !in_function{return Err("`return` is only valid inside a function".into());}self.pos+=1;Ok(Stmt::Return(self.expr()?))}
            Some(Token::If)=>self.parse_if(in_function), Some(Token::While)=>self.parse_while(in_function), _=>Err("expected `let`, `print`, `if`, `while`, or `return`".into())
        }
    }
    fn parse_function(&mut self)->Result<FunctionDecl,String>{
        self.pos+=1;let name=self.identifier()?;self.expect(Token::LeftParen,"expected `(` after function name")?;let mut params=Vec::new();
        if !self.at(&Token::RightParen){loop{let param=self.identifier()?;if params.contains(&param){return Err(format!("duplicate parameter `{param}`"));}params.push(param);if self.at(&Token::Comma){self.pos+=1;}else{break;}}}
        self.expect(Token::RightParen,"expected `)` after function parameters")?;self.skip_newlines();let body=self.block(true,false)?;
        self.expect(Token::End,format!("expected `end` after function `{name}`").as_str())?;
        if !body.iter().any(|s|matches!(s,Stmt::Return(_))){return Err(format!("function `{name}` must contain a return"));}
        Ok(FunctionDecl{name,params,body})
    }
    fn block(&mut self,stop_end:bool,stop_else:bool)->Result<Vec<Stmt>,String>{
        let mut out=Vec::new();loop{self.skip_newlines();if self.at(&Token::Eof)||(stop_end&&self.at(&Token::End))||(stop_else&&self.at(&Token::Else)){break;}if self.at(&Token::Function){return Err("nested functions are not supported".into());}out.push(self.statement(true)?);self.skip_newlines();}Ok(out)
    }
    fn parse_if(&mut self,_:bool)->Result<Stmt,String>{self.pos+=1;let condition=self.expr()?;if self.at(&Token::Then){self.pos+=1;}if !self.at(&Token::Newline)&&!self.at(&Token::End)&&!self.at(&Token::Else){return Err("expected newline after `if` condition".into());}self.skip_newlines();let then_body=self.block(true,true)?;let else_body=if self.at(&Token::Else){self.pos+=1;self.skip_newlines();self.block(true,true)?}else{Vec::new()};self.expect(Token::End,"expected `end` after `if`")?;Ok(Stmt::If{condition,then_body,else_body})}
    fn parse_while(&mut self,_:bool)->Result<Stmt,String>{self.pos+=1;let condition=self.expr()?;if self.at(&Token::Then){self.pos+=1;}if !self.at(&Token::Newline)&&!self.at(&Token::End){return Err("expected newline after `while` condition".into());}self.skip_newlines();let body=self.block(true,false)?;self.expect(Token::End,"expected `end` after `while`")?;Ok(Stmt::While{condition,body})}
    fn identifier(&mut self)->Result<String,String>{match self.tokens.get(self.pos){Some(Token::Identifier(s))=>{let n=s.clone();self.pos+=1;Ok(n)},_=>Err("expected identifier".into())}}
    fn expect(&mut self,t:Token,message:&str)->Result<(),String>{if self.at(&t){self.pos+=1;Ok(())}else{Err(message.into())}}
    fn expr(&mut self)->Result<Expr,String>{let mut left=self.term()?;loop{let op=match self.tokens.get(self.pos){Some(Token::Plus)=>Op::Add,Some(Token::Minus)=>Op::Sub,_=>break};self.pos+=1;left=Expr::Binary(Box::new(left),op,Box::new(self.term()?));}match self.tokens.get(self.pos){Some(Token::EqualEqual)=>self.compare(left,CompareOp::Eq),Some(Token::NotEqual)=>self.compare(left,CompareOp::Ne),Some(Token::Less)=>self.compare(left,CompareOp::Lt),Some(Token::LessEqual)=>self.compare(left,CompareOp::Le),Some(Token::Greater)=>self.compare(left,CompareOp::Gt),Some(Token::GreaterEqual)=>self.compare(left,CompareOp::Ge),_=>Ok(left)}}
    fn compare(&mut self,left:Expr,op:CompareOp)->Result<Expr,String>{self.pos+=1;Ok(Expr::Compare(Box::new(left),op,Box::new(self.term()?)))}
    fn term(&mut self)->Result<Expr,String>{let mut left=self.primary()?;loop{let op=match self.tokens.get(self.pos){Some(Token::Star)=>Op::Mul,Some(Token::Slash)=>Op::Div,_=>break};self.pos+=1;left=Expr::Binary(Box::new(left),op,Box::new(self.primary()?));}Ok(left)}
    fn primary(&mut self)->Result<Expr,String>{match self.tokens.get(self.pos){Some(Token::Number(n))=>{let n=*n;self.pos+=1;Ok(Expr::Number(n))},Some(Token::Identifier(s))=>{let name=s.clone();self.pos+=1;if self.at(&Token::LeftParen){self.pos+=1;let mut args=Vec::new();if !self.at(&Token::RightParen){loop{args.push(self.expr()?);if self.at(&Token::Comma){self.pos+=1;}else{break;}}}self.expect(Token::RightParen,"expected `)` after function arguments")?;Ok(Expr::Call(name,args))}else{Ok(Expr::Variable(name))}},Some(Token::LeftParen)=>{self.pos+=1;let e=self.expr()?;self.expect(Token::RightParen,"expected `)`")?;Ok(e)},_=>Err("expected expression".into())}}
}
