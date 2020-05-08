use std::fmt;
use std::io::ErrorKind;
use std::result::Result as StdResult;

/// Either a parsing error or an IO error.
pub enum IoError {
    /// A phpser parsing error
    Phpser(Error),
    /// An IO error, excluding unexpected EOF error
    Io(std::io::Error),
}

impl From<Error> for IoError {
    fn from(err: Error) -> Self {
        Self::Phpser(err)
    }
}
impl From<std::io::Error> for IoError {
    fn from(err: std::io::Error) -> Self {
        if err.kind() == ErrorKind::UnexpectedEof {
            Self::Phpser(Error::UnexpectedEof)
        } else {
            Self::Io(err)
        }
    }
}

/// A parsing error.
#[derive(Debug, Clone, Copy)]
pub enum Error {
    /// unexpected end of document
    UnexpectedEof,
    /// str is used as string type, but serialized input is not valid UTF-8
    BadEncoding(usize),
    /// encountered invalid token
    BadToken(usize),
    /// encountered malformed or out-of-range number
    BadNumber(usize),
    /// array key must be int or string
    BadArrayKeyType(usize),
    /// object key must be string
    BadObjectKeyType(usize),
}

impl Error {
    /// Returns the offset of this error, if relevant.
    pub fn offset(self) -> Option<usize> {
        match self {
            Self::UnexpectedEof => None,
            Self::BadEncoding(offset) => Some(offset),
            Self::BadToken(offset) => Some(offset),
            Self::BadNumber(offset) => Some(offset),
            Self::BadArrayKeyType(offset) => Some(offset),
            Self::BadObjectKeyType(offset) => Some(offset),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::UnexpectedEof => write!(f, "unexpected end of document"),
            Self::BadEncoding(_) => write!(
                f,
                "str is used as string type, but serialized input is not valid UTF-8"
            ),
            Self::BadToken(_) => write!(f, "encountered invalid token"),
            Self::BadNumber(_) => write!(f, "encountered malformed or out-of-range number"),
            Self::BadArrayKeyType(_) => write!(f, "array key must be int or string"),
            Self::BadObjectKeyType(_) => write!(f, "object key must be string"),
        }?;
        if let Some(offset) = self.offset() {
            write!(f, " at offset {}", offset)?;
        }
        Ok(())
    }
}

/// A parsing result.
pub type Result<T = (), E = Error> = StdResult<T, E>;

/// Result with a parsing error or an IO error.
pub type IoResult<T = ()> = Result<T, IoError>;
