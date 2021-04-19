use std::{error::Error, fmt, num::ParseFloatError, str::ParseBoolError};

#[derive(Debug)]
pub enum XdgParseError {
    ParseBoolError(ParseBoolError),
    ParseFloatError(ParseFloatError),
    Other(&'static str),
}

impl From<ParseBoolError> for XdgParseError {
    fn from(e: ParseBoolError) -> Self {
        XdgParseError::ParseBoolError(e)
    }
}

impl From<ParseFloatError> for XdgParseError {
    fn from(e: ParseFloatError) -> Self {
        XdgParseError::ParseFloatError(e)
    }
}

impl From<&'static str> for XdgParseError {
    fn from(e: &'static str) -> Self {
        XdgParseError::Other(e)
    }
}

impl fmt::Display for XdgParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            XdgParseError::ParseBoolError(e) => e.fmt(f),
            XdgParseError::ParseFloatError(e) => e.fmt(f),
            XdgParseError::Other(s) => write!(f, "{}", s),
        }
    }
}

impl Error for XdgParseError {}
