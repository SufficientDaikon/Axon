// span.rs — Source location tracking for every token and AST node

use serde::Serialize;

/// Represents a position in source code (file, line, column).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Position {
    pub file: String,
    pub line: usize,
    pub column: usize,
}

/// A span in source code (start position to end position).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Span {
    pub start: Position,
    pub end: Position,
}

impl Span {
    pub fn new(file: &str, start_line: usize, start_col: usize, end_line: usize, end_col: usize) -> Self {
        Span {
            start: Position {
                file: file.to_string(),
                line: start_line,
                column: start_col,
            },
            end: Position {
                file: file.to_string(),
                line: end_line,
                column: end_col,
            },
        }
    }

    pub fn dummy() -> Self {
        Span::new("<unknown>", 0, 0, 0, 0)
    }

    /// Merge two spans into one that covers both.
    pub fn merge(&self, other: &Span) -> Span {
        Span {
            start: if (self.start.line, self.start.column) <= (other.start.line, other.start.column) {
                self.start.clone()
            } else {
                other.start.clone()
            },
            end: if (self.end.line, self.end.column) >= (other.end.line, other.end.column) {
                self.end.clone()
            } else {
                other.end.clone()
            },
        }
    }
}

impl std::fmt::Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:{}:{}",
            self.start.file, self.start.line, self.start.column
        )
    }
}
