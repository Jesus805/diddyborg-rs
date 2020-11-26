mod command;

use command::{ Command, CommandValue };

const PWM_MAX: i32 = 255;
const I2C_MAX_LEN: i32 = 4;
const I2C_ID_PICOBORG_REV: i32 = 0x15;

fn raw_write(cmd : Command, data : &[u8]) {
    //data.len()
    let val = cmd.value();
    for byte in data {

    }
}

fn raw_read(cmd : Command) -> u8 {
    0
}

pub fn set_led(state : bool) {
    let data: [u8; 1] = if state {
        [CommandValue::On.value()]
    } else {
        [CommandValue::Off.value()]
    };

    raw_write(Command::SetLed, &data)
}

pub fn get_led() -> bool {
    let val = raw_read(Command::GetLed);
    return val == CommandValue::On.value();
}


fn main() {
    println!("Hello, world!");
}
