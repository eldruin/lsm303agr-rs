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
    /// Magnetometer one-shot (single) mode
    #[derive(Debug)]
    pub struct MagOneShot;
    /// Magnetometer continuous mode
    #[derive(Debug)]
    pub struct MagContinuous;
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

/// Accelerometer scaling factor
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AccelScale {
    /// Plus or minus 2g
    G2,
    /// Plus or minus 4g
    G4,
    /// Plus or minus 8g
    G8,
    /// Plus or minus 16g
    G16,
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
