use bitflags::bitflags;

use crate::register_address::{WHO_AM_I_A_VAL, WHO_AM_I_M_VAL};

/// All possible errors in this crate
#[derive(Debug)]
pub enum Error<CommE, PinE> {
    /// I²C / SPI communication error
    Comm(CommE),
    /// Chip-select pin error (SPI)
    Pin(PinE),
    /// Invalid input data provided
    InvalidInputData,
}

/// All possible errors in this crate
#[derive(Debug)]
pub struct ModeChangeError<CommE, PinE, DEV> {
    /// I²C / SPI communication error
    pub error: Error<CommE, PinE>,
    /// Original device without mode changed
    pub dev: DEV,
}

/// Device operation modes
pub mod mode {
    /// Marker type for magnetometer in one-shot (single) mode.
    #[derive(Debug)]
    pub enum MagOneShot {}
    /// Marker type for magnetometer in continuous mode.
    #[derive(Debug)]
    pub enum MagContinuous {}
}

/// An Accelerometer ID.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AccelerometerId {
    pub(crate) raw: u8,
}

impl AccelerometerId {
    /// Raw accelerometer ID.
    pub const fn raw(&self) -> u8 {
        self.raw
    }

    /// Check if the ID corresponds to the expected value.
    pub const fn is_correct(&self) -> bool {
        self.raw == WHO_AM_I_A_VAL
    }
}

/// An acceleration measurement.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Acceleration {
    pub(crate) x: u16,
    pub(crate) y: u16,
    pub(crate) z: u16,
    pub(crate) mode: AccelMode,
    pub(crate) scale: AccelScale,
}

impl Acceleration {
    /// Raw acceleration in X-direction.
    #[inline]
    pub const fn x_raw(&self) -> u16 {
        self.x
    }

    /// Raw acceleration in Y-direction.
    #[inline]
    pub const fn y_raw(&self) -> u16 {
        self.y
    }

    /// Raw acceleration in Z-direction.
    #[inline]
    pub const fn z_raw(&self) -> u16 {
        self.z
    }

    /// Raw acceleration in X-, Y- and Z-directions.
    #[inline]
    pub const fn xyz_raw(&self) -> (u16, u16, u16) {
        (self.x, self.y, self.z)
    }

    /// Unscaled acceleration in X-direction.
    #[inline]
    pub const fn x_unscaled(&self) -> i16 {
        (self.x as i16) / self.mode.resolution_factor()
    }

    /// Unscaled acceleration in Y-direction.
    #[inline]
    pub const fn y_unscaled(&self) -> i16 {
        (self.y as i16) / self.mode.resolution_factor()
    }

    /// Unscaled acceleration in Z-direction.
    #[inline]
    pub const fn z_unscaled(&self) -> i16 {
        (self.z as i16) / self.mode.resolution_factor()
    }

    /// Unscaled acceleration in X-, Y- and Z-directions.
    #[inline]
    pub const fn xyz_unscaled(&self) -> (i16, i16, i16) {
        let resolution_factor = self.mode.resolution_factor();

        (
            (self.x as i16) / resolution_factor,
            (self.y as i16) / resolution_factor,
            (self.z as i16) / resolution_factor,
        )
    }

    /// Acceleration in X-direction in m*g* (milli-*g*).
    #[inline]
    pub const fn x_mg(&self) -> i32 {
        (self.x_unscaled() as i32) * self.mode.scaling_factor(self.scale)
    }

    /// Acceleration in Y-direction in m*g* (milli-*g*).
    #[inline]
    pub const fn y_mg(&self) -> i32 {
        (self.y_unscaled() as i32) * self.mode.scaling_factor(self.scale)
    }

    /// Acceleration in Z-direction in m*g* (milli-*g*).
    #[inline]
    pub const fn z_mg(&self) -> i32 {
        (self.z_unscaled() as i32) * self.mode.scaling_factor(self.scale)
    }

    /// Acceleration in X-, Y- and Z-directions in m*g* (milli-*g*).
    #[inline]
    pub const fn xyz_mg(&self) -> (i32, i32, i32) {
        let (x_unscaled, y_unscaled, z_unscaled) = self.xyz_unscaled();
        let scaling_factor = self.mode.scaling_factor(self.scale);

        (
            (x_unscaled as i32) * scaling_factor,
            (y_unscaled as i32) * scaling_factor,
            (z_unscaled as i32) * scaling_factor,
        )
    }
}

/// A Magnetometer ID.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MagnetometerId {
    pub(crate) raw: u8,
}

impl MagnetometerId {
    /// Raw magnetometer ID.
    pub const fn raw(&self) -> u8 {
        self.raw
    }

    /// Check if the ID corresponds to the expected value.
    pub const fn is_correct(&self) -> bool {
        self.raw == WHO_AM_I_M_VAL
    }
}

/// A magnetic field measurement.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct MagneticField {
    pub(crate) x: u16,
    pub(crate) y: u16,
    pub(crate) z: u16,
}

impl MagneticField {
    const SCALING_FACTOR: i32 = 150;

