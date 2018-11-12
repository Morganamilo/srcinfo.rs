use std::error;
use std::fmt;
use std::io;

#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ErrorLine {
    pub number: usize,
    pub line: String,
}

#[derive(Debug)]
pub enum ErrorKind {
    DuplicatePkgbase,
    UndeclaredArch(String, String),
    KeyAfterPkgname(String),
    KeyBeforePkgbase(String),
    MissingField(String),
    EmptyKey,
    EmptyValue(String),
    NotArchSpecific(String),
    IoError(io::Error),
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ErrorKind::DuplicatePkgbase => write!(fmt, "pkgbase already set"),
            ErrorKind::UndeclaredArch(k, a) => {
                write!(fmt, "undeclared arch '{}' in key '{}'", a, k)
            }
            ErrorKind::KeyAfterPkgname(k) => write!(fmt, "key '{}' used after pkgname", k),
            ErrorKind::KeyBeforePkgbase(k) => write!(fmt, "key '{}' used before pkgbase", k),
            ErrorKind::MissingField(f) => write!(fmt, "field '{}' is required", f),
            ErrorKind::EmptyKey => write!(fmt, "field has no key"),
            ErrorKind::EmptyValue(k) => write!(fmt, "key '{}' requires a value", k),
            ErrorKind::NotArchSpecific(k) => {
                write!(fmt, "key '{}' can not be architecture specific", k)
            }
            ErrorKind::IoError(err) => err.fmt(fmt),
        }
    }
}

#[derive(Debug)]
pub struct Error {
    pub kind: ErrorKind,
    pub line: Option<ErrorLine>,
}

impl error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self.line {
            Some(ref line) => write!(fmt, "{}: Line {}: {}", self.kind, line.number, line.line),
            None => write!(fmt, "{}", self.kind),
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        ErrorKind::IoError(err).into()
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Error {
        Error { kind, line: None }
    }
}

impl Error {
    pub fn new<S>(kind: ErrorKind, line: S, number: usize) -> Error
    where
        S: Into<String>,
    {
        let line = line.into();
        let line = Some(ErrorLine { number, line });
        Error {
            line,
            ..Error::from(kind)
        }
    }
}
