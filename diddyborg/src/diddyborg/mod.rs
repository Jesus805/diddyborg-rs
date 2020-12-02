mod command;

use std::path::Path;
use i2cdev::{core::I2CDevice, linux::{LinuxI2CDevice, LinuxI2CError, LinuxI2CMessage}};
use command::{ Command, CommandValue };
use super::error::DiddyBorgError;

use std::time::Duration;
use std::thread;

const PWM_MAX: f32 = 255.0;

const I2C_ID_PICOBORG_REV: u8 = 0x15;
const I2C_MAX_LEN: usize = 4;
const I2C_WAIT: u64 = 10;

pub struct DiddyBorg {

    dev: LinuxI2CDevice,
    /// Read buffer
    read_buffer: [u8; I2C_MAX_LEN],
}

impl DiddyBorg {
    /// ## Summary
    /// 
    /// Create a new DiddyBorg instance for the specified path.
    ///
    /// ## Parameters
    /// 
    /// path: Path to the I2C interface.
    ///
    /// device_address: The I2C address of the device.
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
                        read_buffer: [0; I2C_MAX_LEN],
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
    /// Sets the state of the LED.
    ///
    /// ## Parameters
    /// 
    /// state: true for on; false for off.
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
    /// Reads the state of the LED.
    /// 
    /// ## Return value
    /// 
    /// true if the LED is on; false otherwise.
    /// 
    pub fn get_led(&mut self) -> Result<bool, DiddyBorgError> {
        self.raw_read(Command::GetLed).map(|_| self.read_buffer[1] == CommandValue::On.value())
    }

