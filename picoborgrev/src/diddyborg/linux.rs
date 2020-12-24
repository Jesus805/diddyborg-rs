use super::{DiddyBorg, DiddyBorgError, I2C_READ_LEN};
use i2cdev::linux::LinuxI2CDevice;
use std::path::Path;

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
    /// # use diddyborg::DiddyBorg;
    /// 
    /// let mut driver = DiddyBorg::new("/dev/i2c-1", 0x44);
    /// ```
    /// 
    /// ## Errors
    /// 
    /// 
    /// 
    pub fn new<P: AsRef<Path>>(path: P, device_address: u16) -> Result<Self, DiddyBorgError> {
        let mut dev;

        // Try to create a new I2C peripheral.
        match LinuxI2CDevice::new(path, device_address) {
            Ok(d) => { dev = d },
            Err(error) => {
                // Unable to create a new I2C peripheral.
                DiddyBorgError::from(error)
            }
        }
        
        // Ensure that the device is a Diddyborg.
        match DiddyBorg::get_diddyborg_id(&mut dev) {
            Ok(id) => {
                if id == I2C_ID_PICOBORG_REV {
                    // The device is a DiddyBorg.
                    Ok(DiddyBorg {
                        dev,
                        read_buffer: [0; I2C_READ_LEN],
                    })
                }
                else {
                    // The device is not a DiddyBorg.
                    let message = format!("Expected ID {} Got ID {}", I2C_ID_PICOBORG_REV, id);
                    Err(DiddyBorgError::InvalidIdError(message))
                }
            }
            // Failed to read I2C device.
            Err(error) => Err(error)
        }
    }
}