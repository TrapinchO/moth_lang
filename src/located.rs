use std::fmt::Display;

/// a struct to hold a value and its position within the source code
#[derive(Debug, Clone, PartialEq)]
pub struct Located<T> {
    pub val: T,
    pub start: usize,
    pub end: usize,
}

impl<T> Located<T> {
    pub fn new(val: T, start: usize, end: usize) -> Self {
        Located { val, start, end }
    }
}

impl<T: std::fmt::Display> Display for Located<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.val)
    }
}