    /// Raw magnetic field in X-direction.
    #[inline]
    pub const fn x_raw(&self) -> u16 {
        self.x
    }

    /// Raw magnetic field in Y-direction.
    #[inline]
    pub const fn y_raw(&self) -> u16 {
        self.y
    }

    /// Raw magnetic field in Z-direction.
    #[inline]
    pub const fn z_raw(&self) -> u16 {
        self.z
    }

    /// Raw magnetic field in X-, Y- and Z-directions.
    #[inline]
    pub const fn xyz_raw(&self) -> (u16, u16, u16) {
        (self.x, self.y, self.z)
    }

    /// Unscaled magnetic field in X-direction.
    #[inline]
    pub const fn x_unscaled(&self) -> i16 {
        self.x as i16
    }

    /// Unscaled magnetic field in Y-direction.
    #[inline]
    pub const fn y_unscaled(&self) -> i16 {
        self.y as i16
    }

    /// Unscaled magnetic field in Z-direction.
    #[inline]
    pub const fn z_unscaled(&self) -> i16 {
        self.z as i16
    }

    /// Unscaled magnetic field in X-, Y- and Z-directions.
    #[inline]
    pub const fn xyz_unscaled(&self) -> (i16, i16, i16) {
        (self.x as i16, self.y as i16, self.z as i16)
    }

    /// Magnetic field in X-direction in nT (nano-Tesla).
    #[inline]
    pub const fn x_nt(&self) -> i32 {
        (self.x_unscaled() as i32) * Self::SCALING_FACTOR
    }

    /// Magnetic field in Y-direction in nT (nano-Tesla).
    #[inline]
    pub const fn y_nt(&self) -> i32 {
        (self.y_unscaled() as i32) * Self::SCALING_FACTOR
    }

    /// Magnetic field in Z-direction in nT (nano-Tesla).
    #[inline]
    pub const fn z_nt(&self) -> i32 {
        (self.z_unscaled() as i32) * Self::SCALING_FACTOR
    }

    /// Magnetic field in X-, Y- and Z-directions in nT (nano-Tesla).
    #[inline]
    pub const fn xyz_nt(&self) -> (i32, i32, i32) {
        (self.x_nt(), self.y_nt(), self.z_nt())
    }
}

/// Accelerometer output data rate
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AccelOutputDataRate {
    /// 1 Hz (High-resolution/Normal/Low-power)
    Hz1,
    /// 10 Hz (High-resolution/Normal/Low-power)
    Hz10,
    /// 25 Hz (High-resolution/Normal/Low-power)
    Hz25,
    /// 50 Hz (High-resolution/Normal/Low-power)
    Hz50,
    /// 100 Hz (High-resolution/Normal/Low-power)
    Hz100,
    /// 200 Hz (High-resolution/Normal/Low-power)
    Hz200,
    /// 400 Hz (High-resolution/Normal/Low-power)
    Hz400,
    /// 1.344 kHz (High-resolution/Normal)
    Khz1_344,
    /// 1.620 kHz (Low-power)
    Khz1_620LowPower,
    /// 5.376 kHz (Low-power)
    Khz5_376LowPower,
}

impl AccelOutputDataRate {
    /// 1/ODR ms
    pub(crate) const fn turn_on_time_us_frac_1(&self) -> u32 {
        match self {
            Self::Hz1 => 1000,
            Self::Hz10 => 100,
            Self::Hz25 => 40,
            Self::Hz50 => 20,
            Self::Hz100 => 10,
            Self::Hz200 => 5,
            Self::Hz400 => 3,            // 2.5
            Self::Khz1_344 => 1,         // ~0.7
            Self::Khz1_620LowPower => 1, // ~0.6
            Self::Khz5_376LowPower => 1, // ~0.2
        }
    }

    /// 7/ODR ms
    pub(crate) const fn turn_on_time_us_frac_7(&self) -> u32 {
        match self {
            Self::Hz1 => 7000,
            Self::Hz10 => 700,
            Self::Hz25 => 280,
            Self::Hz50 => 140,
            Self::Hz100 => 70,
            Self::Hz200 => 35,
            Self::Hz400 => 18,           // 17.5
            Self::Khz1_344 => 6,         // ~5.2
            Self::Khz1_620LowPower => 5, // ~4.3
            Self::Khz5_376LowPower => 2, // ~1.3
        }
    }
}

/// Accelerometer mode
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AccelMode {
    /// Power down
    PowerDown,
    /// Low power (8-bit)
    LowPower,
    /// Normal mode (10-bit)
    Normal,
    /// High resolution (12-bit)
    HighResolution,
}

impl AccelMode {
    pub(crate) const fn turn_on_time_us(&self, odr: AccelOutputDataRate) -> u32 {
        match self {
            Self::PowerDown => 0,
            Self::LowPower => 1000,
            Self::Normal => 1600,
            Self::HighResolution => odr.turn_on_time_us_frac_7(),
        }
    }