    /// ## Summary
    ///
    /// Sets the drive level for motor 1.
    /// 
    /// ## Parameters
    ///  
    /// power: The power to set. Allowed interval: [-1, 1].
    ///
    /// ## Example
    ///
    /// ```no_run
    /// # use piborg::PiBorg;
    /// 
    /// let mut driver = PiBorg::new();
    /// // Stop motor 1.
    /// driver.set_motor1(0);
    /// // Set motor 1 forward at 75% power.
    /// driver.set_motor1(0.75);
    /// // Set motor 1 reverse at 50% power.
    /// driver.set_motor1(-0.5);
    /// // Set motor 1 forward at 100% power.
    /// driver.set_motor1(1);
    /// ```
    /// 
    /// ## Remarks
    /// 
    /// Power is capped at [-1, 1], any higher/lower will be reduced.
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
    /// Gets the drive level for motor 1.
    ///
    /// ## Return value
    /// 
    /// 
    /// 
    /// ## Example
    ///
    /// ```no_run
    /// # use piborg::PiBorg;
    /// 
    /// let mut driver = PiBorg::new();
    /// // Stop motor 1.
    /// driver.set_motor1(0);
    /// // Returns ~ 0.0
    /// driver.get_motor1();
    /// // Set motor 1 forward at 75% power.
    /// driver.set_motor1(0.75);
    /// // Returns ~ 0.75
    /// driver.get_motor1();
    /// // Set motor 1 reverse at 50% power.
    /// driver.set_motor1(-0.5);
    /// // Returns ~ -0.5
    /// driver.get_motor1();
    /// // Set motor 1 forward at 100% power.
    /// driver.set_motor1(1);
    /// // Returns ~ 1.0
    /// driver.get_motor(1);
    /// ```
    /// 
    pub fn get_motor1(&mut self) -> Result<f32, DiddyBorgError> {
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
    /// Sets the drive level for motor 2, from +1 to -1.
    ///
    /// ## Example
    ///
    /// ```no_run
    /// # use piborg::PiBorg;
    /// 
    /// let mut driver = PiBorg::new();
    /// // Stop motor 2.
    /// driver.set_motor2(0);
    /// // Set motor 2 forward at 75% power.
    /// driver.set_motor2(0.75);
    /// // Set motor 2 reverse at 50% power.
    /// driver.set_motor2(-0.5);
    /// // Set motor 2 forward at 100% power.
    /// driver.set_motor2(1);
    /// ```
    /// 
    /// ## Remarks
    /// 
    /// Power is capped at [-1, 1], any higher/lower will be reduced.
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
    /// Gets the drive level for motor 2, from +1 to -1.
    ///
    /// ## Example
    ///
    /// ```no_run
    /// # use piborg::PiBorg;
    /// 
    /// let mut driver = PiBorg::new();
    /// // Stop motor 2.
    /// driver.set_motor1(0);
    /// // Returns ~ 0.0
    /// driver.get_motor1();
    /// // Set motor 2 forward at 75% power.
    /// driver.set_motor1(0.75);
    /// // Returns ~ 0.75
    /// driver.get_motor1();
    /// // Set motor 2 reverse at 50% power.
    /// driver.set_motor1(-0.5);
    /// // Returns ~ -0.5
    /// driver.get_motor1();
    /// // Set motor 2 forward at 100% power.
    /// driver.set_motor1(1);
    /// // Returns ~ 1.0
    /// driver.get_motor(1);
    /// ```
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

    pub fn set_motors(&mut self, power: f32) -> Result<(), DiddyBorgError> {
        let command = if power >= 0.0 { 
            Command::SetAllFwd 
        } else {
            Command::SetAllRev
        };

        let pwm = DiddyBorg::power_to_pwm(power);

        self.raw_write(&[command.value(), pwm])
    }

    pub fn stop_motors(&mut self) -> Result<(), DiddyBorgError> {
        self.raw_write(&[Command::AllOff.value(), 0])
    }

    pub fn reset_epo(&mut self) -> Result<(), DiddyBorgError> {
        self.raw_write(&[Command::ResetEpo.value(), 0])
    }

    pub fn get_epo(&mut self) -> Result<bool, DiddyBorgError> {
        self.raw_read(Command::GetEpo).map(|_| self.read_buffer[1] == CommandValue::On.value())
    }

    pub fn set_epo_ignore(&mut self, state: bool) -> Result<(), DiddyBorgError> {
        let data: [u8; 2] = if state {
            [Command::SetEpoIgnore.value(), CommandValue::On.value()]
        }
        else {
            [Command::SetEpoIgnore.value(), CommandValue::Off.value()]
        };

        self.raw_write(&data)
    }

    pub fn get_epo_ignore(&mut self) -> Result<bool, DiddyBorgError> {
        self.raw_read(Command::GetEpoIgnore).map(|_| self.read_buffer[1] == CommandValue::On.value())
    }

    pub fn set_comms_failsafe(&mut self, state: bool) -> Result<(), DiddyBorgError> {
        let data: [u8; 2] = if state {
            [Command::SetFailsafe.value(), CommandValue::On.value()]
        }
        else {
            [Command::SetFailsafe.value(), CommandValue::Off.value()]
        };

        self.raw_write(&data)
    }

    pub fn get_comms_failsafe(&mut self) -> Result<bool, DiddyBorgError> {
        self.raw_read(Command::GetFailsafe).map(|_| self.read_buffer[1] == CommandValue::On.value())
    }

    pub fn get_drive_fault(&mut self) -> Result<bool, DiddyBorgError> {
        self.raw_read(Command::GetDriveFault).map(|_| self.read_buffer[1] == CommandValue::On.value())
    }

    fn raw_read(&mut self, command : Command) -> Result<(), DiddyBorgError> {
        // Clear existing buffer data.
        self.read_buffer.iter_mut().for_each(|x| *x = 0);

        // Write the command then read the data from the DiddyBorg.
        DiddyBorg::read(self.dev, command, &mut self.read_buffer)
    }

    fn raw_write(&mut self, data : &[u8]) -> Result<(), DiddyBorgError> {
        // Write the data to the DiddyBorg.
        DiddyBorg::write(self.dev, data)
    }

    fn get_diddyborg_id<T: I2CDevice>(dev: T) -> Result<u8, DiddyBorgError> {
        let mut buffer: [u8; I2C_MAX_LEN] = [0; I2C_MAX_LEN];

        DiddyBorg::read(dev, Command::GetId, &mut buffer).map(|_| buffer[1])
    }

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
    /// Write to the I2C device.
    ///
    /// ## Parameters
    /// 
    /// dev: Device to write to.
    /// 
    /// data: Data to write.
    /// 
    /// ## Remarks
    /// 
    /// Power inputs with a magnitude greater than 1 will be converted to 1.
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