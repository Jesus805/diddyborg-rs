use i2cdev::mock::MockI2CDevice;

use super::diddyborg::DiddyBorg;

impl DiddyBorg<MockI2CDevice> {
    /// ## Summary
    /// 
    /// Initialize a new mock DiddyBorg instance.
    /// 
    #[cfg(any(target_os = "linux", test))]
    pub(crate) fn new() -> Self {
        // Create a new mock device.
        DiddyBorg::internal_new(MockI2CDevice::new())
    }
}