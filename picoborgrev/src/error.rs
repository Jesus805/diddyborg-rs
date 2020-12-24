use std::fmt;

#[cfg(any(target_os = "linux", target_os = "android"))]
use i2cdev::linux;

#[derive(Debug)]
pub enum DiddyBorgError {
    #[cfg(any(target_os = "linux", target_os = "android"))]
    I2CError(LinuxI2CError),
    InvalidPicoBorgDataError(String),
    InvalidIdError(String)
}

impl fmt::Display for DiddyBorgError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            #[cfg(any(target_os = "linux", target_os = "android"))]
            DiddyBorgError::I2CError(ref e) => fmt::Display(e, f),
            DiddyBorgError::InvalidIdError(ref e) => "Invalid PicoBorgRev ID",
            DiddyBorgError::InvalidPicoBorgDataError(ref e) => "",
        }
    }
}