use std::fmt;

#[derive(Debug, PartialEq, Eq)]
pub enum CompileError {
    UnmatchedOpen { line: usize, col: usize },
    UnmatchedClose { line: usize, col: usize },
    Io(String),
}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompileError::UnmatchedOpen { line, col } => {
                write!(f, "unmatched '[' at line {line}, column {col}")
            }
            CompileError::UnmatchedClose { line, col } => {
                write!(f, "unmatched ']' at line {line}, column {col}")
            }
            CompileError::Io(msg) => {
                write!(f, "I/O error: {msg}")
            }
        }
    }
}

impl std::error::Error for CompileError {}

impl From<std::io::Error> for CompileError {
    fn from(e: std::io::Error) -> Self {
        CompileError::Io(e.to_string())
    }
}

pub type Result<T> = std::result::Result<T, CompileError>;
