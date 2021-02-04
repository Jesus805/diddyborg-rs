use std::time::Duration;
use std::thread;

use i2cdev::core::I2CDevice;

use crate::error::DiddyBorgError;

use super::command::{Command, CommandValue};

// I2C read length.
pub(crate) const I2C_READ_LEN: usize = 4;
// Maximum allowable PWM value.
const PWM_MAX: f32 = 255.0;
// Wait time in milliseconds after sending a command.
const I2C_WAIT: u64 = 10;

/// ## Summary 
/// 
/// Interface for interacting with a DiddyBorg peripheral using I2C.
/// 
pub struct DiddyBorg<T: I2CDevice> {
    // Interface to I2C peripheral.
    pub(crate) dev: T,
    // Reusable read buffer.
    pub(crate) read_buffer: [u8; I2C_READ_LEN],
}

impl<T: I2CDevice> DiddyBorg<T> {
    /// ## Summary
    /// 
    /// Set the state of the LED.
    ///
    /// ## Parameters
    /// 
    /// state: `true` for on; `false` for off.
    ///
    /// ## Example
    /// 
    /// ```no_run
    /// # use diddyborg::DiddyBorg;
    /// 
    /// let mut driver = DiddyBorg::new("/dev/i2c-1", 0x44);
    /// 
    /// // Turn the LED on.
    /// driver.set_led(true).unwrap();
    /// ```
    /// 
    /// ## Errors
    /// 
    /// 
    /// 
    pub fn set_led(&mut self, state : bool) -> Result<(), DiddyBorgError<T::Error>> {
        let data: [u8; 2] = if state {
            [Command::SetLed as u8, CommandValue::On as u8]
        } else {
            [Command::SetLed as u8, CommandValue::Off as u8]
        };
        
        self.raw_write(&data)
    }
    
    /// ## Summary
    /// 
    /// Read the state of the LED.
    /// 
    /// ## Return value
    /// 
    /// `true` if the LED is on; `false` otherwise.
    /// 
    /// ```no_run
    /// # use diddyborg::DiddyBorg;
    /// 
    /// let mut driver = DiddyBorg::new("/dev/i2c-1", 0x44).unwrap();
    /// 
    /// // Turn the LED on.
    /// driver.set_led(true).unwrap();
    /// 
    /// // Check if the LED is on.
    /// let result: bool = driver.get_led().unwrap();
    /// ```
    /// 
    /// ## Errors
    /// 
    /// 
    /// 
    pub fn get_led(&mut self) -> Result<bool, DiddyBorgError<T::Error>> {
        self.raw_read(Command::GetLed).and_then(|_| {
            let state = self.read_buffer[1];

            if state == CommandValue::Off as u8 {
                Ok(false)
            } else if state == CommandValue::On as u8 {
                Ok(true)
            } else {
                Err(DiddyBorgError::CorruptedData)
            }
        })
    }

    /// ## Summary
    ///
    /// Set the drive level for motor 1.
    /// 
    /// ## Parameters
    ///  
    /// power: The power to set. Allowed interval: [-1, 1].
    ///
    /// ## Example
    ///
    /// ```no_run
    /// # use diddyborg::DiddyBorg;
    /// # use std::time::Duration;
    /// # use std::thread;
    /// 
    /// let mut driver = DiddyBorg::new("/dev/i2c-1", 0x44).unwrap();
    /// 
    /// // Set motor 1 forward at 75% power for 2 seconds.
    /// driver.set_motor1(0.75).unwrap();
    /// thread::sleep(Duration::from_millis(2000));
    /// 
    /// // Set motor 1 reverse at 50% power for 2 seconds.
    /// driver.set_motor1(-0.5).unwrap();
    /// thread::sleep(Duration::from_millis(2000));
    /// 
    /// // Set motor 1 forward at 100% power for 2 seconds.
    /// driver.set_motor1(1).unwrap();
    /// thread::sleep(Duration::from_millis(2000));
    /// 
    /// // Stop motor 1.
    /// driver.set_motor1(0).unwrap();
    /// ```
    /// 
    /// ## Remarks
    /// 
    /// Power is capped at [-1, 1], any higher/lower will be reduced.
    /// 
    /// ## Errors
    /// 
    /// 
    /// 
    pub fn set_motor1(&mut self, power: f32) -> Result<(), DiddyBorgError<T::Error>> {
        let command = if power >= 0.0 {
            Command::SetBFwd
        } else {
            Command::SetBRev
        };

        let pwm = DiddyBorg::<T>::power_to_pwm(power);

        self.raw_write(&[command as u8, pwm])
    }
    
