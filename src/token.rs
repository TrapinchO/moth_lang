use std::fmt::Display;

use crate::located::Located;

#[derive(Debug, PartialEq, Clone)]
pub enum TokenType {
    Int(i32),
    Float(f32),
    Identifier(String),
    True,
    False,
    Let,
    Fun,
    If,
    Else,
    While,
    String(String),
    Eof,
    Symbol(String),
    LParen,
    RParen,
    LBracket,
    RBracket,
    LBrace,
    RBrace,
    Equals,
    QuestionMark,
    Semicolon,
    Dot,
}

impl TokenType {
    // TODO: better format for non-value types
    fn format(&self) -> String {
        match self {
            Self::Int(n) => n.to_string(),
            Self::String(s) => format!("\"{}\"", s),
            Self::Symbol(s) => s.to_string(),
            Self::Identifier(i) => i.to_string(),
            typ => format!("{:?}", typ),
        }
    }

    pub fn compare_variant(&self, other: &TokenType) -> bool {
        // probably courtesy of https://stackoverflow.com/a/32554326
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}

impl Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.format())
    }
}

pub type Token = Located<TokenType>;
