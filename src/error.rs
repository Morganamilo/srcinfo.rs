use std::error;
use std::fmt;
use std::io;

/// Error Line holds a line of text and the line number the line is from.
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ErrorLine {
    /// The line number that the  error occurred at
    pub number: usize,
    /// The full line containing the error
    pub line: String,
}

/// A list of possible errors that may occur when parsing a .SRCINFO.
///
/// Variants that hold a string hold the key that caused the error.
///
/// UndeclaredArch holds the key that caused the error and the architecture.
///
/// IoError holds the underlying IO::Error.
#[derive(Debug)]
pub enum ErrorKind {
    /// pkgbase was specified more than once
    DuplicatePkgbase,
    /// An architecture specific field was declared using an architecture
    /// that has not been declared
    UndeclaredArch(String, String),
    /// A key that must be used inside of the pkgbase section was used
    /// inside of a pkgname section
    KeyAfterPkgname(String),
    /// A key that must be used inside of a pkgname section was used
    /// inside of the pkgbase section
    KeyBeforePkgbase(String),
    /// A required field is missing
    MissingField(String),
    /// A line has an empty key. E.g. " = foo"
    EmptyKey,
    /// A line has an empty value where a value is required. E.g. "foo = "
    EmptyValue(String),
    /// An architecture specific field was declared on a field that can not
    /// be architecture specific
    NotArchSpecific(String),
    /// An IoError occurred
    IoError(io::Error),
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ErrorKind::DuplicatePkgbase => write!(fmt, "pkgbase already set"),
            ErrorKind::UndeclaredArch(k, a) => {
                write!(fmt, "undeclared architecture '{}' in key '{}'", a, k)
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

/// The error type for .SRCINFO parsing.
#[derive(Debug)]
pub struct Error {
    /// The kind of Error that occurred
    pub kind: ErrorKind,
    /// The line where the error occurred
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
    /// Create a new Error from a given ErrorKind and ErrorLine.
    ///
    /// If the line is none then Errors can be created using the the From/Into traits.
    pub fn new<S: Into<String>>(kind: ErrorKind, line: S, number: usize) -> Error {
        let line = line.into();
        let line = Some(ErrorLine { number, line });
        Error {
            line,
            ..Error::from(kind)
        }
    }
}
