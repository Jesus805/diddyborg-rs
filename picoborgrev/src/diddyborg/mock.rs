use i2cdev::mock::MockI2CDevice;
use super::{DiddyBorg, I2C_READ_LEN};

impl DiddyBorg<MockI2CDevice> {
    /// ## Summary
    /// 
    /// Initialize a new mock DiddyBorg instance.
    /// 
    fn new() -> Self {
        // Create a new mock device.
        DiddyBorg {
            dev: MockI2CDevice::new(),
            read_buffer: [0; I2C_READ_LEN],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checksum_should_be_correct() {
        let test = DiddyBorg::<MockI2CDevice>::new();
    }
}