pub mod diddyborg;
pub mod error;

use std::time::Duration;
use std::thread;

pub const DEFAULT_PERIPHERAL_ID: u16 = 0x44;
pub const DEFAULT_I2C_PATH: &str = "/dev/i2c-1";

fn main() {
    let mut piborg;

    match diddyborg::DiddyBorg::new(DEFAULT_I2C_PATH, DEFAULT_PERIPHERAL_ID) {
        Ok(p) => piborg = p,
        Err(error) => return,
    }

    piborg.set_motor1(-0.7);
    piborg.set_motor2(0.7);
    thread::sleep(Duration::from_millis(5000));
    piborg.stop_motors();
}
