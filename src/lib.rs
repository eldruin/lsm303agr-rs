//! This is a platform agnostic Rust driver for the LSM303AGR ultra-compact
//! high-performance eCompass module: ultra-low-power 3D accelerometer and
//! 3D magnetometer using the [`embedded-hal`] traits.
//!
//! [`embedded-hal`]: https://github.com/rust-embedded/embedded-hal
//!
//! This driver allows you to:
//! - Connect through I2C or SPI. See: [`new_with_i2c()`](Lsm303agr::new_with_i2c) and [`new_with_spi()`](Lsm303agr::new_with_spi) .
//! - Initialize the device. See: [`init()`](Lsm303agr::init).
//! - Accelerometer:
//!     - Read measured acceleration. See: [`acceleration()`](Lsm303agr::acceleration).
//!     - Get accelerometer status. See: [`accel_status()`](Lsm303agr::accel_status).
//!     - Set accelerometer mode and output data rate. See: [`set_accel_mode_and_odr()`](Lsm303agr::set_accel_mode_and_odr).
//!     - Set accelerometer scale. See: [`set_accel_scale()`](Lsm303agr::set_accel_scale).
//!     - Get accelerometer ID. See: [`accelerometer_id()`](Lsm303agr::accelerometer_id).
//!     - Get temperature sensor status. See: [`temperature_status()`](Lsm303agr::temperature_status).
//!     - Read measured temperature. See: [`temperature()`](Lsm303agr::temperature).
//!     - Configure FIFO. See: [`acc_set_fifo_mode()`](Lsm303agr::acc_set_fifo_mode).
//!     - Enable/disable interrupts. See: [`acc_enable_interrupt()`](Lsm303agr::acc_enable_interrupt).
//! - Magnetometer:
//!     - Get the magnetometer status. See: [`mag_status()`](Lsm303agr::mag_status).
//!     - Change into continuous/one-shot mode. See: [`into_mag_continuous()`](Lsm303agr::into_mag_continuous).
//!     - Read measured magnetic field. See: [`magnetic_field()`](Lsm303agr::magnetic_field).
//!     - Set magnetometer mode and output data rate. See: [`set_mag_mode_and_odr()`](Lsm303agr::set_mag_mode_and_odr).
//!     - Get magnetometer ID. See: [`magnetometer_id()`](Lsm303agr::magnetometer_id).
//!     - Enable/disable magnetometer built in offset cancellation. See: [`enable_mag_offset_cancellation()`](Lsm303agr::enable_mag_offset_cancellation).
//!     - Enable/disable magnetometer low-pass filter. See: [`mag_enable_low_pass_filter()`](Lsm303agr::mag_enable_low_pass_filter).
//!
//! <!-- TODO
//! [Introductory blog post](TODO)
//! -->
//!
//! ## The devices
//!
//! The LSM303AGR is an ultralow-power high-performance system-in-package featuring a
//! 3-axis digital linear acceleration sensor and a 3-axis digital magnetic sensor.
//!
//! The LSM303AGR has linear acceleration full scales of ±2g/±4g/±8g/±16g and a magnetic field
//! dynamic range of ±50 gauss. The LSM303AGR includes an I²C serial bus
//! interface that supports standard, fast mode, fast mode plus, and high-speed
//! (100 kHz, 400 kHz, 1 MHz, and 3.4 MHz) and an SPI serial standard interface.
//!
//! The system can be configured to generate an interrupt signal for free-fall,
//! motion detection, and magnetic field detection. The magnetic and accelerometer blocks can be
//! enabled or put into power-down mode separately.
//!
//! Documents: [Datasheet](https://www.st.com/resource/en/datasheet/lsm303agr.pdf) - [Application note](https://www.st.com/resource/en/application_note/dm00265383-ultracompact-highperformance-ecompass-module-based-on-the-lsm303agr-stmicroelectronics.pdf)
//!
//!
//! ## Usage examples (see also examples folder)
//!
//! To use this driver, import this crate and an `embedded_hal` implementation,
//! then instantiate the appropriate device.
//!
//! Please find additional examples using hardware in this repository: [driver-examples]
//!
//! [driver-examples]: https://github.com/eldruin/driver-examples
//!
//! ### Connect through I2C, initialize and take some measurements
//!
//! ```no_run
//! # #[cfg(target_os = "linux")] {
//! use linux_embedded_hal::{Delay, I2cdev};
//! use lsm303agr::{AccelMode, AccelOutputDataRate, Lsm303agr};
//!
//! let dev = I2cdev::new("/dev/i2c-1").unwrap();
//! let mut sensor = Lsm303agr::new_with_i2c(dev);
//!
//! sensor.init().unwrap();
//! sensor.set_accel_mode_and_odr(&mut Delay, AccelMode::Normal, AccelOutputDataRate::Hz10).unwrap();
//!
//! loop {
//!     if sensor.accel_status().unwrap().xyz_new_data() {
//!         let data = sensor.acceleration().unwrap();
//!         println!("Acceleration: x {} y {} z {}", data.x_mg(), data.y_mg(), data.z_mg());
//!     }
//! }
//! # }
//! ```
//!
//! ### Connect through SPI, initialize and take some measurements
//!
//! ```no_run
//! # #[cfg(target_os = "linux")] {
//! use linux_embedded_hal::{Delay, SpidevDevice};
//! use lsm303agr::{AccelMode, AccelOutputDataRate, Lsm303agr};
//!
//! let accel_dev = SpidevDevice::open("/dev/spidev0.0").unwrap();
//! let mag_dev = SpidevDevice::open("/dev/spidev0.1").unwrap();
//! let mut sensor = Lsm303agr::new_with_spi(accel_dev, mag_dev);
//!
//! sensor.init().unwrap();
//! sensor.set_accel_mode_and_odr(&mut Delay, AccelMode::Normal, AccelOutputDataRate::Hz10).unwrap();
//!
//! loop {
//!     if sensor.accel_status().unwrap().xyz_new_data() {
//!         let data = sensor.acceleration().unwrap();
//!         println!("Acceleration: x {} y {} z {}", data.x_mg(), data.y_mg(), data.z_mg());
//!     }
//! }
//! # }
//! ```

