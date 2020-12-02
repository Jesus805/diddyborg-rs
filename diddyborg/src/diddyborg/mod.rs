mod command;

use command::{ Command, CommandValue };
use i2cdev::{core::I2CDevice, linux::{LinuxI2CDevice, LinuxI2CError, LinuxI2CMessage}};
use std::path::Path;
use std::time::Duration;
use std::thread;
use super::error::DiddyBorgError;

// Maximum allowable PWM value.
const PWM_MAX: f32 = 255.0;
// PicoBorg peripheral ID.
const I2C_ID_PICOBORG_REV: u8 = 0x15;
// I2C read length.
const I2C_READ_LEN: usize = 4;
// Wait time in milliseconds after sending a command.
const I2C_WAIT: u64 = 10;

/// ## Summary 
/// 
/// Interface for interacting with a DiddyBorg peripheral using I2C.
/// 
pub struct DiddyBorg {
    // Interface to I2C peripheral.
    dev: LinuxI2CDevice,
    // Reusable read buffer.
    read_buffer: [u8; I2C_READ_LEN],
}

impl DiddyBorg {
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
        let dev;

        // Try to create a new I2C peripheral.
        match LinuxI2CDevice::new(path, device_address) {
            Ok(d) => { dev = d },
            Err(error) => {
                // Unable to create a new I2C peripheral.
                return Err(DiddyBorgError { });
            }
        }
        
        // Ensure that the device is a Diddyborg.
        match DiddyBorg::get_diddyborg_id(dev) {
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
                    Err(DiddyBorgError { })
                }
            }
            // Failed to read I2C device.
            Err(error) => Err(error)
        }
    }

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
    pub fn set_led(&mut self, state : bool) -> Result<(), DiddyBorgError> {
        let data: [u8; 2] = if state {
            [Command::SetLed.value(), CommandValue::On.value()]
        } else {
            [Command::SetLed.value(), CommandValue::Off.value()]
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
    pub fn get_led(&mut self) -> Result<bool, DiddyBorgError> {
        self.raw_read(Command::GetLed).map(|_| self.read_buffer[1] == CommandValue::On.value())
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
    pub fn set_motor1(&mut self, power: f32) -> Result<(), DiddyBorgError> {
        let command = if power >= 0.0 {
            Command::SetBFwd
        } else {
            Command::SetBRev
        };

        let pwm = DiddyBorg::power_to_pwm(power);

        self.raw_write(&[command.value(), pwm])
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
    pub fn get_motor1(&mut self) -> Result<f32, DiddyBorgError> {
        // Convert a Result<(), DiddyBorgError> into Result<f32, DiddyBorgError>
        self.raw_read(Command::GetB).map(|_| {
            let power = self.read_buffer[2] as f32 / PWM_MAX;

            if self.read_buffer[1] == CommandValue::Fwd.value() {
                power
            }
            else {
                -power
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
    pub fn set_motor2(&mut self, power: f32) -> Result<(), DiddyBorgError> {
        let command = if power >= 0.0 {
            Command::SetAFwd
        } else {
            Command::SetARev
        };

        let pwm = DiddyBorg::power_to_pwm(power);

        self.raw_write(&[command.value(), pwm])
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
    pub fn get_motor2(&mut self) -> Result<f32, DiddyBorgError> {
        self.raw_read(Command::GetA).map(|_| {
            let power = self.read_buffer[2] as f32 / PWM_MAX;

            if self.read_buffer[1] == CommandValue::Fwd.value() {
                power
            }
            else {
                -power
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
    pub fn set_motors(&mut self, power: f32) -> Result<(), DiddyBorgError> {
        let command = if power >= 0.0 { 
            Command::SetAllFwd 
        } else {
            Command::SetAllRev
        };

        let pwm = DiddyBorg::power_to_pwm(power);

        self.raw_write(&[command.value(), pwm])
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
    pub fn stop_motors(&mut self) -> Result<(), DiddyBorgError> {
        self.raw_write(&[Command::AllOff.value(), 0])
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
    pub fn reset_epo(&mut self) -> Result<(), DiddyBorgError> {
        self.raw_write(&[Command::ResetEpo.value(), 0])
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
    pub fn get_epo(&mut self) -> Result<bool, DiddyBorgError> {
        self.raw_read(Command::GetEpo).map(|_| self.read_buffer[1] == CommandValue::On.value())
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
    pub fn set_epo_ignore(&mut self, state: bool) -> Result<(), DiddyBorgError> {
        let data: [u8; 2] = if state {
            [Command::SetEpoIgnore.value(), CommandValue::On.value()]
        }
        else {
            [Command::SetEpoIgnore.value(), CommandValue::Off.value()]
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
    pub fn get_epo_ignore(&mut self) -> Result<bool, DiddyBorgError> {
        self.raw_read(Command::GetEpoIgnore).map(|_| self.read_buffer[1] == CommandValue::On.value())
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
    pub fn set_comms_failsafe(&mut self, state: bool) -> Result<(), DiddyBorgError> {
        let data: [u8; 2] = if state {
            [Command::SetFailsafe.value(), CommandValue::On.value()]
        }
        else {
            [Command::SetFailsafe.value(), CommandValue::Off.value()]
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
    pub fn get_comms_failsafe(&mut self) -> Result<bool, DiddyBorgError> {
        self.raw_read(Command::GetFailsafe).map(|_| self.read_buffer[1] == CommandValue::On.value())
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
    pub fn get_drive_fault(&mut self) -> Result<bool, DiddyBorgError> {
        self.raw_read(Command::GetDriveFault).map(|_| self.read_buffer[1] == CommandValue::On.value())
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
    fn raw_read(&mut self, command : Command) -> Result<(), DiddyBorgError> {
        // Clear existing buffer data.
        self.read_buffer.iter_mut().for_each(|x| *x = 0);

        // Write the command then read the data from the DiddyBorg.
        DiddyBorg::read(self.dev, command, &mut self.read_buffer)
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
    fn raw_write(&mut self, data : &[u8]) -> Result<(), DiddyBorgError> {
        // Write the data to the DiddyBorg.
        DiddyBorg::write(self.dev, data)
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
    fn get_diddyborg_id<T: I2CDevice>(dev: T) -> Result<u8, DiddyBorgError> {
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
    fn read<T: I2CDevice>(dev: T, command: Command, mut buffer : &mut [u8]) -> Result<(), DiddyBorgError> {
        match dev.write(&[command.value()]) {
            Ok(_) => {},
            Err(_) => { return Err(DiddyBorgError { })}
        }

        thread::sleep(Duration::from_millis(I2C_WAIT));

        match dev.read(&mut buffer) {
            Ok(_) => Ok(()),
            Err(_) => { Err(DiddyBorgError { }) }
        }
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
    fn write<T: I2CDevice>(dev: T, data : &[u8]) -> Result<(), DiddyBorgError> {
        match dev.write(&data) {
            Ok(_) => Ok(()),
            Err(_) => { Err(DiddyBorgError { }) },
        }
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