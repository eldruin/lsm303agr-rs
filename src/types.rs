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

/// Data status
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Status {
    /// X,Y,Z-axis data overrun
    pub xyz_overrun: bool,
    /// X-axis data overrun
    pub x_overrun: bool,
    /// Y-axis data overrun
    pub y_overrun: bool,
    /// Z-axis data overrun
    pub z_overrun: bool,
    /// X,Y,Z-axis new data ready
    pub xyz_new_data: bool,
    /// X-axis data new data ready
    pub x_new_data: bool,
    /// Y-axis data new data ready
    pub y_new_data: bool,
    /// Z-axis data new data ready
    pub z_new_data: bool,
}
