#[derive(Debug, PartialEq, Eq)]
pub enum TokenType {
    Plus,
    Minus,
    Star,
    Slash,
    Number(i32),
    Identifier(String),
    String(String),
    Eof,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Token {
    pub pos: i32,
    pub line: i32,
    pub typ: TokenType,
}

pub fn lex(code: &str) -> Vec<Token> {
    let mut tokens = vec![];
    let chars: Vec<char> = code.chars().collect();

    let mut idx = 0;
    let mut line = 1;
    let mut pos = 0;
    while idx < code.len() {
        let typ = match chars[idx] {
            '+' => TokenType::Plus,
            '-' => TokenType::Minus,
            '*' => TokenType::Star,
            '/' => TokenType::Slash,
            ' ' => {
                idx += 1;
                continue;
            }
            '\n' => {
                line += 1;
                pos = 0;
                idx += 1;
                continue;
            }
            num if num.is_digit(10) => match lex_number(&code[idx..]) {
                Ok((num, idx2)) => {
                    idx += idx2 - 1;
                    pos += (idx2 - 1) as i32;
                    TokenType::Number(num)
                }
                Err(err) => panic!("{}", err),
            },
            ident if ident.is_alphanumeric() => {
                let (ident, idx2) = lex_identifier(&code[idx..]);
                idx += idx2 - 1;
                pos += (idx2 - 1) as i32;
                TokenType::Identifier(ident)
            }
            // +1 to ignore the quote
            '\"' => match lex_string(&code[idx+1..]) {
                Ok((string, idx2)) => {
                    idx += idx2 - 1;
                    pos += (idx2 - 1) as i32;
                    TokenType::String(string)
                }
                Err(err) => panic!("{}", err),
            },
            unknown => panic!(
                "[MOTH] Unknown character: \"{}\" at pos {} on line {}",
                unknown, pos, line
            ),
        };
        tokens.push(Token { pos, line, typ });
        idx += 1;
        pos += 1;
    }

    tokens.push(Token {
        pos,
        line,
        typ: TokenType::Eof,
    });
    tokens
}

fn lex_number(code: &str) -> Result<(i32, usize), String> {
    let mut num = String::from("");

    let chars: Vec<char> = code.chars().collect();
    let mut idx = 0;
    while idx < code.len() {
        if chars[idx].is_digit(10) {
            num.push(chars[idx]);
        } else if chars[idx].is_alphabetic() {
            return Err(format!("[MOTH] Invalid digit: \"{}\"", chars[idx]));
        } else {
            break;
        }
        idx += 1;
    }
    Ok((num.parse::<i32>().unwrap(), idx))
}

fn lex_identifier(code: &str) -> (String, usize) {
    let mut s = String::from("");

    let chars: Vec<char> = code.chars().collect();
    let mut idx = 0;
    while idx < code.len() {
        if !chars[idx].is_alphanumeric() {
            break;
        }
        s.push(chars[idx]);
        idx += 1;
    }
    (s, idx)
}

fn lex_string(code: &str) -> Result<(String, usize), String> {
    let mut s = String::from("");

    let chars: Vec<char> = code.chars().collect();
    let mut idx = 0;
    while idx < code.len() {
        if chars[idx] == '\"' {
            // +2 to move after the closing quote
            return Ok((s, idx+2));
        }
        if chars[idx] == '\n' {
            return Err("[MOTH] EOL while parsing string".to_string());
        }
        s.push(chars[idx]);
        idx += 1;
    }
    Err("[MOTH] EOF while parsing string".to_string())
}
