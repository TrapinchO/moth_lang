#[derive(Debug, PartialEq, Eq, Clone, Copy)]
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

}
*/

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Error {
    pub msg: String,
    pub lines: Vec<(usize, usize)>, // start, end INDEXES
}

impl Error {
    pub fn format_message(&self, code: &str) -> String {
        let code_lines = code.lines().collect::<Vec<_>>();
        let lines = self
            .lines
            .iter()
            .map(|(start_idx, end_idx)| (pos_from_idx(code, *start_idx), pos_from_idx(code, *end_idx)))
            .collect::<Vec<_>>();
        let last_line = lines
            .iter()
            .map(|x| x.0.line)
            .max()
            .unwrap_or_else(|| panic!("Expected error position(s);\n{}", self.msg));
        let width = last_line.to_string().len();

        assert!(
            last_line < code_lines.len(),
            "Error's line ({}) is greater than that of the code ({})",
            last_line,
            code_lines.len()
        );

        let lines = lines
            .iter()
            .map(|(start, end)| {
                if start.line == end.line {
                    format!(
                        "{:width$} | {}\n   {}{}",
                        start.line + 1,
                        code_lines[start.line],        // line of the code; doesnt work with tabs
                        " ".repeat(width + start.col), // align it properly
                        "^".repeat(end.col - start.col + 1),
                        width = width
                    )
                } else {
                    let mut s: Vec<String> = vec![];
                    let line = code_lines[start.line];
                    s.push(format!(
                        "{:width$} | {}\n   {}{}",
                        start.line + 1,
                        line, // line of the code
                        " ".repeat(width + start.col),
                        "^".repeat(line.len() - start.col),
                        width = width
                    ));
                    for (i, line) in code_lines[start.line + 1..end.line].iter().enumerate() {
                        s.push(format!(
                            "{:width$} | {}\n   {}{}",
                            i + 1,
                            line, // line of the code
                            " ".repeat(width),
                            "^".repeat(line.len()),
                            width = width
                        ));
                    }
                    let line = code_lines[end.line];
                    s.push(format!(
                        "{:width$} | {}\n   {}{}",
                        end.line + 1,
                        line, // line of the code
                        " ".repeat(width),
                        "^".repeat(end.col + 1),
                        width = width
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
        "Index {} is higher than code length {}",
        idx,
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