#![deny(unsafe_code, missing_docs)]
#![allow(async_fn_in_trait)]
#![no_std]

use core::marker::PhantomData;
mod accel_mode_and_odr;
mod device_impl;
pub mod interface;
mod mag_mode_change;
mod magnetometer;
mod types;
pub use crate::types::{
    mode, AccelMode, AccelOutputDataRate, AccelScale, Acceleration, AccelerometerId, Error,
    FifoMode, Interrupt, MagMode, MagOutputDataRate, MagneticField, MagnetometerId,
    ModeChangeError, Status, Temperature, TemperatureStatus,
};
mod register_address;
use crate::register_address::{
    CfgRegAM, CfgRegBM, CfgRegCM, CtrlReg1A, CtrlReg3A, CtrlReg4A, CtrlReg5A, FifoCtrlRegA,
    TempCfgRegA,
};

/// LSM303AGR device driver
#[derive(Debug)]
pub struct Lsm303agr<DI, MODE> {
    /// Digital interface: I2C or SPI
    iface: DI,
    ctrl_reg1_a: CtrlReg1A,
    ctrl_reg3_a: CtrlReg3A,
    ctrl_reg4_a: CtrlReg4A,
    ctrl_reg5_a: CtrlReg5A,
    cfg_reg_a_m: CfgRegAM,
    cfg_reg_b_m: CfgRegBM,
    cfg_reg_c_m: CfgRegCM,
    temp_cfg_reg_a: TempCfgRegA,
    fifo_ctrl_reg_a: FifoCtrlRegA,
    accel_odr: Option<AccelOutputDataRate>,
    _mag_mode: PhantomData<MODE>,
}

mod private {
    use crate::interface;
    pub trait Sealed {}

    impl<SPIXL, SPIMAG> Sealed for interface::SpiInterface<SPIXL, SPIMAG> {}
    impl<I2C> Sealed for interface::I2cInterface<I2C> {}
}
