use crate::lexer::Token;

#[derive(Debug, Clone)]
pub enum Expr { Number(i32), Variable(String), Binary(Box<Expr>, Op, Box<Expr>) }
#[derive(Debug, Clone)]
pub enum Op { Add, Sub, Mul, Div }
#[derive(Debug, Clone)]
pub enum Stmt { Let(String, Expr), Print(Expr) }

pub fn parse(tokens: &[Token]) -> Result<Vec<Stmt>, String> {
    let mut p = Parser { tokens, pos: 0 };
    let mut out = Vec::new();
    while !p.at(&Token::Eof) {
        if p.at(&Token::Newline) { p.pos += 1; continue; }
        out.push(p.statement()?);
        if p.at(&Token::Newline) { p.pos += 1; }
    }
    Ok(out)
}

struct Parser<'a> { tokens: &'a [Token], pos: usize }
impl<'a> Parser<'a> {
    fn at(&self, t: &Token) -> bool { self.tokens.get(self.pos) == Some(t) }
    fn statement(&mut self) -> Result<Stmt, String> {
        match self.tokens.get(self.pos) {
            Some(Token::Let) => {
                self.pos += 1;
                let name = match self.tokens.get(self.pos) { Some(Token::Identifier(s)) => { let n=s.clone(); self.pos+=1; n }, _ => return Err("expected variable name".into()) };
                if !self.at(&Token::Equal) { return Err("expected `=`".into()); }
                self.pos += 1;
                Ok(Stmt::Let(name, self.expr()?))
            }
            Some(Token::Print) => { self.pos += 1; Ok(Stmt::Print(self.expr()?)) }
            _ => Err("expected `let` or `print`".into()),
        }
    }
    fn expr(&mut self) -> Result<Expr, String> {
        let mut left = self.term()?;
        loop {
            let op = match self.tokens.get(self.pos) { Some(Token::Plus)=>Op::Add, Some(Token::Minus)=>Op::Sub, _=>break };
            self.pos += 1; left = Expr::Binary(Box::new(left), op, Box::new(self.term()?));
        }
        Ok(left)
    }
    fn term(&mut self) -> Result<Expr, String> {
        let mut left = self.primary()?;
        loop {
            let op = match self.tokens.get(self.pos) { Some(Token::Star)=>Op::Mul, Some(Token::Slash)=>Op::Div, _=>break };
            self.pos += 1; left = Expr::Binary(Box::new(left), op, Box::new(self.primary()?));
        }
        Ok(left)
    }
    fn primary(&mut self) -> Result<Expr, String> {
        match self.tokens.get(self.pos) {
            Some(Token::Number(n)) => { let n=*n; self.pos+=1; Ok(Expr::Number(n)) }
            Some(Token::Identifier(s)) => { let s=s.clone(); self.pos+=1; Ok(Expr::Variable(s)) }
            Some(Token::LeftParen) => { self.pos+=1; let e=self.expr()?; if !self.at(&Token::RightParen) { return Err("expected `)`".into()); } self.pos+=1; Ok(e) }
            _ => Err("expected expression".into()),
        }
    }
}
