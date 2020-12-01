mod command;

use i2cdev::linux::{LinuxI2CDevice, LinuxI2CError, LinuxI2CMessage};
use command::{ Command, CommandValue };

const I2C_FOLLOWER: u32 = 0x0703;
const PWM_MAX: f32 = 255.0;
const I2C_MAX_LEN: usize = 4;
const I2C_ID_PICOBORG_REV: u32 = 0x15;
const PERIPHERAL_ID: u16 = 0x44;
const I2C_PATH: &str = "/dev/i2c-1";
pub struct PiBorg {
    port: LinuxI2CDevice,
}

impl PiBorg {
    
    pub fn new() -> Self {
        PiBorg {
            LinuxI2CDevice::new("/dev/i2c-1", PERIPHERAL_ID),
        }
    }

    fn raw_read(&mut self, cmd : Command, buffer : &mut [u8], ) {
        let mut msgs = [
            LinuxI2CMessage::write(&[cmd.value()]),
            LinuxI2CMessage::read(&mut buffer),
        ];

        self.port.transfer(msgs).unwrap();
    }

    fn raw_write(&mut self, data : &[u8]) {
        self.port.write(&data);
    }

    /// ## Summary
    /// 
    /// Sets the current state of the LED.
    ///
    /// ## Parameters
    /// 
    /// state: true for on; false for off.
    ///
    pub fn set_led(&mut self, state : bool) {
        let data: [u8; 2] = if state {
            [Command::SetLed.value(), CommandValue::On.value()]
        } else {
            [Command::SetLed.value(), CommandValue::Off.value()]
        };
        
        self.raw_write(&data)
    }
    
    /// ## Summary
    /// 
    /// Reads the current state of the LED.
    /// 
    /// ## Return value
    /// 
    /// true if the LED is on; false otherwise.
    /// 
    pub fn get_led(&mut self) -> bool {
        let mut buffer: [u8; I2C_MAX_LEN] = [0; I2C_MAX_LEN];
        self.raw_read(Command::GetLed, &mut buffer);
        buffer[1] == CommandValue::On.value()
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
    pub fn set_motor1(&mut self, power: f32) {
        let command = if power >= 0.0 {
            Command::SetBFwd
        } else {
            Command::SetBRev
        };

        let pwm = power_to_pwm(power);

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
    pub fn get_motor1(&mut self) -> f32 {
        let mut buffer: [u8; I2C_MAX_LEN] = [0; I2C_MAX_LEN];
        self.raw_read(Command::GetA, &mut buffer);

        let power = buffer[2] as f32 / PWM_MAX;

        if buffer[1] == CommandValue::Fwd.value() {
            power
        }
        else {
            -power
        }
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
    pub fn set_motor2(&mut self, power: f32) {
        let command = if power >= 0.0 {
            Command::SetAFwd
        } else {
            Command::SetARev
        };

        let pwm = power_to_pwm(power);

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
    pub fn get_motor2(&mut self) -> f32 {
        let mut buffer: [u8; I2C_MAX_LEN] = [0; I2C_MAX_LEN];
        self.raw_read(Command::GetA, &mut buffer);

        let power = buffer[2] as f32 / PWM_MAX;

        if buffer[1] == CommandValue::Fwd.value() {
            power
        }
        else {
            -power
        }
    }

    pub fn set_motors(&mut self, power: f32) {
        let command = if power >= 0.0 { 
            Command::SetAllFwd 
        } else {
            Command::SetAllRev
        };

        let pwm = power_to_pwm(power);

        self.raw_write(&[command.value(), pwm])
    }

    pub fn stop_motors(&mut self) {
        self.raw_write(&[Command::AllOff.value(), 0])
    }

    pub fn reset_epo(&mut self) {
        self.raw_write(&[Command::ResetEpo.value(), 0])
    }

    pub fn get_epo(&mut self) -> bool {
        let mut buffer: [u8; I2C_MAX_LEN] = [0; I2C_MAX_LEN];
        self.raw_read(Command::GetEpo, &mut buffer);
        
        buffer[1] == CommandValue::On.value()
    }

    pub fn set_epo_ignore(&mut self, state: bool) {
        let data: [u8; 2] = if state {
            [Command::SetEpoIgnore.value(), CommandValue::On.value()]
        }
        else {
            [Command::SetEpoIgnore.value(), CommandValue::Off.value()]
        };

        self.raw_write(&data);
    }

    pub fn get_epo_ignore(&mut self) -> bool {
        let mut buffer: [u8; I2C_MAX_LEN] = [0; I2C_MAX_LEN];
        self.raw_read(Command::GetEpoIgnore, &mut buffer);
        
        buffer[1] == CommandValue::On.value()
    }

    pub fn set_comms_failsafe(&mut self, state: bool) {
        let data: [u8; 2] = if state {
            [Command::SetFailsafe.value(), CommandValue::On.value()]
        }
        else {
            [Command::SetFailsafe.value(), CommandValue::Off.value()]
        };

        self.raw_write(&data);
    }

    pub fn get_comms_failsafe(&mut self) -> bool {
        let mut buffer: [u8; I2C_MAX_LEN] = [0; I2C_MAX_LEN];
        self.raw_read(Command::GetFailsafe, &mut buffer);
        
        buffer[1] == CommandValue::On.value()
    }

    pub fn get_drive_fault(&mut self) -> bool {
        let mut buffer: [u8; I2C_MAX_LEN] = [0; I2C_MAX_LEN];
        self.raw_read(Command::GetDriveFault, &mut buffer);
        
        buffer[1] == CommandValue::On.value()
    }
}

fn power_to_pwm(power: f32) -> u8 {
    let mut pwm = PWM_MAX * power;
    
    if power < 0.0 {
        pwm *= -1.0;
    }

    if pwm > PWM_MAX {
        pwm = PWM_MAX;
    }

    pwm as u8
}

fn main() {
    println!("Hello, world!");
}
