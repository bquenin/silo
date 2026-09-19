use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("unexpected end of file at offset {offset}")]
    Eof { offset: u64 },

    #[error("invalid magic: expected KW replay header, got {got:?}")]
    BadMagic { got: String },

    #[error("invalid string encoding at offset {offset}: {message}")]
    BadString { offset: u64, message: String },

    #[error("header parse error: {0}")]
    BadHeader(String),

    #[error("replay {field} length {length} exceeds limit {limit}")]
    LimitExceeded {
        field: &'static str,
        length: usize,
        limit: usize,
    },

    #[error("command stream error: {0}")]
    BadBody(String),

    #[error("unsupported game: {0}")]
    UnsupportedGame(String),
}

pub type Result<T> = std::result::Result<T, ParseError>;
