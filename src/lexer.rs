#[derive(Debug, PartialEq, Eq)]
pub enum TokenType {
    Plus,
    Minus,
    Star,
    Slash,
    Number(i32),
    Identifier(String),
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
        tokens.push(Token {
            pos,
            line,
            typ: match chars[idx] {
                '+' => TokenType::Plus,
                '-' => TokenType::Minus,
                '*' => TokenType::Star,
                '/' => TokenType::Slash,
                ' ' => {
                    idx += 1;
                    continue
                }
                '\n' => {
                    line += 1;
                    pos = 0;
                    idx += 1;
                    continue;
                }
                n if n.is_digit(10) => {
                    let (num, idx2) = lex_number(&code[idx..]);
                    idx += idx2-1;
                    pos += (idx2-1) as i32;
                    TokenType::Number(num)
                },
                s if s.is_alphanumeric() => {
                    let (string, idx2) = lex_identifier(&code[idx..]);
                    idx += idx2-1;
                    pos += (idx2-1) as i32;
                    TokenType::Identifier(string)
                },
                unknown => panic!("Unknown character: {} at pos {} on line {}", unknown, pos, line)
        }});
        idx += 1;
        pos += 1;
    }

    tokens.push(Token {
        pos,
        line,
        typ: TokenType::Eof
    });
    tokens
}

fn lex_number(code: &str) -> (i32, usize) {
    let mut num = String::from("");

    let chars: Vec<char> = code.chars().collect();
    let mut idx = 0;
    while idx < code.len() {
        if chars[idx].is_digit(10) {
            num.push(chars[idx]);
        } else {
            break;
        }
        idx += 1;
    }
    (num.parse::<i32>().unwrap(), idx)
}

fn lex_identifier(code: &str) -> (String, usize) {
    let mut s = String::from("");

    let chars: Vec<char> = code.chars().collect();
    let mut idx = 0;
    while idx < code.len() {
        if chars[idx].is_alphanumeric() {
            s.push(chars[idx]);
        } else {
            break;
        }
        idx += 1;
    }
    (s, idx)
}
