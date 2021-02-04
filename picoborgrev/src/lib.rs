pub mod error;
pub mod diddyborg;
mod mock;
mod test;
#[cfg(any(target_os = "linux"))]
pub mod linux;

pub use diddyborg::*;

/// The default peripheral ID for DiddyBorgs.
pub const DEFAULT_PERIPHERAL_ID: u16 = 0x44;
/// The default path to the I2C file descriptor in a Raspberry Pi.
pub const DEFAULT_I2C_PATH: &str = "/dev/i2c-1";