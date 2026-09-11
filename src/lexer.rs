#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Let, Local, Const, Print, If, Else, ElseIf, While, For, Repeat, Until, Do, Then, Function, Return, End, True, False, And, Or, Not, Break, Continue,
    Identifier(String), Number(i32), String(String),
    Plus, Minus, Star, Slash, Percent,
    Equal, EqualEqual, NotEqual, Less, LessEqual, Greater, GreaterEqual,
    AndAnd, OrOr, Bang,
    LeftParen, RightParen, Comma, Semicolon, Newline, Eof,
}

pub fn lex(source: &str) -> Result<Vec<Token>, String> {
    let chars: Vec<char> = source.chars().collect();
    let mut tokens = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        match chars[i] {
            ' ' | '\t' | '\r' => i += 1,
            '\n' => { tokens.push(Token::Newline); i += 1; }
            ';' => { tokens.push(Token::Semicolon); i += 1; }
            '-' if i + 1 < chars.len() && chars[i + 1] == '-' => { i += 2; while i < chars.len() && chars[i] != '\n' { i += 1; } }
            '/' if i + 1 < chars.len() && chars[i + 1] == '/' => { i += 2; while i < chars.len() && chars[i] != '\n' { i += 1; } }
            '"' | '\'' => {
                let quote = chars[i]; i += 1; let mut value = String::new();
                while i < chars.len() && chars[i] != quote {
                    if chars[i] == '\\' { i += 1; if i >= chars.len() { return Err("unterminated string".into()); } match chars[i] { 'n'=>value.push('\n'),'r'=>value.push('\r'),'t'=>value.push('\t'),'"'=>value.push('"'),'\''=>value.push('\''),'\\'=>value.push('\\'),c=>return Err(format!("unsupported escape `\\{c}`")), } } else { value.push(chars[i]); }
                    i += 1;
                }
                if i >= chars.len() { return Err("unterminated string".into()); } i += 1; tokens.push(Token::String(value));
            }
            '=' => { i += 1; if i < chars.len() && chars[i] == '=' { tokens.push(Token::EqualEqual); i += 1; } else { tokens.push(Token::Equal); } }
            '!' => { i += 1; if i < chars.len() && chars[i] == '=' { tokens.push(Token::NotEqual); i += 1; } else { tokens.push(Token::Bang); } }
            '&' => { i += 1; if i < chars.len() && chars[i] == '&' { tokens.push(Token::AndAnd); i += 1; } else { return Err("expected `&&`".into()); } }
            '|' => { i += 1; if i < chars.len() && chars[i] == '|' { tokens.push(Token::OrOr); i += 1; } else { return Err("expected `||`".into()); } }
            '<' => { i += 1; if i < chars.len() && chars[i] == '=' { tokens.push(Token::LessEqual); i += 1; } else { tokens.push(Token::Less); } }
            '>' => { i += 1; if i < chars.len() && chars[i] == '=' { tokens.push(Token::GreaterEqual); i += 1; } else { tokens.push(Token::Greater); } }
            '+' => { tokens.push(Token::Plus); i += 1; }
            '-' => { tokens.push(Token::Minus); i += 1; }
            '*' => { tokens.push(Token::Star); i += 1; }
            '/' => { tokens.push(Token::Slash); i += 1; }
            '%' => { tokens.push(Token::Percent); i += 1; }
            '(' => { tokens.push(Token::LeftParen); i += 1; }
            ')' => { tokens.push(Token::RightParen); i += 1; }
            ',' => { tokens.push(Token::Comma); i += 1; }
            '0' if i + 1 < chars.len() && (chars[i + 1] == 'x' || chars[i + 1] == 'X') => {
                i += 2; let start = i; while i < chars.len() && (chars[i].is_ascii_hexdigit() || chars[i] == '_') { i += 1; }
                let raw: String = chars[start..i].iter().filter(|c| **c != '_').collect(); if raw.is_empty() { return Err("expected hexadecimal digits after `0x`".into()); }
                tokens.push(Token::Number(i32::from_str_radix(&raw, 16).map_err(|_| "invalid hexadecimal number")?));
            }
            '0' if i + 1 < chars.len() && (chars[i + 1] == 'b' || chars[i + 1] == 'B') => {
                i += 2; let start = i; while i < chars.len() && (chars[i] == '0' || chars[i] == '1' || chars[i] == '_') { i += 1; }
                let raw: String = chars[start..i].iter().filter(|c| **c != '_').collect(); if raw.is_empty() { return Err("expected binary digits after `0b`".into()); }
                tokens.push(Token::Number(i32::from_str_radix(&raw, 2).map_err(|_| "invalid binary number")?));
            }
            c if c.is_ascii_digit() => { let start=i; while i<chars.len()&&(chars[i].is_ascii_digit()||chars[i]=='_'){i+=1;} let value:String=chars[start..i].iter().filter(|c| **c != '_').collect(); tokens.push(Token::Number(value.parse().map_err(|_|"invalid number")?)); }
            c if c.is_ascii_alphabetic() || c == '_' => {
                let start=i; while i<chars.len()&&(chars[i].is_ascii_alphanumeric()||chars[i]=='_'){i+=1;} let word:String=chars[start..i].iter().collect();
                tokens.push(match word.as_str(){"let"=>Token::Let,"local"=>Token::Local,"const"=>Token::Const,"print"=>Token::Print,"if"=>Token::If,"else"=>Token::Else,"elseif"=>Token::ElseIf,"while"=>Token::While,"for"=>Token::For,"repeat"=>Token::Repeat,"until"=>Token::Until,"do"=>Token::Do,"then"=>Token::Then,"function"=>Token::Function,"return"=>Token::Return,"end"=>Token::End,"true"=>Token::True,"false"=>Token::False,"and"=>Token::And,"or"=>Token::Or,"not"=>Token::Not,"break"=>Token::Break,"continue"=>Token::Continue,_=>Token::Identifier(word)});
            }
            c => return Err(format!("unexpected character `{c}`")),
        }
    }
    tokens.push(Token::Eof); Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::{lex, Token};
    #[test] fn lexes_booleans_and_logic() { assert_eq!(lex("true and not false || true").unwrap(), vec![Token::True, Token::And, Token::Not, Token::False, Token::OrOr, Token::True, Token::Eof]); }
    #[test] fn lexes_comments() { assert_eq!(lex("print 1 -- hello\nprint 2 // world").unwrap(), vec![Token::Print,Token::Number(1),Token::Newline,Token::Print,Token::Number(2),Token::Eof]); }
    #[test] fn lexes_numeric_formats() { assert_eq!(lex("1_000 0xff 0b1010").unwrap(), vec![Token::Number(1000),Token::Number(255),Token::Number(10),Token::Eof]); }
    #[test] fn lexes_single_quotes_and_control_keywords() { assert_eq!(lex("local x = 'hi'; repeat\nbreak\nuntil true").unwrap(), vec![Token::Local,Token::Identifier("x".into()),Token::Equal,Token::String("hi".into()),Token::Semicolon,Token::Repeat,Token::Newline,Token::Break,Token::Newline,Token::Until,Token::True,Token::Eof]); }
}
