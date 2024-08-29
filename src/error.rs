use std::fmt::Display;

use crate::associativity::Precedence;
use crate::frontend::token::TokenType;
use crate::located::Location;

#[derive(Debug, PartialEq, Clone, Copy)]
struct Pos {
    line: usize,
    col: usize,
}

#[derive(Debug, PartialEq, Clone)]
pub enum ErrorType {
    // lexer
    UnknownCharacter(char),
    StringEol,
    StringEof,
    InvalidEscapeChar(String), // string because of handling escaping special characters
    TwoDecimalPoints,
    InvalidDigit(char),
    IntegerOverflow,
    FloatOverflow,
    CommentEof,
    CommentSymbol,
    // parser
    ExpectedSemicolon,
    ExpectedToken(String),
    UnknownElement(TokenType),
    UnknownUnaryOperator,
    InvalidAssignmentTarget,
    InvalidPrecedence,
    PrecedenceOutOfRange(i32),
    IncorrectOperatorParameterCount(usize),
    InvalidFunctionName,
    InvalidOperatorname,
    ExpectedOpeningToken(TokenType),
    ExpectedClosingToken(TokenType),
    //ExpectedParameterName,
    ExpectedIdentifier,
    ExpectedStructName,
    ExpectedFieldName,
    UnexpectedEof,
    // reassoc
    OperatorNotFound(String),
    IncompatiblePrecedence(String, Precedence, String, Precedence),
    // varcheck
    AlreadyDeclaredItem,
    UndeclaredItem,
    DuplicateParameter(String),
    DuplicateField(String),
    // varcheck warns
    ItemNotUsed(String),
    DeadCode,
    IfNeverExecutes,
    LoopNeverExecutes,
    // interpreter
    ExpectedListIndex,
    ExpectedIndex,
    IndexOutOfRange(i32, usize), // tried, max
    ExpectedBool,
    ItemNotCalleable,
    ExpectedUnaryNumber,
    ExpectedUnaryBool,
    ItemNotIndexable,
    IncorrectParameterCount(usize, usize), // args, paramas
    ReturnOutsideFunction,
    BreakOutsideLoop,
    ContinueOutsideLoop,
    NativeFunctionError(String),
    FieldNotFound(String, String),
    ExpectedInstance,
    UnknownField(String),
    // other
    OtherError(String),
}

