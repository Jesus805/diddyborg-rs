use std::path::Path;

use i2cdev::linux::{LinuxI2CDevice, LinuxI2CError};

use crate::diddyborg::DiddyBorg;
use crate::error::DiddyBorgError;

// PicoBorg peripheral ID.
const I2C_ID_PICOBORG_REV: u8 = 0x15;

impl DiddyBorg<LinuxI2CDevice> {
    /// ## Summary
    /// 
    /// Initialize a new DiddyBorg instance.
    /// 
    /// ## Parameters
    /// 
    /// path: Path to the I2C file.
    /// 
    /// device_address: The I2C address of the peripheral.
    /// 
    /// ## Example
    /// 
    /// ```no_run
    /// # use picoborgrev::diddyborg::DiddyBorg;
    /// 
    /// let mut driver = DiddyBorg::new("/dev/i2c-1", 0x44).unwrap();
    /// ```
    /// 
    pub fn new<P: AsRef<Path>>(path: P, device_address: u16) -> Result<Self, DiddyBorgError<LinuxI2CError>> {
        let mut dev;

        // Try to create a new I2C peripheral.
        match LinuxI2CDevice::new(path, device_address) {
            Ok(d) => dev = d,
            // Unable to create a new I2C peripheral.
            Err(error) => return Err(DiddyBorgError::I2C(error)),
        }

        // Ensure that the device is a Diddyborg.
        DiddyBorg::get_diddyborg_id(&mut dev).and_then(|id| {
            if id == I2C_ID_PICOBORG_REV {
                // The device is a DiddyBorg.
                Ok(DiddyBorg::internal_new(dev))
            } else {
                // The device is not a DiddyBorg.
                Err(DiddyBorgError::NotFound)
            }
        })
    }
}