    /// ## Summary
    ///
    /// Get the drive level for motor 1.
    ///
    /// ## Example
    ///
    /// ```no_run
    /// # use diddyborg::DiddyBorg;
    /// # use std::time::Duration;
    /// # use std::thread;
    /// 
    /// let mut driver = DiddyBorg::new("/dev/i2c-1", 0x44).unwrap();
    /// // Set motor 1 forward at 75% power for 2 seconds.
    /// driver.set_motor1(0.75).unwrap();
    /// // Returns ~ 0.75
    /// driver.get_motor1().unwrap();
    /// thread::sleep(Duration::from_millis(2000));
    /// 
    /// // Set motor 1 reverse at 50% power for 2 seconds.
    /// driver.set_motor1(-0.5).unwrap();
    /// // Returns ~ -0.5
    /// driver.get_motor1().unwrap();
    /// thread::sleep(Duration::from_millis(2000));
    /// // Stop motor 1.
    /// driver.set_motor1(0).unwrap();
    /// // Returns ~ 0
    /// driver.get_motor1().unwrap();
    /// ```
    /// 
    /// ## Errors
    /// 
    /// 
    /// 
    pub fn get_motor1(&mut self) -> Result<f32, DiddyBorgError<T::Error>> {
        // Convert a Result<(), DiddyBorgError> into Result<f32, DiddyBorgError>
        self.raw_read(Command::GetB).and_then(|_| {
            let direction = self.read_buffer[1];
            let power = self.read_buffer[2] as f32 / PWM_MAX;

            if direction == CommandValue::Fwd as u8 {
                Ok(power)
            } else if direction == CommandValue::Rev as u8 {
                Ok(-power)
            } else {
                Err(DiddyBorgError::CorruptedData)
            }            
        })
    }

    /// ## Summary
    ///
    /// Set the drive level for motor 2.
    ///
    /// ## Parameters
    ///  
    /// power: The power to set. Allowed interval: [-1, 1].
    ///
    /// ## Example
    ///
    /// ```no_run
    /// # use diddyborg::DiddyBorg;
    /// # use std::time::Duration;
    /// # use std::thread;
    /// 
    /// let mut driver = DiddyBorg::new("/dev/i2c-1", 0x44).unwrap();
    /// // Set motor 2 forward at 75% power for 2 seconds.
    /// driver.set_motor2(0.75).unwrap();
    /// thread::sleep(Duration::from_millis(2000));
    /// // Set motor 2 reverse at 50% power for 2 seconds.
    /// driver.set_motor2(-0.5).unwrap();
    /// thread::sleep(Duration::from_millis(2000));
    /// // Set motor 2 forward at 100% power for 2 seconds.
    /// driver.set_motor2(1).unwrap();
    /// thread::sleep(Duration::from_millis(2000));
    /// // Stop motor 2.
    /// driver.set_motor2(0).unwrap();
    /// ```
    /// 
    /// ## Remarks
    /// 
    /// Power is capped at [-1, 1], any higher/lower will be reduced.
    /// 
    /// ## Errors
    /// 
    /// 
    /// 
    pub fn set_motor2(&mut self, power: f32) -> Result<(), DiddyBorgError<T::Error>> {
        let command = if power >= 0.0 {
            Command::SetAFwd
        } else {
            Command::SetARev
        };

        let pwm = DiddyBorg::<T>::power_to_pwm(power);

        self.raw_write(&[command as u8, pwm])
    }

