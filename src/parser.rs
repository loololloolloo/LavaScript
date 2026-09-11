use crate::lexer::Token;

#[derive(Debug, Clone)]
pub enum Expr {
    Number(i32),
    Variable(String),
    Binary(Box<Expr>, Op, Box<Expr>),
    Compare(Box<Expr>, CompareOp, Box<Expr>),
}

#[derive(Debug, Clone)]
pub enum Op { Add, Sub, Mul, Div }

#[derive(Debug, Clone)]
pub enum CompareOp { Eq, Ne, Lt, Le, Gt, Ge }

#[derive(Debug, Clone)]
pub enum Stmt {
    Let(String, Expr),
    Print(Expr),
    If { condition: Expr, then_body: Vec<Stmt>, else_body: Vec<Stmt> },
    While { condition: Expr, body: Vec<Stmt> },
}

pub fn parse(tokens: &[Token]) -> Result<Vec<Stmt>, String> {
    let mut p = Parser { tokens, pos: 0 };
    p.skip_newlines();
    p.block(false, false)
}

struct Parser<'a> { tokens: &'a [Token], pos: usize }

impl<'a> Parser<'a> {
    fn at(&self, t: &Token) -> bool { self.tokens.get(self.pos) == Some(t) }

    fn skip_newlines(&mut self) {
        while self.at(&Token::Newline) { self.pos += 1; }
    }

    fn block(&mut self, stop_else: bool, stop_end: bool) -> Result<Vec<Stmt>, String> {
        let mut out = Vec::new();
        loop {
            self.skip_newlines();
            if self.at(&Token::Eof) || (stop_end && self.at(&Token::End)) || (stop_else && self.at(&Token::Else)) { break; }
            out.push(self.statement()?);
            self.skip_newlines();
        }
        Ok(out)
    }

    fn statement(&mut self) -> Result<Stmt, String> {
        match self.tokens.get(self.pos) {
            Some(Token::Let) => {
                self.pos += 1;
                let name = match self.tokens.get(self.pos) {
                    Some(Token::Identifier(s)) => { let n=s.clone(); self.pos+=1; n }
                    _ => return Err("expected variable name".into()),
                };
                if !self.at(&Token::Equal) { return Err("expected `=`".into()); }
                self.pos += 1;
                Ok(Stmt::Let(name, self.expr()?))
            }
            Some(Token::Print) => { self.pos += 1; Ok(Stmt::Print(self.expr()?)) }
            Some(Token::If) => self.parse_if(),
            Some(Token::While) => self.parse_while(),
            _ => Err("expected `let`, `print`, `if`, or `while`".into()),
        }
    }

    fn parse_if(&mut self) -> Result<Stmt, String> {
        self.pos += 1;
        let condition = self.expr()?;
        if self.at(&Token::Then) { self.pos += 1; }
        if !self.at(&Token::Newline) && !self.at(&Token::End) && !self.at(&Token::Else) {
            return Err("expected newline after `if` condition".into());
        }
        self.skip_newlines();
        let then_body = self.block(true, true)?;
        let else_body = if self.at(&Token::Else) {
            self.pos += 1;
            self.skip_newlines();
            self.block(false, true)?
        } else { Vec::new() };
        if !self.at(&Token::End) { return Err("expected `end` after `if`".into()); }
        self.pos += 1;
        Ok(Stmt::If { condition, then_body, else_body })
    }

    fn parse_while(&mut self) -> Result<Stmt, String> {
        self.pos += 1;
        let condition = self.expr()?;
        if self.at(&Token::Then) { self.pos += 1; }
        if !self.at(&Token::Newline) && !self.at(&Token::End) {
            return Err("expected newline after `while` condition".into());
        }
        self.skip_newlines();
        let body = self.block(false, true)?;
        if !self.at(&Token::End) { return Err("expected `end` after `while`".into()); }
        self.pos += 1;
        Ok(Stmt::While { condition, body })
    }

    fn expr(&mut self) -> Result<Expr, String> {
        let mut left = self.term()?;
        loop {
            let op = match self.tokens.get(self.pos) {
                Some(Token::Plus) => Op::Add,
                Some(Token::Minus) => Op::Sub,
                _ => break,
            };
            self.pos += 1;
            left = Expr::Binary(Box::new(left), op, Box::new(self.term()?));
        }
        match self.tokens.get(self.pos) {
            Some(Token::EqualEqual) => self.compare(left, CompareOp::Eq),
            Some(Token::NotEqual) => self.compare(left, CompareOp::Ne),
            Some(Token::Less) => self.compare(left, CompareOp::Lt),
            Some(Token::LessEqual) => self.compare(left, CompareOp::Le),
            Some(Token::Greater) => self.compare(left, CompareOp::Gt),
            Some(Token::GreaterEqual) => self.compare(left, CompareOp::Ge),
            _ => Ok(left),
        }
    }

    fn compare(&mut self, left: Expr, op: CompareOp) -> Result<Expr, String> {
        self.pos += 1;
        Ok(Expr::Compare(Box::new(left), op, Box::new(self.term()?)))
    }

    fn term(&mut self) -> Result<Expr, String> {
        let mut left = self.primary()?;
        loop {
            let op = match self.tokens.get(self.pos) {
                Some(Token::Star) => Op::Mul,
                Some(Token::Slash) => Op::Div,
                _ => break,
            };
            self.pos += 1;
            left = Expr::Binary(Box::new(left), op, Box::new(self.primary()?));
        }
        Ok(left)
    }

    fn primary(&mut self) -> Result<Expr, String> {
        match self.tokens.get(self.pos) {
            Some(Token::Number(n)) => { let n=*n; self.pos+=1; Ok(Expr::Number(n)) }
            Some(Token::Identifier(s)) => { let s=s.clone(); self.pos+=1; Ok(Expr::Variable(s)) }
            Some(Token::LeftParen) => {
                self.pos += 1;
                let e = self.expr()?;
                if !self.at(&Token::RightParen) { return Err("expected `)`".into()); }
                self.pos += 1;
                Ok(e)
            }
            _ => Err("expected expression".into()),
        }
    }
}
