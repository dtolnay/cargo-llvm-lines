use std::fmt::{self, Display};
use std::io;
use std::path::PathBuf;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Msg(&'static str),
    Io(io::Error),
    PathIo(PathBuf, io::Error),
    Quote(shlex::QuoteError),
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::Io(error)
    }
}

impl From<shlex::QuoteError> for Error {
    fn from(error: shlex::QuoteError) -> Self {
        Error::Quote(error)
    }
}

impl Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Msg(msg) => formatter.write_str(msg),
            Error::Io(e) => Display::fmt(e, formatter),
            Error::PathIo(path, e) => write!(formatter, "{}: {}", path.display(), e),
            Error::Quote(e) => Display::fmt(e, formatter),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Msg(_) => None,
            Error::Io(e) => e.source(),
            Error::PathIo(_path, e) => e.source(),
            Error::Quote(e) => e.source(),
        }
    }
}
