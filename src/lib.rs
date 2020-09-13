//! This is a platform agnostic Rust driver for the LSM303AGR
//! inertial measurement unit using the [`embedded-hal`] traits.
//!
//! [`embedded-hal`]: https://github.com/rust-embedded/embedded-hal
//!
//! This driver allows you to:
//! - Connect through I2C or SPI. See: [`new_with_i2c()`].
//! - Initialize the device. See: [`init()`].
//! - Accelerometer:
//!     - Read accelerometer data. See: [`accel_data()`].
//!     - Get the accelerometer status. See: [`accel_status()`].
//!     - Get accelerometer ID. See: [`accelerometer_id()`].
//! - Magnetometer:
//!     - Get the magnetometer status. See: [`mag_status()`].
//!     - Get magnetometer ID. See: [`magnetometer_id()`].
//!
//! [`new_with_i2c()`]: struct.Lsm303agr.html#method.new_with_i2c
//! [`init()`]: struct.Lsm303agr.html#method.init
//! [`accel_status()`]: struct.Lsm303agr.html#method.accel_status
//! [`accel_data()`]: struct.Lsm303agr.html#method.accel_data
//! [`mag_status()`]: struct.Lsm303agr.html#method.mag_status
//! [`accelerometer_id()`]: struct.Lsm303agr.html#method.accelerometer_id
//! [`magnetometer_id()`]: struct.Lsm303agr.html#method.magnetometer_id
//!
//! <!-- TODO
//! [Introductory blog post](TODO)
//! -->
//!
//! ## The devices
//!
//! The LSM303AGR is an inertial measurement unit (IMU) consisting of a
//! state-of-the-art 3-axis, low-g accelerometer and a low power 3-axis
//! gyroscope. It has been designed for low power, high precision 6-axis and
//! 9-axis applications in mobile phones, tablets, wearable devices, remote
//! controls, game controllers, head-mounted devices and toys.
//!
//! The LSM303AGR is available in a compact 14-pin 2.5 × 3.0 × 0.83 mm3 LGA
//! package. When accelerometer and gyroscope are in full operation mode, power
//! consumption is typically 925 μA, enabling always-on applications in
//! battery driven devices.
//!
//! Further Bosch Sensortec sensors, e.g. geomagnetic (BMM150) can be connected
//! as slave via a secondary I2C interface. In this configuration, the LSM303AGR
//! controls the data acquisition of the external sensor and the synchronized
//! data of all sensors is stored the register data and can be additionally
//! stored in the built-in FIFO.
//!
//! Besides the flexible primary interface (I2C or SPI) that is used to connect
//! to the host, LSM303AGR provides an additional secondary interface. This
//! secondary interface can be used in SPI mode for OIS (optical image
//! stabilization) applications in conjunction with camera modules, or in
//! advanced gaming use cases.
//!
//! [Datasheet](https://www.bosch-sensortec.com/media/boschsensortec/downloads/datasheets/bst-lsm303agr-ds000.pdf)
//!

#![deny(unsafe_code, missing_docs)]
#![no_std]

mod accel_mode_and_odr;
mod device_impl;
pub mod interface;
mod types;
pub use crate::types::{AccelMode, AccelOutputDataRate, Error, Status, UnscaledMeasurement};
mod register_address;
use crate::register_address::{BitFlags, Register};

/// LSM303AGR device driver
#[derive(Debug)]
pub struct Lsm303agr<DI> {
    /// Digital interface: I2C or SPI
    iface: DI,
    ctrl_reg1_a: Config,
    ctrl_reg4_a: Config,
    accel_odr: Option<AccelOutputDataRate>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
struct Config {
    bits: u8,
}

impl Config {
    fn with_high(self, mask: u8) -> Self {
        Config {
            bits: self.bits | mask,
        }
    }
    fn with_low(self, mask: u8) -> Self {
        Config {
            bits: self.bits & !mask,
        }
    }
    fn is_high(&self, mask: u8) -> bool {
        (self.bits & mask) != 0
    }
}

impl From<u8> for Config {
    fn from(bits: u8) -> Self {
        Config { bits }
    }
}

mod private {
    use crate::interface;
    pub trait Sealed {}

    impl<SPI, CSXL, CSMAG> Sealed for interface::SpiInterface<SPI, CSXL, CSMAG> {}
    impl<I2C> Sealed for interface::I2cInterface<I2C> {}
}
