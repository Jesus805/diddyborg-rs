use super::{DiddyBorg, I2C_READ_LEN};
use i2cdev::mock::MockI2CDevice;

impl DiddyBorg<MockI2CDevice> {
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
    /// # use i2cdev::mock::MockI2CDevice;
    /// 
    /// let mut driver = DiddyBorg::<MockI2CDevice>::new();
    /// ```
    /// 
    /// ## Errors
    /// 
    /// 
    /// 
    pub fn new() -> Self {
        // Create a new mock device.
        DiddyBorg {
            dev: MockI2CDevice::new(),
            read_buffer: [0; I2C_READ_LEN],
        }
    }
}