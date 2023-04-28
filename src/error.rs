#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Error {
    pub msg: String,
    pub line: usize,
    pub start: usize,
    pub end: usize,
}

impl Error {
    pub fn format_message(&self, line: &str) -> String {
        format!(
            "Error on line {}:\n{}\n{}{}^ {}",
            self.line,
            line,
            " ".repeat(self.start),
            "-".repeat(self.end - self.start),
            self.msg
        )
    }
}