impl ErrorType {
    pub fn msg(&self) -> String {
        match self {
            // lexer
            Self::UnknownCharacter(c) => format!("Unknown character: \"{c}\""),
            Self::StringEol => "EOL while parsing string".to_string(),
            Self::StringEof => "EOF while parsing string".to_string(),
            Self::InvalidEscapeChar(c) => format!("Invalid escape character: \"\\{c}\""),
            Self::TwoDecimalPoints => "Found two decimal delimiters".to_string(),
            Self::InvalidDigit(c) => format!("Invalid digit: \"{c}\""),
            Self::IntegerOverflow => "Integer overflow".to_string(),
            Self::FloatOverflow => "Float overflow".to_string(),
            Self::CommentEof => "EOF while parsing block comment".to_string(),
            Self::CommentSymbol => "Block comment ending cannot be an operator".to_string(),
            // parser
            Self::ExpectedSemicolon => "Expected a semicolon".to_string(),
            Self::ExpectedToken(msg) => msg.clone(),
            Self::UnknownElement(tok) => format!("Unknown element: {tok}"),
            Self::UnknownUnaryOperator => "Unary operators must be either \"-\" or \"!\"".to_string(),
            Self::InvalidAssignmentTarget => "The left side of an assignment must be either a variable or an index".to_string(),
            Self::InvalidPrecedence => "Precendence value must be an integer".to_string(),
            Self::PrecedenceOutOfRange(n) => format!("Precedence value must be between 0 and 10, got: {n}"),
            Self::IncorrectOperatorParameterCount(n) => format!("Operator declaration must have exactly two parameters, got {n}"),
            Self::InvalidFunctionName => "Function name must be either an identifier or a valid symbol".to_string(),
            Self::InvalidOperatorname => "Operator name must be a valid symbol".to_string(),
            Self::ExpectedOpeningToken(tok) => format!("Expected opening token {tok}"),
            Self::ExpectedClosingToken(tok) => format!("Expected closing token {tok}"),
            Self::UnexpectedEof => "Expected an element but reached EOF".to_string(),
            //Self::ExpectedParameterName => "Expected a parameter name".to_string(),
            Self::ExpectedIdentifier => "Expected an identifier".to_string(),
            Self::ExpectedFieldName => "Expected a field name".to_string(),
            Self::ExpectedStructName => "Expected a struct name".to_string(),
            // reassoc
            Self::OperatorNotFound(s) => format!("Operator not found: {s}"),
            Self::IncompatiblePrecedence(op1, prec1, op2, prec2) => format!("Incompatible operator precedence: \"{op1}\" ({prec1:?}) and \"{op2}\" ({prec2:?}) - both have precedence {}", prec1.prec),
            // varcheck
            Self::AlreadyDeclaredItem => "Item already declared".to_string(),
            Self::UndeclaredItem => "Item not declared".to_string(),
            Self::DuplicateParameter(s) => format!("Duplicate parameter: {s}"),
            Self::DuplicateField(f) => format!("Duplicate parameter: {f}"),
            // varcheck warns
            Self::ItemNotUsed(s) => format!("Item \"{s}\" not used"),
            Self::DeadCode => "Unreachable code".to_string(),
            Self::IfNeverExecutes => "If branch never executes".to_string(),
            Self::LoopNeverExecutes => "Loop never executes".to_string(),
            // interpreter
            Self::ExpectedListIndex => "Expected a list index expression".to_string(),
            Self::ExpectedIndex => "Expected an integer index".to_string(),
            Self::IndexOutOfRange(n, len) => format!("Index out of range: {n} (length {len})"),
            Self::ExpectedBool => "Expected bool in a condition".to_string(),
            Self::ItemNotCalleable => "Item is not calleable".to_string(),
            Self::ExpectedUnaryNumber => "Expected a number to negate".to_string(),
            Self::ExpectedUnaryBool => "Expected a bool to negate".to_string(),
            Self::ItemNotIndexable => "Item is not indexable".to_string(),
            Self::IncorrectParameterCount(n, max) => format!("The number of arguments ({n}) must match the number of parameters ({max})"),
            Self::ReturnOutsideFunction => "Cannot use return outside of a function".to_string(),
            Self::BreakOutsideLoop => "Cannot use break outside of a loop".to_string(),
            Self::ContinueOutsideLoop => "Cannot use continue outside of a loop".to_string(),
            Self::NativeFunctionError(msg) => msg.clone(),
            Self::FieldNotFound(field, struc) => format!("Field \"{field}\" not found in \"{struc}\""),
            Self::ExpectedInstance => "Expected struct instance".to_string(),
            Self::UnknownField(name) => format!("Field \"{name}\" does not exist"),
            // other
            Self::OtherError(msg) => msg.clone(),
        }
    }

    pub fn is_warn(&self) -> bool {
        // done for clarity
        // also more items will follow eventually
        #[allow(clippy::match_like_matches_macro)]
        match self {
            Self::ItemNotUsed(_)
            | Self::DeadCode
            | Self::IfNeverExecutes
            | Self::LoopNeverExecutes => true,
            _ => false,
        }
    }
}

impl Display for ErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // f.alternate();
        write!(f, "{}", self.msg())
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Error {
    pub msg: ErrorType,
    pub lines: Vec<Location>,
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
            .unwrap_or_else(|| panic!("Expected error position(s);\nMessage: {}", self.msg.msg()));
        // otherwise it would consider the 10th line as 9th, thus one less character for padding
        // see commit d86b034
        let width = (last_line + 1).to_string().len();

        assert!(
            last_line < code_lines.len(),
            "Error's line ({last_line}) is greater or equal than that of the code ({})",
            code_lines.len()
        );

        let lines = lines
            .iter()
            .map(|(start, end)| {
                if start.line == end.line {
                    format!(
                        "{:width$} | {}\n   {padding}{underline}",
                        start.line + 1,
                        code_lines[start.line], // line of the code; doesnt work with tabs
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
        let warn = if self.msg.is_warn() { "Warning" } else { "Error" };
        format!("{warn}: {}\n{lines}", self.msg)
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
