#[cfg(test)]
mod tests {
    use crate::diddyborg::DiddyBorg;
    use i2cdev::mock::MockI2CDevice;

    #[test]
    fn checksum_should_be_correct() {
        let mut test = DiddyBorg::<MockI2CDevice>::new();
        test.set_led(true).unwrap();
        let state = test.get_led();
        assert!(state.is_ok())
    }
}