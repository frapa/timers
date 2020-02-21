#[derive(Debug)]
pub struct ValueError {
    description: String
}

impl std::fmt::Display for ValueError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Value error: {}", self.description)
    }
}

impl std::error::Error for ValueError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> { None }
}

impl ValueError {
    pub fn new(description: &str) -> ValueError {
        ValueError { description: description.to_string() }
    }
}

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Value(ValueError),
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::Io(err)
    }
}

impl From<ValueError> for Error {
    fn from(err: ValueError) -> Error {
        Error::Value(err)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::Io(err) => err.fmt(f),
            Error::Value(err) => err.fmt(f),
        }
    }
}

impl std::error::Error for Error {
    fn cause(&self) -> Option<&dyn std::error::Error> {
        match self {
            Error::Io(err) => err.source(),
            Error::Value(err) => err.source(),
        }
    }
}