    /// ## Summary
    ///
    /// Get the drive level for motor 2.
    ///
    /// ## Example
    ///
    /// ```no_run
    /// # use diddyborg::DiddyBorg;
    /// # use std::time::Duration;
    /// # use std::thread;
    /// 
    /// let mut driver = DiddyBorg::new("/dev/i2c-1", 0x44).unwrap();
    /// // Set motor 2 forward at 75% power for 2 seconds.
    /// driver.set_motor2(0.75).unwrap();
    /// // Returns ~ 0.75
    /// driver.get_motor2().unwrap();
    /// thread::sleep(Duration::from_millis(2000));
    /// 
    /// // Set motor 2 reverse at 50% power for 2 seconds.
    /// driver.set_motor2(-0.5).unwrap();
    /// // Returns ~ -0.5
    /// driver.get_motor2().unwrap();
    /// thread::sleep(Duration::from_millis(2000));
    /// // Stop motor 2.
    /// driver.set_motor2(0).unwrap();
    /// // Returns ~ 0
    /// driver.get_motor2().unwrap();
    /// ```
    /// 
    /// ## Errors
    /// 
    /// 
    /// 
    pub fn get_motor2(&mut self) -> Result<f32, DiddyBorgError<T::Error>> {
        self.raw_read(Command::GetA).and_then(|_| {
            let direction = self.read_buffer[1];
            let power = self.read_buffer[2] as f32 / PWM_MAX;

            if direction == CommandValue::Fwd as u8 {
                Ok(power)
            } else if direction == CommandValue::Rev as u8 {
                Ok(-power)
            } else {
                Err(DiddyBorgError::CorruptedData)
            }
        })
    }

    /// ## Summary
    ///
    /// Set the drive level for both motors.
    ///
    /// ## Parameters
    ///  
    /// power: The power to set. Allowed interval: [-1, 1].
    ///
    /// ## Example
    ///
    /// ```no_run
    /// # use diddyborg::DiddyBorg;
    /// # use std::time::Duration;
    /// # use std::thread;
    /// 
    /// let mut driver = DiddyBorg::new("/dev/i2c-1", 0x44).unwrap();
    /// // Set the motors forward at 75% power for 2 seconds.
    /// driver.set_motors(0.75).unwrap();
    /// thread::sleep(Duration::from_millis(2000));
    /// // Set the motors reverse at 50% power for 2 seconds.
    /// driver.set_motors(-0.5).unwrap();
    /// thread::sleep(Duration::from_millis(2000));
    /// // Set the motors forward at 100% power for 2 seconds.
    /// driver.set_motors(1).unwrap();
    /// thread::sleep(Duration::from_millis(2000));
    /// // Stop the motors.
    /// driver.set_motors(0).unwrap();
    /// ```
    /// 
    /// ## Remarks
    /// 
    /// Power is capped at [-1, 1], any higher/lower will be reduced.
    /// 
    /// ## Errors
    /// 
    /// 
    /// 
    pub fn set_motors(&mut self, power: f32) -> Result<(), DiddyBorgError<T::Error>> {
        let command = if power >= 0.0 { 
            Command::SetAllFwd 
        } else {
            Command::SetAllRev
        };

        let pwm = DiddyBorg::<T>::power_to_pwm(power);

        self.raw_write(&[command as u8, pwm])
    }

    /// ## Summary
    ///
    /// Stop both motors
    ///
    /// ## Example
    ///
    /// ```no_run
    /// # use diddyborg::DiddyBorg;
    /// # use std::time::Duration;
    /// # use std::thread;
    /// 
    /// let mut driver = DiddyBorg::new("/dev/i2c-1", 0x44).unwrap();
    /// // Set motors forward at 100% power.
    /// driver.set_motors(1).unwrap();
    /// thread::sleep(Duration::from_millis(2000));
    /// 
    /// // Stop motors.
    /// driver.stop_motors();
    /// ```
    /// 
    pub fn stop_motors(&mut self) -> Result<(), DiddyBorgError<T::Error>> {
        self.raw_write(&[Command::AllOff as u8, 0])
    }

