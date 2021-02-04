use i2cdev::mock::MockI2CDevice;

use super::diddyborg::{DiddyBorg, I2C_READ_LEN};

impl DiddyBorg<MockI2CDevice> {
    /// ## Summary
    /// 
    /// Initialize a new mock DiddyBorg instance.
    /// 
    pub(crate) fn new() -> Self {
        // Create a new mock device.
        DiddyBorg {
            dev: MockI2CDevice::new(),
            read_buffer: [0; I2C_READ_LEN],
        }
    }
}