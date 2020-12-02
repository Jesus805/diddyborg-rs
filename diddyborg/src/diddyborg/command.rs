/// ## Summary
/// Represents a I2C Command to write to the PicoBorgRev.
/// 
pub(crate) enum Command {
    /// Set the LED status.
    SetLed,
    /// Get the LED status.
    GetLed,
    /// Set motor 2 PWM rate in a forwards direction.
    SetAFwd,
    /// Set motor 2 PWM rate in a reverse direction.
    SetARev,
    /// Get motor 2 direction and PWM rate.
    GetA,
    /// Set motor 1 PWM rate in a forwards direction.
    SetBFwd,
    /// Set motor 1 PWM rate in a reverse direction.
    SetBRev,
    /// Get motor 1 direction and PWM rate.
    GetB,
    /// Switch everything off.
    AllOff,
    /// Resets the EPO flag, use after EPO has been tripped and switch is now clear.
    ResetEpo,
    /// Get the EPO latched flag.
    GetEpo,
    /// Set the EPO ignored flag, allows the system to run without an EPO.
    SetEpoIgnore,
    /// Get the EPO ignored flag.
    GetEpoIgnore,
    /// Get the drive fault flag, indicates faults such as short-circuits and under voltage.
    GetDriveFault,
    /// Set all motors PWM rate in a forwards direction.
    SetAllFwd,
    /// Set all motors PWM rate in a reverse direction.
    SetAllRev,
    /// Set the failsafe flag, turns the motors off if communication is interrupted.
    SetFailsafe,
    /// Get the failsafe flag.
    GetFailsafe,
    /// Set the board into encoder or speed mode.
    SetEncMode,
    /// Get the boards current mode, encoder or speed.
    GetEncMode,
    /// Move motor 2 forward by n encoder ticks.
    MoveAFwd,
    /// Move motor 2 reverse by n encoder ticks.
    MoveARev,
    /// Move motor 1 forward by n encoder ticks.
    MoveBFwd,
    /// Move motor 1 reverse by n encoder ticks.
    MoveBRev,
    /// Move all motors forward by n encoder ticks.
    MoveAllFwd,
    /// Move all motors reverse by n encoder ticks.
    MoveAllRev,
    /// Get the status of encoders moving.
    GetEncMoving,
    /// Set the maximum PWM rate in encoder mode.
    SetEncSpeed,
    /// Get the maximum PWM rate in encoder mode.
    GetEncSpeed,
    /// Get the board identifier.
    GetId,
    /// Set a new I2C address.
    SetI2cAdd,
}

impl Command {
    /// ## Summary
    ///
    /// Convert a Command to it's byte equivalent.
    ///
    pub(crate) fn value(&self) -> u8 {
        match self {
            Command::SetLed => 0x01,
            Command::GetLed => 0x02,
            Command::SetAFwd => 0x03,
            Command::SetARev => 0x04,
            Command::GetA => 0x05,
            Command::SetBFwd => 0x06,
            Command::SetBRev => 0x07,
            Command::GetB => 0x08,
            Command::AllOff => 0x09,
            Command::ResetEpo => 0x0A,
            Command::GetEpo => 0x0B,
            Command::SetEpoIgnore => 0x0C,
            Command::GetEpoIgnore => 0x0D,
            Command::GetDriveFault => 0x0E,
            Command::SetAllFwd => 0x0F,
            Command::SetAllRev => 0x10,
            Command::SetFailsafe => 0x11,
            Command::GetFailsafe => 0x12,
            Command::SetEncMode => 0x13,
            Command::GetEncMode => 0x14,
            Command::MoveAFwd => 0x15,
            Command::MoveARev => 0x16,
            Command::MoveBFwd => 0x17,
            Command::MoveBRev => 0x18,
            Command::MoveAllFwd => 0x19,
            Command::MoveAllRev => 0x1A,
            Command::GetEncMoving => 0x1B,
            Command::SetEncSpeed => 0x1C,
            Command::GetEncSpeed => 0x1D,
            Command::GetId => 0x99,
            Command::SetI2cAdd => 0xAA,
        }
    }
}

pub(crate) enum CommandValue {
    // Off
    Off,
    // On
    On,
    // Forward
    Fwd,
    // Reverse
    Rev,
}

impl CommandValue {
    /// ## Summary
    ///
    /// Convert a CommandValue to it's byte equivalent.
    ///
    pub(crate) fn value(&self) -> u8 {
        match self {
            CommandValue::Off => 0x00,        
            CommandValue::On => 0x01,

            CommandValue::Fwd => 0x01,
            CommandValue::Rev => 0x02,
        }
    }
}