    /// ## Summary
    ///
    /// Resets the EPO latch state, use to allow movement again after the EPO has been tripped
    ///
    /// ## Example
    ///
    /// ```no_run
    /// # use diddyborg::DiddyBorg;
    /// 
    /// let mut driver = DiddyBorg::new("/dev/i2c-1", 0x44).unwrap();
    /// 
    /// driver.reset_epo().unwrap();
    /// ```
    /// 
    /// ## Errors
    /// 
    /// 
    /// 
    pub fn reset_epo(&mut self) -> Result<(), DiddyBorgError<T::Error>> {
        self.raw_write(&[Command::ResetEpo as u8, 0])
    }

    /// ## Summary
    ///
    /// Reads the system EPO latch state. Movement can be re-enabled by calling `reset_epo`.
    ///
    /// # Return value
    /// 
    /// If `false` the EPO has not been tripped, and movement is allowed.
    /// If `true` the EPO has been tripped, movement is disabled if the EPO is not ignored (see `set_epo_ignore`).
    /// 
    /// ## Example
    ///
    /// ```no_run
    /// # use diddyborg::DiddyBorg;
    /// 
    /// let mut driver = DiddyBorg::new("/dev/i2c-1", 0x44).unwrap();
    /// 
    /// let is_epo: bool = driver.get_epo().unwrap();
    /// ```
    /// 
    /// ## Errors
    /// 
    /// 
    /// 
    pub fn get_epo(&mut self) -> Result<bool, DiddyBorgError<T::Error>> {
        self.raw_read(Command::GetEpo).and_then(|_| {
            let state = self.read_buffer[1];

            if state == CommandValue::Off as u8 {
                Ok(false)
            } else if state == CommandValue::On as u8 {
                Ok(true)
            } else {
                Err(DiddyBorgError::CorruptedData)
            }
        })
    }

    /// ## Summary
    ///
    /// Sets the system to ignore or use the EPO latch.
    ///
    /// ## Parameters
    /// 
    /// state: Set to `false` if you have an EPO switch; `true` if you do not.
    /// 
    /// ## Example
    ///
    /// ```no_run
    /// # use diddyborg::DiddyBorg;
    /// 
    /// let mut driver = DiddyBorg::new("/dev/i2c-1", 0x44).unwrap();
    /// 
    /// driver.set_epo_ignore().unwrap();
    /// ```
    /// 
    /// ## Errors
    /// 
    /// 
    /// 
    pub fn set_epo_ignore(&mut self, state: bool) -> Result<(), DiddyBorgError<T::Error>> {
        let data: [u8; 2] = if state {
            [Command::SetEpoIgnore as u8, CommandValue::On as u8]
        }
        else {
            [Command::SetEpoIgnore as u8, CommandValue::Off as u8]
        };

        self.raw_write(&data)
    }

    /// ## Summary
    ///
    /// Reads the system EPO ignore state.
    ///
    /// # Return value
    /// 
    /// `false` for using the EPO latch.
    /// `true` for ignoring the EPO latch.
    /// 
    /// ## Example
    ///
    /// ```no_run
    /// # use diddyborg::DiddyBorg;
    /// 
    /// let mut driver = DiddyBorg::new("/dev/i2c-1", 0x44).unwrap();
    /// 
    /// let is_epo_ignore: bool = driver.get_epo_ignore().unwrap();
    /// ```
    /// 
    /// ## Errors
    /// 
    /// 
    /// 
    pub fn get_epo_ignore(&mut self) -> Result<bool, DiddyBorgError<T::Error>> {
        self.raw_read(Command::GetEpoIgnore).and_then(|_| {
            let state = self.read_buffer[1];

            if state == CommandValue::Off as u8 {
                Ok(false)
            } else if state == CommandValue::On as u8 {
                Ok(true)
            } else {
                Err(DiddyBorgError::CorruptedData)
            }
        })
    }

