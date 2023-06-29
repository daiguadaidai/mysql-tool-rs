use std::fmt::{Display, Formatter, Result as FmtResult};

use std::io::Error as IoError;
use std::num::ParseIntError;

#[derive(Debug)]
pub enum CustomError {
    NormalError(String),
    ParseError(ParseIntError),
    ReadError(IoError),
    SQLXError(sqlx::Error),
    ConfigErrors(log4rs::config::runtime::ConfigErrors),
    SetLoggerError(log::SetLoggerError),
}

impl std::error::Error for CustomError {}

impl CustomError {
    pub fn new(s: String) -> CustomError {
        CustomError::NormalError(s)
    }
}

impl Display for CustomError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match *self {
            CustomError::NormalError(ref e) => e.fmt(f),
            CustomError::ParseError(ref e) => e.fmt(f),
            CustomError::ReadError(ref e) => e.fmt(f),
            CustomError::SQLXError(ref e) => e.fmt(f),
            CustomError::ConfigErrors(ref e) => e.fmt(f),
            CustomError::SetLoggerError(ref e) => e.fmt(f),
        }
    }
}

//将IoError转为IdError::ReadError
impl From<IoError> for CustomError {
    fn from(error: IoError) -> CustomError {
        CustomError::ReadError(error)
    }
}

//将ParseIntError转为IdError::ParseError
impl From<ParseIntError> for CustomError {
    fn from(error: ParseIntError) -> CustomError {
        CustomError::ParseError(error)
    }
}

//将SqlxError转为 CustomError
impl From<sqlx::Error> for CustomError {
    fn from(error: sqlx::Error) -> CustomError {
        CustomError::SQLXError(error)
    }
}

//将SqlxError转为 CustomError
impl From<String> for CustomError {
    fn from(error: String) -> CustomError {
        CustomError::NormalError(error)
    }
}

//将ConfigErrors转为 CustomError
impl From<log4rs::config::runtime::ConfigErrors> for CustomError {
    fn from(error: log4rs::config::runtime::ConfigErrors) -> CustomError {
        CustomError::ConfigErrors(error)
    }
}

//将SetLoggerError转为 CustomError
impl From<log::SetLoggerError> for CustomError {
    fn from(error: log::SetLoggerError) -> CustomError {
        CustomError::SetLoggerError(error)
    }
}
