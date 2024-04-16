use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Location {
    pub start: usize,
    pub end: usize,
}

/// a struct to hold a value and its position within the source code
#[derive(Debug, Clone, PartialEq)]
pub struct Located<T> {
    pub val: T,
    pub loc: Location,
}

impl<T: std::fmt::Display> Display for Located<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.val)
    }
}
