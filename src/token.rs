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
    Return,
    If,
    Else,
    While,
    Break,
    Continue,
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
    Comma,
}

impl TokenType {
    fn format(&self) -> String {
        match self {
            Self::Int(n) => n.to_string(),
            Self::Float(n) => n.to_string(),
            Self::String(s) => format!("\"{}\"", s),
            Self::Symbol(s) => s.to_string(),
            Self::Identifier(i) => i.to_string(),
            typ => format!("{:?}", typ),
        }
    }

    fn variant_str(&self) -> String {
        match self {
            Self::Int(_) => "Int".to_string(),
            Self::String(_) => "String".to_string(),
            Self::Symbol(_) => "Symbol".to_string(),
            Self::Identifier(_) => "Identifier".to_string(),
            typ => format!("{:?}", typ),
        }
    }

    // TODO: does NOT work if other is misspelled or wrong case
    pub fn compare_variant_str(&self, other: String) -> bool {
        self.variant_str() == other
    }
}

impl Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.format())
    }
}

pub type Token = Located<TokenType>;

