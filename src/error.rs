//use crate::{token::{TokenType}, reassoc::Precedence};

use crate::{located::Location, value::Value};

#[derive(Debug, PartialEq, Clone, Copy)]
struct Pos {
    line: usize,
    col: usize,
}

/*
#[derive(Debug, PartialEq)]
enum ErrorType {
    // lexer
    UnknownCharacter,
    StringEol,
    StringEoF,
    TwoDecimalPoints,
    InvalidDigit(char),
    IntegerOverflow,
    CommentEof,
    // parser
    ExpectedSemicolon,
    ExpectedLBrace,
    ExpectedRBrace,
    ExpectedToken(TokenType),
    // reassoc
    NotASymbol,
    IncompatiblePrecedence(Precedence, Precedence),
    // varcheck
    UndeclaredVariable(String),
}
*/

#[derive(Debug)]
pub enum ErrorType {
    Error(Error),
    Return(Value),
    Continue,
    Break,
}
// a miracle
impl From<Error> for ErrorType {
    fn from(value: Error) -> Self {
        ErrorType::Error(value)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Error {
    pub msg: String,
    pub lines: Vec<Location>, // start, end INDEXES
}

impl Error {
    pub fn format_message(&self, code: &str) -> String {
        let code_lines = code.lines().collect::<Vec<_>>();
        let lines = self
            .lines
            .iter()
            .map(|loc| (pos_from_idx(code, loc.start), pos_from_idx(code, loc.end)))
            .collect::<Vec<_>>();
        let last_line = lines
            .iter()
            .map(|x| x.0.line)
            .max()
            .unwrap_or_else(|| panic!("Expected error position(s);\nMessage: {}", self.msg));
        // otherwise it would consider the 10th line as 9th, thus one less character for padding
        // see commit d86b034 
        let width = (last_line+1).to_string().len();

        assert!(
            last_line < code_lines.len(),
            "Error's line ({last_line}) is greater than that of the code ({})",
            code_lines.len()
        );

        let lines = lines
            .iter()
            .map(|(start, end)| {
                if start.line == end.line {
                    format!(
                        "{:width$} | {}\n   {padding}{underline}",
                        start.line + 1,
                        code_lines[start.line],        // line of the code; doesnt work with tabs
                        padding = " ".repeat(width + start.col), // align it properly
                        underline = "^".repeat(end.col - start.col + 1),
                    )
                } else {
                    let mut s: Vec<String> = vec![];
                    let line = code_lines[start.line];
                    s.push(format!(
                        "{:width$} | {line}\n   {}{}",
                        start.line + 1,
                        " ".repeat(width + start.col),
                        "^".repeat(line.len() - start.col),
                        width = width
                    ));
                    // note to the future me:
                    // the for highlights the ENTIRE LINE, when the error spans one the one line
                    for (i, line) in code_lines[start.line + 1..end.line].iter().enumerate() {
                        s.push(format!(
                            "{:width$} | {line}\n   {}{}",
                            i + 1,
                            " ".repeat(width),
                            "^".repeat(line.len()),
                        ));
                    }
                    let line = code_lines[end.line];
                    s.push(format!(
                        "{:width$} | {line}\n   {}{}",
                        end.line + 1,
                        " ".repeat(width),
                        "^".repeat(end.col + 1),
                    ));
                    s.join("\n")
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
        format!("Error: {}\n{}", self.msg, lines)
    }
}

fn pos_from_idx(code: &str, idx: usize) -> Pos {
    let code = code.chars().collect::<Vec<_>>();
    assert!(
        idx <= code.len(),
        "Index {idx} is higher than code length {}",
        code.len()
    );

    let mut line = 0;
    let mut col = 0;
    for chr in code.iter().take(idx) {
        if *chr == '\n' {
            line += 1;
            col = 0;
        } else {
            col += 1;
        }
    }
    Pos { line, col }
}
