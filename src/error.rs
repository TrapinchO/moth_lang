#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Error {
    pub msg: String,
    pub lines: Vec<(usize, usize, usize)>,
    //pub line: usize,
    //pub start: usize,
    //pub end: usize,
}

impl Error {
    pub fn format_message(&self, code: Vec<&str>) -> String {
        let last_line = self.lines.iter().map(|x| x.0).max()
            .unwrap_or_else(|| panic!("Expected error position(s);\n{}", self.msg));
        if last_line >= code.len() {
            panic!("Error's line is greater than the code's: {} and {}", last_line, code.len());
        }
        let line_num_len = last_line.to_string().len();
        let x = self.lines.iter().map(|(line, start, end)| format!(
            "{:line_num_len$} | {}\n   {:line_num_len$}{}{}^",
            line,
            code[*line],  // line of the code
            "",
            " ".repeat(*start),
            "-".repeat(end - start),
            line_num_len = line_num_len,
        )).collect::<Vec<_>>().join("\n");
        format!("Error: {}\n{}", self.msg, x)
        //"".to_string()
    }
}
