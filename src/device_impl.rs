use crate::{
    interface::{I2cInterface, ReadData, SpiInterface, WriteData},
    mode,
    register_address::{WHO_AM_I_A_VAL, WHO_AM_I_M_VAL},
    AccelMode, AccelScale, BitFlags as BF, Config, Error, Lsm303agr, Measurement, PhantomData,
    Register, Status, UnscaledMeasurement,
};

impl<I2C> Lsm303agr<I2cInterface<I2C>, mode::MagOneShot> {
    /// Create new instance of the LSM303AGR device communicating through I2C.
    pub fn new_with_i2c(i2c: I2C) -> Self {
        Lsm303agr {
            iface: I2cInterface { i2c },
            ctrl_reg1_a: Config { bits: 0x7 },
            ctrl_reg4_a: Config { bits: 0 },
            cfg_reg_a_m: Config { bits: 0x3 },
            cfg_reg_c_m: Config { bits: 0 },
            accel_odr: None,
            _mag_mode: PhantomData,
        }
    }
}

impl<I2C, MODE> Lsm303agr<I2cInterface<I2C>, MODE> {
    /// Destroy driver instance, return I2C bus.
    pub fn destroy(self) -> I2C {
        self.iface.i2c
    }
}

impl<SPI, CSXL, CSMAG> Lsm303agr<SpiInterface<SPI, CSXL, CSMAG>, mode::MagOneShot> {
    /// Create new instance of the LSM303AGR device communicating through SPI.
    pub fn new_with_spi(spi: SPI, chip_select_accel: CSXL, chip_select_mag: CSMAG) -> Self {
        Lsm303agr {
            iface: SpiInterface {
                spi,
                cs_xl: chip_select_accel,
                cs_mag: chip_select_mag,
            },
            ctrl_reg1_a: Config { bits: 0x7 },
            ctrl_reg4_a: Config { bits: 0 },
            cfg_reg_a_m: Config { bits: 0x3 },
            cfg_reg_c_m: Config { bits: 0 },
            accel_odr: None,
            _mag_mode: PhantomData,
        }
    }
}

impl<SPI, CSXL, CSMAG, MODE> Lsm303agr<SpiInterface<SPI, CSXL, CSMAG>, MODE> {
    /// Destroy driver instance, return SPI bus instance and chip select pin.
    pub fn destroy(self) -> (SPI, CSXL, CSMAG) {
        (self.iface.spi, self.iface.cs_xl, self.iface.cs_mag)
    }
}

impl<DI, CommE, PinE, MODE> Lsm303agr<DI, MODE>
where
    DI: ReadData<Error = Error<CommE, PinE>> + WriteData<Error = Error<CommE, PinE>>,
{
    /// Initialize registers
    pub fn init(&mut self) -> Result<(), Error<CommE, PinE>> {
        let reg4 = self.ctrl_reg4_a.with_high(BF::ACCEL_BDU);
        self.iface
            .write_accel_register(Register::CTRL_REG4_A, reg4.bits)?;
        self.ctrl_reg4_a = reg4;
        let regc = self.cfg_reg_c_m.with_high(BF::MAG_BDU);
        self.iface
            .write_mag_register(Register::CFG_REG_C_M, regc.bits)?;
        self.cfg_reg_c_m = regc;
        Ok(())
    }

    /// Accelerometer status
    pub fn accel_status(&mut self) -> Result<Status, Error<CommE, PinE>> {
        self.iface
            .read_accel_register(Register::STATUS_REG_A)
            .map(convert_status)
    }

    /// Accelerometer data
    ///
    /// Returned in mg (milli-g) where 1g is 9.8m/s².
    ///
    /// If you need the raw unscaled measurement see [`Lsm303agr::accel_data_unscaled`].
    pub fn accel_data(&mut self) -> Result<Measurement, Error<CommE, PinE>> {
        let unscaled = self.accel_data_unscaled()?;

        let mode = self.get_accel_mode();
        let scale = self.get_accel_scale();

        let scaling_factor = match mode {
            AccelMode::PowerDown => 0,
            AccelMode::HighResolution => match scale {
                AccelScale::G2 => 1,
                AccelScale::G4 => 2,
                AccelScale::G8 => 4,
                AccelScale::G16 => 8,
            },
            AccelMode::LowPower => match scale {
                AccelScale::G2 => 16,
                AccelScale::G4 => 32,
                AccelScale::G8 => 64,
                AccelScale::G16 => 128,
            },
            AccelMode::Normal => match scale {
                AccelScale::G2 => 4,
                AccelScale::G4 => 8,
                AccelScale::G8 => 16,
                AccelScale::G16 => 32,
            },
        };

        Ok(Measurement {
            x: (unscaled.x as i32) * scaling_factor,
            y: (unscaled.y as i32) * scaling_factor,
            z: (unscaled.z as i32) * scaling_factor,
        })
    }

    /// Unscaled accelerometer data
    pub fn accel_data_unscaled(&mut self) -> Result<UnscaledMeasurement, Error<CommE, PinE>> {
        let data = self
            .iface
            .read_accel_3_double_registers(Register::OUT_X_L_A)?;

        let mode = self.get_accel_mode();

        let resolution_factor = match mode {
            AccelMode::PowerDown => 1,
            AccelMode::HighResolution => 1 << 4,
            AccelMode::LowPower => 1 << 8,
            AccelMode::Normal => 1 << 6,
        };

        Ok(UnscaledMeasurement {
            x: (data.0 as i16) / resolution_factor,
            y: (data.1 as i16) / resolution_factor,
            z: (data.2 as i16) / resolution_factor,
        })
    }

    /// Magnetometer status
    pub fn mag_status(&mut self) -> Result<Status, Error<CommE, PinE>> {
        self.iface
            .read_mag_register(Register::STATUS_REG_M)
            .map(convert_status)
    }

    /// Get accelerometer device ID
    pub fn accelerometer_id(&mut self) -> Result<u8, Error<CommE, PinE>> {
        self.iface.read_accel_register(Register::WHO_AM_I_A)
    }

    /// Read and verify the accelerometer device ID
    pub fn accelerometer_is_detected(&mut self) -> Result<bool, Error<CommE, PinE>> {
        Ok(self.accelerometer_id()? == WHO_AM_I_A_VAL)
    }

    /// Get magnetometer device ID
    pub fn magnetometer_id(&mut self) -> Result<u8, Error<CommE, PinE>> {
        self.iface.read_mag_register(Register::WHO_AM_I_M)
    }

    /// Read and verify the magnetometer device ID
    pub fn magnetometer_is_detected(&mut self) -> Result<bool, Error<CommE, PinE>> {
        Ok(self.magnetometer_id()? == WHO_AM_I_M_VAL)
    }
}

fn convert_status(st: u8) -> Status {
    Status {
        xyz_overrun: (st & BF::XYZOR) != 0,
        z_overrun: (st & BF::ZOR) != 0,
        y_overrun: (st & BF::YOR) != 0,
        x_overrun: (st & BF::XOR) != 0,
        xyz_new_data: (st & BF::XYZDR) != 0,
        z_new_data: (st & BF::ZDR) != 0,
        y_new_data: (st & BF::YDR) != 0,
        x_new_data: (st & BF::XDR) != 0,
    }
}