    /// ## Summary
    ///
    /// Sets the system to enable or disable the communications failsafe.
    /// The failsafe will turn the motors off unless it is commanded at least once every 1/4 of a second.
    /// The failsafe is disabled at power on.
    ///
    /// ## Parameters
    /// 
    /// state: If `true` the failsafe will be enabled; if `false` the failsafe will be disabled.
    /// 
    /// ## Example
    ///
    /// ```no_run
    /// # use diddyborg::DiddyBorg;
    /// 
    /// let mut driver = DiddyBorg::new("/dev/i2c-1", 0x44).unwrap();
    /// 
    /// driver.set_comms_failsafe().unwrap();
    /// ```
    /// 
    /// ## Errors
    /// 
    /// 
    /// 
    pub fn set_comms_failsafe(&mut self, state: bool) -> Result<(), DiddyBorgError<T::Error>> {
        let data: [u8; 2] = if state {
            [Command::SetFailsafe as u8, CommandValue::On as u8]
        }
        else {
            [Command::SetFailsafe as u8, CommandValue::Off as u8]
        };

        self.raw_write(&data)
    }

    /// ## Summary
    ///
    /// Read the current system state of the communications failsafe.
    /// The failsafe will turn the motors off unless it is commanded at least once every 1/4 of a second.
    ///
    /// # Return value
    /// 
    /// `true` if the failsafe is enabled.
    /// `false` if the failsafe is disabled.
    /// 
    /// ## Example
    ///
    /// ```no_run
    /// # use diddyborg::DiddyBorg;
    /// 
    /// let mut driver = DiddyBorg::new("/dev/i2c-1", 0x44).unwrap();
    /// 
    /// let is_failsafe_enabled: bool = driver.get_comms_failsafe().unwrap();
    /// ```
    /// 
    /// ## Errors
    /// 
    /// 
    /// 
    pub fn get_comms_failsafe(&mut self) -> Result<bool, DiddyBorgError<T::Error>> {
        self.raw_read(Command::GetFailsafe).and_then(|_| {
            let state = self.read_buffer[1];

            if state == CommandValue::Off as u8 {
                Ok(false)
            } else if state == CommandValue::On as u8 {
                Ok(true)
            } else {
                Err(DiddyBorgError::CorruptedData)
            }
        })
    }

    /// ## Summary
    ///
    /// Reads the system drive fault state. 
    /// Faults may indicate power problems, such as under-voltage (not enough power), and may be cleared by setting a lower drive power.
    /// If a fault is persistent, it repeatably occurs when trying to control the board, this may indicate a wiring problem such as:
    ///  * The supply is not powerful enough for the motors.
    ///    > The board has a bare minimum requirement of 6V to operate correctly.
    ///    > A recommended minimum supply of 7.2V should be sufficient for smaller motors.
    ///  * The + and - connections for either motor are connected to each other.
    ///  * Either + or - is connected to ground (GND, also known as 0V or earth).
    ///  * Either + or - is connected to the power supply (V+, directly to the battery or power pack).
    ///  * One of the motors may be damaged.
    /// Faults will self-clear, they do not need to be reset, however some faults require both motors to be moving at less than 100% to clear.
    /// The easiest way to check is to put both motors at a low power setting which is high enough for them to rotate easily, such as 30%.
    /// Note that the fault state may be true at power up, this is normal and should clear when both motors have been driven.
    /// If there are no faults but you cannot make your motors move check `get_epo` to see if the safety switch has been tripped.
    /// For more details check the website at www.piborg.org/picoborgrev and double check the wiring instructions.
    ///
    /// # Return value
    /// 
    /// `true` if a fault has been detected.
    /// `false` if there are no problems.
    /// 
    /// ## Example
    ///
    /// ```no_run
    /// # use diddyborg::DiddyBorg;
    /// 
    /// let mut driver = DiddyBorg::new("/dev/i2c-1", 0x44).unwrap();
    /// 
    /// let is_drive_fault: bool = driver.get_drive_fault().unwrap();
    /// ```
    /// 
    /// ## Errors
    /// 
    /// 
    /// 
    pub fn get_drive_fault(&mut self) -> Result<bool, DiddyBorgError<T::Error>> {
        self.raw_read(Command::GetDriveFault).and_then(|_| {
            let state = self.read_buffer[1];

            if state == CommandValue::Off as u8 {
                Ok(false)
            } else if state == CommandValue::On as u8 {
                Ok(true)
            } else {
                Err(DiddyBorgError::CorruptedData)
            }
        })
    }

