use crate::{
    interface::{I2cInterface, ReadData, SpiInterface, WriteData},
    mode,
    register_address::{
        CfgRegAM, CfgRegBM, CfgRegCM, CtrlReg1A, CtrlReg4A, StatusRegA, StatusRegAuxA, StatusRegM,
        TempCfgRegA, WhoAmIA, WhoAmIM,
    },
    Acceleration, AccelerometerId, Error, Lsm303agr, MagnetometerId, PhantomData, Status,
    Temperature, TemperatureStatus,
};

impl<I2C> Lsm303agr<I2cInterface<I2C>, mode::MagOneShot> {
    /// Create new instance of the LSM303AGR device communicating through I2C.
    pub fn new_with_i2c(i2c: I2C) -> Self {
        Lsm303agr {
            iface: I2cInterface { i2c },
            ctrl_reg1_a: CtrlReg1A::default(),
            ctrl_reg4_a: CtrlReg4A::default(),
            cfg_reg_a_m: CfgRegAM::default(),
            cfg_reg_b_m: CfgRegBM::default(),
            cfg_reg_c_m: CfgRegCM::default(),
            temp_cfg_reg_a: TempCfgRegA::default(),
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
            ctrl_reg1_a: CtrlReg1A::default(),
            ctrl_reg4_a: CtrlReg4A::default(),
            cfg_reg_a_m: CfgRegAM::default(),
            cfg_reg_b_m: CfgRegBM::default(),
            cfg_reg_c_m: CfgRegCM::default(),
            temp_cfg_reg_a: TempCfgRegA::default(),
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
        self.acc_enable_temp()?; // Also enables BDU.
        self.mag_enable_bdu()
    }

    /// Enable block data update for accelerometer.
    fn acc_enable_bdu(&mut self) -> Result<(), Error<CommE, PinE>> {
        let reg4 = self.ctrl_reg4_a | CtrlReg4A::BDU;
        self.iface.write_accel_register(reg4)?;
        self.ctrl_reg4_a = reg4;

        Ok(())
    }

    /// Enable the temperature sensor.
    #[inline]
    fn acc_enable_temp(&mut self) -> Result<(), Error<CommE, PinE>> {
        self.acc_enable_bdu()?;

        let temp_cfg_reg = self.temp_cfg_reg_a | TempCfgRegA::TEMP_EN;
        self.iface.write_accel_register(temp_cfg_reg)?;
        self.temp_cfg_reg_a = temp_cfg_reg;

        Ok(())
    }

    /// Enable block data update for magnetometer.
    #[inline]
    fn mag_enable_bdu(&mut self) -> Result<(), Error<CommE, PinE>> {
        let regc = self.cfg_reg_c_m | CfgRegCM::BDU;
        self.iface.write_mag_register(regc)?;
        self.cfg_reg_c_m = regc;

        Ok(())
    }

    /// Configure the DRDY pin as a digital output.
    pub fn mag_enable_int(&mut self) -> Result<(), Error<CommE, PinE>> {
        let regc = self.cfg_reg_c_m | CfgRegCM::INT_MAG;
        self.iface.write_mag_register(regc)?;
        self.cfg_reg_c_m = regc;

        Ok(())
    }

    /// Accelerometer status
    pub fn accel_status(&mut self) -> Result<Status, Error<CommE, PinE>> {
        self.iface
            .read_accel_register::<StatusRegA>()
            .map(Status::new)
    }

    /// Get measured acceleration.
    pub fn acceleration(&mut self) -> Result<Acceleration, Error<CommE, PinE>> {
        let (x, y, z) = self.iface.read_accel_3_double_registers::<Acceleration>()?;

        Ok(Acceleration {
            x,
            y,
            z,
            mode: self.get_accel_mode(),
            scale: self.get_accel_scale(),
        })
    }

    /// Magnetometer status
    pub fn mag_status(&mut self) -> Result<Status, Error<CommE, PinE>> {
        self.iface
            .read_mag_register::<StatusRegM>()
            .map(Status::new)
    }

    /// Get the accelerometer device ID.
    pub fn accelerometer_id(&mut self) -> Result<AccelerometerId, Error<CommE, PinE>> {
        self.iface.read_accel_register::<WhoAmIA>()
    }

    /// Get the magnetometer device ID.
    pub fn magnetometer_id(&mut self) -> Result<MagnetometerId, Error<CommE, PinE>> {
        self.iface.read_mag_register::<WhoAmIM>()
    }

    /// Get measured temperature.
    pub fn temperature(&mut self) -> Result<Temperature, Error<CommE, PinE>> {
        self.iface.read_accel_double_register::<Temperature>()
    }

    /// Temperature sensor status
    pub fn temperature_status(&mut self) -> Result<TemperatureStatus, Error<CommE, PinE>> {
        self.iface
            .read_accel_register::<StatusRegAuxA>()
            .map(TemperatureStatus::new)
    }
}
