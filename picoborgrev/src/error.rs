use std::fmt::{Display, Formatter, Result};
use std::error::Error;

/// ## Summary
/// 
/// A DiddyBorg error.
/// 
#[derive(Debug)]
pub enum DiddyBorgError<T> where T: Error {
    // An error occured when trying to read from I2C.
    I2C(T),
    // Invalid Data received.
    CorruptedData,
    // A PicoBorg Reverse could not be found with the given I2C address.
    NotFound,
}

impl<T: Error> Display for DiddyBorgError<T> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            DiddyBorgError::I2C(_) => write!(f, "I2C error occured"),
            DiddyBorgError::CorruptedData => write!(f, "Corrupted Data Received"),
            DiddyBorgError::NotFound => write!(f, "Invalid PicoBorgRev ID"),
        }
    }
}

impl<T: Error + 'static> Error for DiddyBorgError<T> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            DiddyBorgError::I2C(e) => Some(e),
            _ => None,
        }
    }
}