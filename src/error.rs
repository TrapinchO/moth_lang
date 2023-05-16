#[derive(Debug, PartialEq, Eq, Clone, Copy)]
struct Pos {
    line: usize,
    col: usize,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Error {
    pub msg: String,
    pub lines: Vec<(usize, usize)>,  // start, end INDEXES
}

impl Error {
    pub fn format_message(&self, code: &str) -> String {
        let code_lines = code.lines().collect::<Vec<_>>();
        let last_line = self.lines.iter().map(|x| x.0).max()
            .unwrap_or_else(|| panic!("Expected error position(s);\n{}", self.msg));

        assert!(last_line < code_lines.len(),
                "Error's line ({}) is greater than that of the code ({})",
                last_line,
                code.len()
        );

        let x = self.lines.iter()
            .map(|(start_idx, end_idx)| (pos_from_idx(code, *start_idx), pos_from_idx(code, *end_idx)))
            .map(|(start, end)| format!(
                "{:width$} | {}\n   {:width$}{}{}",
                start.line+1,
                code_lines[start.line],  // line of the code
                "",
                " ".repeat(start.col),
                "^".repeat(end.col - start.col + 1),
                width = last_line.to_string().len())
            ).collect::<Vec<_>>().join("\n");
        format!("Error: {}\n{}", self.msg, x)
    }
}

fn pos_from_idx(code: &str, idx: usize) -> Pos {
    let code = code.chars().collect::<Vec<_>>();
    assert!(idx <= code.len(), "Index {} is higher than code length {}", idx, code.len());

    let mut line = 0;
    let mut col = 0;
    let mut i = 0;
    while i < idx {
        let chr = code[i];
        if chr == '\n' {
            line += 1;
            col = 0;
        } else {
            col += 1;
        }
        i += 1
    }
    Pos { line, col }
}
