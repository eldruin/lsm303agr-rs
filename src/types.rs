use bitflags::bitflags;

use crate::register_address::{RegRead, StatusRegAuxA, WhoAmIA, WhoAmIM};

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
    raw: u8,
}

impl AccelerometerId {
    pub(crate) fn from_bits_truncate(raw: u8) -> Self {
        Self { raw }
    }

    /// Raw accelerometer ID.
    pub const fn raw(&self) -> u8 {
        self.raw
    }

    /// Check if the ID corresponds to the expected value.
    pub const fn is_correct(&self) -> bool {
        self.raw == WhoAmIA::ID
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

impl RegRead<(u16, u16, u16)> for Acceleration {
    type Output = (u16, u16, u16);

    /// OUT_X_L_A
    const ADDR: u8 = 0x28;

    #[inline(always)]
    fn from_data(data: (u16, u16, u16)) -> Self::Output {
        data
    }
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
    raw: u8,
}

impl MagnetometerId {
    pub(crate) fn from_bits_truncate(raw: u8) -> Self {
        Self { raw }
    }

    /// Raw magnetometer ID.
    pub const fn raw(&self) -> u8 {
        self.raw
    }

    /// Check if the ID corresponds to the expected value.
    pub const fn is_correct(&self) -> bool {
        self.raw == WhoAmIM::ID
    }
}

/// A magnetic field measurement.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct MagneticField {
    pub(crate) x: u16,
    pub(crate) y: u16,
    pub(crate) z: u16,
}

impl RegRead<(u16, u16, u16)> for MagneticField {
    type Output = Self;

    /// OUTX_L_REG_M
    const ADDR: u8 = 0x68;

    #[inline(always)]
    fn from_data((x, y, z): (u16, u16, u16)) -> Self::Output {
        Self { x, y, z }
    }
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
    /// 1 Hz (all modes)
    Hz1,
    /// 10 Hz all modes)
    Hz10,
    /// 25 Hz (all modes)
    Hz25,
    /// 50 Hz (all modes)
    Hz50,
    /// 100 Hz (all modes)
    Hz100,
    /// 200 Hz (all modes)
    Hz200,
    /// 400 Hz (all modes)
    Hz400,
    /// 1.344 kHz (only normal and high-resolution modes)
    Khz1_344,
    /// 1.620 kHz (only low-power mode)
    Khz1_620LowPower,
    /// 5.376 kHz (only low-power mode)
    Khz5_376LowPower,
}

impl AccelOutputDataRate {
    /// Create an `AccelOutputDataRate` with the given frequency in Hertz.
    pub const fn from_hertz(hz: u16) -> Option<Self> {
        Some(match hz {
            1 => Self::Hz1,
            10 => Self::Hz10,
            25 => Self::Hz25,
            50 => Self::Hz50,
            100 => Self::Hz100,
            200 => Self::Hz200,
            400 => Self::Hz400,
            1344 => Self::Khz1_344,
            1620 => Self::Khz1_620LowPower,
            5376 => Self::Khz5_376LowPower,
            _ => return None,
        })
    }

    /// 1/ODR ms
    pub(crate) const fn turn_on_time_us_frac_1(&self) -> u32 {
        match self {
            Self::Hz1 => 1000,
            Self::Hz10 => 100,
            Self::Hz25 => 40,
            Self::Hz50 => 20,
            Self::Hz100 => 10,
            Self::Hz200 => 5,
            Self::Hz400 => 3,            //  2.5
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

impl MagOutputDataRate {
    /// Create an `MagOutputDataRate` with the given frequency in Hertz.
    pub const fn from_hertz(hz: u16) -> Option<Self> {
        Some(match hz {
            10 => Self::Hz10,
            20 => Self::Hz20,
            50 => Self::Hz50,
            100 => Self::Hz100,
            _ => return None,
        })
    }

    /// 1/ODR ms
    pub(crate) const fn turn_on_time_us_frac_1(&self) -> u32 {
        match self {
            Self::Hz10 => 100,
            Self::Hz20 => 50,
            Self::Hz50 => 20,
            Self::Hz100 => 10,
        }
    }
}

/// Magnetometer mode
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MagMode {
    /// Low-power mode
    LowPower,
    /// High resolution mode
    HighResolution,
}

impl Default for MagMode {
    fn default() -> Self {
        Self::HighResolution
    }
}

impl MagMode {
    pub(crate) const fn turn_on_time_us(&self) -> u32 {
        match self {
            Self::LowPower => 9400,
            Self::HighResolution => 6400,
        }
    }
}

bitflags! {
    #[derive(Default)]
    pub struct StatusFlags: u8 {
        const ZYXOR = 0b10000000;
        const ZOR   = 0b01000000;
        const YOR   = 0b00100000;
        const XOR   = 0b00010000;
        const ZYXDA = 0b00001000;
        const ZDA   = 0b00000100;
        const YDA   = 0b00000010;
        const XDA   = 0b00000001;
    }
}

/// Data status
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Status {
    flags: StatusFlags,
}

impl Status {
    pub(crate) const fn new(flags: StatusFlags) -> Self {
        Self { flags }
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

/// Temperature sensor status
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct TemperatureStatus {
    flags: StatusRegAuxA,
}

impl TemperatureStatus {
    pub(crate) const fn new(flags: StatusRegAuxA) -> Self {
        Self { flags }
    }

    /// Temperature data overrun.
    #[inline]
    pub const fn overrun(&self) -> bool {
        self.flags.contains(StatusRegAuxA::TOR)
    }

    /// Temperature new data available.
    #[inline]
    pub const fn new_data(&self) -> bool {
        self.flags.contains(StatusRegAuxA::TDA)
    }
}

/// A temperature measurement.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Temperature {
    pub(crate) raw: u16,
}

impl RegRead<u16> for Temperature {
    type Output = Self;

    /// OUT_TEMP_L_A
    const ADDR: u8 = 0x0C;

    #[inline]
    fn from_data(data: u16) -> Self::Output {
        Temperature { raw: data }
    }
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

/// A FIFO mode.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FifoMode {
    /// Bypass mode
    Bypass,
    /// FIFO mode
    Fifo,
    /// Stream mode
    Stream,
    /// Stream-to-FIFO mode
    StreamToFifo,
}

/// An interrupt.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Interrupt {
    /// AOI1 interrupt on INT1 pin.
    Aoi1,
    /// AOI2 interrupt on INT1 pin.
    Aoi2,
    /// CLICK interrupt on INT1 pin.
    Click,
    /// DRDY1 interrupt on INT1 pin.
    DataReady1,
    /// DRDY2 interrupt on INT1 pin.
    DataReady2,
    /// FIFO overrun interrupt on INT1 pin.
    FifoOverrun,
    /// FIFO watermark interrupt on INT1 pin.
    FifoWatermark,
}
