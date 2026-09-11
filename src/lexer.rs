#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Let, Print, If, Else, While, Then, Function, Return, End,
    Identifier(String), Number(i32), Plus, Minus, Star, Slash,
    Equal, EqualEqual, NotEqual, Less, LessEqual, Greater, GreaterEqual,
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
            '=' => { i += 1; if i < chars.len() && chars[i] == '=' { tokens.push(Token::EqualEqual); i += 1; } else { tokens.push(Token::Equal); } }
            '!' => { i += 1; if i < chars.len() && chars[i] == '=' { tokens.push(Token::NotEqual); i += 1; } else { return Err("unexpected character `!`".into()); } }
            '<' => { i += 1; if i < chars.len() && chars[i] == '=' { tokens.push(Token::LessEqual); i += 1; } else { tokens.push(Token::Less); } }
            '>' => { i += 1; if i < chars.len() && chars[i] == '=' { tokens.push(Token::GreaterEqual); i += 1; } else { tokens.push(Token::Greater); } }
            '+' => { tokens.push(Token::Plus); i += 1; }
            '-' => { tokens.push(Token::Minus); i += 1; }
            '*' => { tokens.push(Token::Star); i += 1; }
            '/' => { tokens.push(Token::Slash); i += 1; }
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
                    "while" => Token::While, "then" => Token::Then, "function" => Token::Function,
                    "return" => Token::Return, "end" => Token::End, _ => Token::Identifier(word),
                });
            }
            c => return Err(format!("unexpected character `{c}`")),
        }
    }
    tokens.push(Token::Eof);
    Ok(tokens)
}