    pub(crate) const fn change_time_us(&self, other: AccelMode, odr: AccelOutputDataRate) -> u32 {
        match (self, other) {
            (Self::HighResolution, Self::LowPower) => odr.turn_on_time_us_frac_1(),
            (Self::HighResolution, Self::Normal) => odr.turn_on_time_us_frac_1(),
            (Self::Normal, Self::LowPower) => odr.turn_on_time_us_frac_1(),
            (Self::Normal, Self::HighResolution) => odr.turn_on_time_us_frac_7(),
            (Self::LowPower, Self::Normal) => odr.turn_on_time_us_frac_1(),
            (Self::LowPower, Self::HighResolution) => odr.turn_on_time_us_frac_7(),
            (Self::PowerDown, new_mode) => new_mode.turn_on_time_us(odr),
            _ => 0,
        }
    }

    pub(crate) const fn resolution_factor(&self) -> i16 {
        match self {
            Self::PowerDown => 1,
            Self::HighResolution => 1 << 4,
            Self::Normal => 1 << 6,
            Self::LowPower => 1 << 8,
        }
    }

    pub(crate) const fn scaling_factor(&self, scale: AccelScale) -> i32 {
        match self {
            Self::PowerDown => 0,
            Self::HighResolution => scale as i32 / 2,
            Self::Normal => scale as i32 * 2,
            Self::LowPower => scale as i32 * 8,
        }
    }
}

/// Accelerometer scaling factor
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AccelScale {
    /// Plus or minus 2g
    G2 = 2,
    /// Plus or minus 4g
    G4 = 4,
    /// Plus or minus 8g
    G8 = 8,
    /// Plus or minus 16g
    G16 = 16,
}

/// Magnetometer output data rate
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MagOutputDataRate {
    /// 10 Hz
    Hz10,
    /// 20 Hz
    Hz20,
    /// 50 Hz
    Hz50,
    /// 100 Hz
    Hz100,
}

bitflags! {
    #[derive(Default)]
    struct StatusFlags: u8 {
        const XDA   = 1 << 0;
        const YDA   = 1 << 1;
        const ZDA   = 1 << 2;
        const ZYXDA = 1 << 3;
        const XOR   = 1 << 4;
        const YOR   = 1 << 5;
        const ZOR   = 1 << 6;
        const ZYXOR = 1 << 7;
    }
}

/// Data status
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Status {
    flags: StatusFlags,
}

impl Status {
    pub(crate) const fn new(flags: u8) -> Self {
        Self {
            flags: StatusFlags::from_bits_truncate(flags),
        }
    }

    /// X-axis new data available.
    #[inline]
    pub const fn x_new_data(&self) -> bool {
        self.flags.contains(StatusFlags::XDA)
    }

    /// Y-axis new data available.
    #[inline]
    pub const fn y_new_data(&self) -> bool {
        self.flags.contains(StatusFlags::YDA)
    }

    /// Z-axis new data available.
    #[inline]
    pub const fn z_new_data(&self) -> bool {
        self.flags.contains(StatusFlags::ZDA)
    }

    /// X-, Y- and Z-axis new data available.
    #[inline]
    pub const fn xyz_new_data(&self) -> bool {
        self.flags.contains(StatusFlags::ZYXDA)
    }

    /// X-axis data overrun.
    #[inline]
    pub const fn x_overrun(&self) -> bool {
        self.flags.contains(StatusFlags::XOR)
    }

    /// Y-axis data overrun.
    #[inline]
    pub const fn y_overrun(&self) -> bool {
        self.flags.contains(StatusFlags::YOR)
    }

    /// Z-axis data overrun.
    #[inline]
    pub const fn z_overrun(&self) -> bool {
        self.flags.contains(StatusFlags::ZOR)
    }

    /// X-, Y- and Z-axis data overrun.
    #[inline]
    pub const fn xyz_overrun(&self) -> bool {
        self.flags.contains(StatusFlags::ZYXOR)
    }
}

bitflags! {
    #[derive(Default)]
    struct TemperatureStatusFlags: u8 {
        const TDA = 1 << 2;
        const TOR = 1 << 6;
    }
}

/// Temperature sensor status
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct TemperatureStatus {
    flags: TemperatureStatusFlags,
}

impl TemperatureStatus {
    pub(crate) const fn new(flags: u8) -> Self {
        Self {
            flags: TemperatureStatusFlags::from_bits_truncate(flags),
        }
    }

    /// Temperature data overrun.
    #[inline]
    pub const fn overrun(&self) -> bool {
        self.flags.contains(TemperatureStatusFlags::TOR)
    }

    /// Temperature new data available.
    #[inline]
    pub const fn new_data(&self) -> bool {
        self.flags.contains(TemperatureStatusFlags::TDA)
    }
}

/// A temperature measurement.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Temperature {
    pub(crate) raw: u16,
}

impl Temperature {
    const DEFAULT: f32 = 25.0;

    /// Raw temperature.
    #[inline]
    pub const fn raw(&self) -> u16 {
        self.raw
    }

    /// Unscaled temperature.
    #[inline]
    pub const fn unscaled(&self) -> i16 {
        self.raw as i16
    }

    /// Temperature in °C.
    #[inline]
    pub fn degrees_celsius(&self) -> f32 {
        (self.unscaled() as f32) / 256.0 + Self::DEFAULT
    }
}
