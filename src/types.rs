use bitflags::bitflags;

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

/// Measurement
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Measurement {
    /// X-axis data.
    pub x: i32,
    /// Y-axis data.
    pub y: i32,
    /// Z-axis data.
    pub z: i32,
}

/// Unscaled measurement
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct UnscaledMeasurement {
    /// X-axis data.
    pub x: i16,
    /// Y-axis data.
    pub y: i16,
    /// Z-axis data.
    pub z: i16,
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