    /// ## Summary
    /// 
    /// Read from the DiddyBorg.
    ///
    /// ## Parameters
    /// 
    /// command: Read command to send to the DiddyBorg.
    /// 
    /// # Errors
    /// 
    /// 
    /// 
    fn raw_read(&mut self, command : Command) -> Result<(), DiddyBorgError<T::Error>> {
        // Clear existing buffer data.
        self.read_buffer.iter_mut().for_each(|x| *x = 0);

        // Write the command then read the data from the DiddyBorg.
        DiddyBorg::read(&mut self.dev, command, &mut self.read_buffer)
    }

    /// ## Summary
    /// 
    /// Write to the DiddyBorg.
    ///
    /// ## Parameters
    /// 
    /// data: Data to write.
    /// 
    /// # Errors
    /// 
    /// 
    /// 
    fn raw_write(&mut self, data : &[u8]) -> Result<(), DiddyBorgError<T::Error>> {
        // Write the data to the DiddyBorg.
        DiddyBorg::write(&mut self.dev, data)
    }
    
    /// ## Summary
    /// 
    /// Attempt to read the DiddyBorg ID from an I2C device.
    ///
    /// ## Parameters
    /// 
    /// dev: Device to read from.
    /// 
    /// # Errors
    /// 
    /// 
    /// 
    #[cfg(target_os = "linux")]
    fn get_diddyborg_id(dev: &mut T) -> Result<u8, DiddyBorgError<T::Error>> {
        let mut buffer: [u8; I2C_READ_LEN] = [0; I2C_READ_LEN];

        DiddyBorg::read(dev, Command::GetId, &mut buffer).map(|_| buffer[1])
    }

    /// ## Summary
    /// 
    /// Read from an I2C device.
    ///
    /// ## Parameters
    /// 
    /// dev: Device to read from.
    /// 
    /// command: Read command to send to the I2C device.
    /// 
    /// buffer: Buffer to hold read data.
    /// 
    /// # Errors
    /// 
    /// 
    /// 
    fn read(dev: &mut T, command: Command, mut buffer : &mut [u8]) -> Result<(), DiddyBorgError<T::Error>> {
        if let Err(err) = dev.write(&[command as u8]) {
            return Err(DiddyBorgError::<T::Error>::I2C(err));
        }

        thread::sleep(Duration::from_millis(I2C_WAIT));

        dev.read(&mut buffer).map_err(|e| {
            DiddyBorgError::<T::Error>::I2C(e)
        })
    }

    /// ## Summary
    /// 
    /// Write to an I2C device.
    ///
    /// ## Parameters
    /// 
    /// dev: Device to write to.
    /// 
    /// data: Data to write.
    /// 
    /// # Errors
    /// 
    /// 
    /// 
    fn write(dev: &mut T, data : &[u8]) -> Result<(), DiddyBorgError<T::Error>> {
        dev.write(&data).map_err(|e| {
            DiddyBorgError::<T::Error>::I2C(e)
        })
    }

    /// ## Summary
    /// 
    /// Convert a power to PWM.
    ///
    /// ## Parameters
    /// 
    /// power: Power to convert to PWM.
    ///
    /// ## Remarks
    /// 
    /// Power inputs with a magnitude greater than 1 will be converted to 1.
    /// 
    fn power_to_pwm(power: f32) -> u8 {
        let mut pwm = PWM_MAX * power.abs();

        if pwm > PWM_MAX {
            pwm = PWM_MAX;
        }

        pwm as u8
    }
}