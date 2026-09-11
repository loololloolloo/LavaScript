#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Let, Print, If, Else, ElseIf, While, Then, Function, Return, End, True, False, And, Or, Not, Break,
    Identifier(String), Number(i32), String(String),
    Plus, Minus, Star, Slash, Percent,
    Equal, EqualEqual, NotEqual, Less, LessEqual, Greater, GreaterEqual,
    AndAnd, OrOr, Bang,
    LeftParen, RightParen, Comma, Newline, Eof,
}

pub fn lex(source: &str) -> Result<Vec<Token>, String> {
    let chars: Vec<char> = source.chars().collect();
    let mut tokens = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        match chars[i] {
            ' ' | '\t' | '\r' => i += 1,
            '\n' => { tokens.push(Token::Newline); i += 1; }
            '-' if i + 1 < chars.len() && chars[i + 1] == '-' => {
                i += 2;
                while i < chars.len() && chars[i] != '\n' { i += 1; }
            }
            '/' if i + 1 < chars.len() && chars[i + 1] == '/' => {
                i += 2;
                while i < chars.len() && chars[i] != '\n' { i += 1; }
            }
            '"' => {
                i += 1;
                let mut value = String::new();
                while i < chars.len() && chars[i] != '"' {
                    if chars[i] == '\\' {
                        i += 1;
                        if i >= chars.len() { return Err("unterminated string".into()); }
                        match chars[i] {
                            'n' => value.push('\n'), 'r' => value.push('\r'), 't' => value.push('\t'),
                            '"' => value.push('"'), '\\' => value.push('\\'),
                            c => return Err(format!("unsupported escape `\\{c}`")),
                        }
                    } else { value.push(chars[i]); }
                    i += 1;
                }
                if i >= chars.len() { return Err("unterminated string".into()); }
                i += 1;
                tokens.push(Token::String(value));
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
            c if c.is_ascii_digit() => {
                let start = i; while i < chars.len() && chars[i].is_ascii_digit() { i += 1; }
                let value: String = chars[start..i].iter().collect();
                tokens.push(Token::Number(value.parse().map_err(|_| "invalid number")?));
            }
            c if c.is_ascii_alphabetic() || c == '_' => {
                let start = i; while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') { i += 1; }
                let word: String = chars[start..i].iter().collect();
                tokens.push(match word.as_str() {
                    "let" => Token::Let, "print" => Token::Print, "if" => Token::If, "else" => Token::Else,
                    "elseif" => Token::ElseIf, "while" => Token::While, "then" => Token::Then,
                    "function" => Token::Function, "return" => Token::Return, "end" => Token::End,
                    "true" => Token::True, "false" => Token::False, "and" => Token::And, "or" => Token::Or,
                    "not" => Token::Not, "break" => Token::Break, _ => Token::Identifier(word),
                });
            }
            c => return Err(format!("unexpected character `{c}`")),
        }
    }
    tokens.push(Token::Eof);
    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::{lex, Token};
    #[test] fn lexes_booleans_and_logic() { assert_eq!(lex("true and not false || true").unwrap(), vec![Token::True, Token::And, Token::Not, Token::False, Token::OrOr, Token::True, Token::Eof]); }
    #[test] fn lexes_comments() { assert_eq!(lex("print 1 -- hello\nprint 2 // world").unwrap().len(), 6); }
}
