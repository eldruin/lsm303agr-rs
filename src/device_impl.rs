use crate::{
    interface::{I2cInterface, ReadData, SpiInterface, WriteData},
    mode,
    register_address::{
        CfgRegAM, CfgRegBM, CfgRegCM, CtrlReg1A, CtrlReg3A, CtrlReg4A, CtrlReg5A, FifoCtrlRegA,
        StatusRegA, StatusRegAuxA, StatusRegM, TempCfgRegA, WhoAmIA, WhoAmIM,
    },
    Acceleration, AccelerometerId, Error, FifoMode, Interrupt, Lsm303agr, MagnetometerId,
    PhantomData, Status, Temperature, TemperatureStatus,
};

impl<I2C> Lsm303agr<I2cInterface<I2C>, mode::MagOneShot> {
    /// Create new instance of the LSM303AGR device communicating through I2C.
    pub fn new_with_i2c(i2c: I2C) -> Self {
        Lsm303agr {
            iface: I2cInterface { i2c },
            ctrl_reg1_a: CtrlReg1A::default(),
            ctrl_reg3_a: CtrlReg3A::default(),
            ctrl_reg4_a: CtrlReg4A::default(),
            ctrl_reg5_a: CtrlReg5A::default(),
            cfg_reg_a_m: CfgRegAM::default(),
            cfg_reg_b_m: CfgRegBM::default(),
            cfg_reg_c_m: CfgRegCM::default(),
            temp_cfg_reg_a: TempCfgRegA::default(),
            fifo_ctrl_reg_a: FifoCtrlRegA::default(),
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

impl<SPIXL, SPIMAG> Lsm303agr<SpiInterface<SPIXL, SPIMAG>, mode::MagOneShot> {
    /// Create new instance of the LSM303AGR device communicating through SPI.
    pub fn new_with_spi(spi_accel: SPIXL, spi_mag: SPIMAG) -> Self {
        Lsm303agr {
            iface: SpiInterface {
                spi_xl: spi_accel,
                spi_mag,
            },
            ctrl_reg1_a: CtrlReg1A::default(),
            ctrl_reg3_a: CtrlReg3A::default(),
            ctrl_reg4_a: CtrlReg4A::default(),
            ctrl_reg5_a: CtrlReg5A::default(),
            cfg_reg_a_m: CfgRegAM::default(),
            cfg_reg_b_m: CfgRegBM::default(),
            cfg_reg_c_m: CfgRegCM::default(),
            temp_cfg_reg_a: TempCfgRegA::default(),
            fifo_ctrl_reg_a: FifoCtrlRegA::default(),
            accel_odr: None,
            _mag_mode: PhantomData,
        }
    }
}

impl<SPIXL, SPIMAG, MODE> Lsm303agr<SpiInterface<SPIXL, SPIMAG>, MODE> {
    /// Destroy driver instance, return SPI bus instance and chip select pin.
    pub fn destroy(self) -> (SPIXL, SPIMAG) {
        (self.iface.spi_xl, self.iface.spi_mag)
    }
}

impl<DI, CommE, MODE> Lsm303agr<DI, MODE>
where
    DI: ReadData<Error = Error<CommE>> + WriteData<Error = Error<CommE>>,
{
    /// Initialize registers
    pub fn init(&mut self) -> Result<(), Error<CommE>> {
        self.acc_enable_temp()?; // Also enables BDU.
        self.mag_enable_bdu()
    }

    /// Enable block data update for accelerometer.
    #[inline]
    fn acc_enable_bdu(&mut self) -> Result<(), Error<CommE>> {
        let reg4 = self.ctrl_reg4_a | CtrlReg4A::BDU;
        self.iface.write_accel_register(reg4)?;
        self.ctrl_reg4_a = reg4;

        Ok(())
    }

    /// Enable the temperature sensor.
    #[inline]
    fn acc_enable_temp(&mut self) -> Result<(), Error<CommE>> {
        self.acc_enable_bdu()?;

        let temp_cfg_reg = self.temp_cfg_reg_a | TempCfgRegA::TEMP_EN;
        self.iface.write_accel_register(temp_cfg_reg)?;
        self.temp_cfg_reg_a = temp_cfg_reg;

        Ok(())
    }

    /// Enable block data update for magnetometer.
    #[inline]
    fn mag_enable_bdu(&mut self) -> Result<(), Error<CommE>> {
        let regc = self.cfg_reg_c_m | CfgRegCM::BDU;
        self.iface.write_mag_register(regc)?;
        self.cfg_reg_c_m = regc;

        Ok(())
    }

    /// Set the accelerometer FIFO mode and full threshold.
    ///
    /// The threshold is clamped to \[0, 31\].
    pub fn acc_set_fifo_mode(&mut self, mode: FifoMode, fth: u8) -> Result<(), Error<CommE>> {
        let mut reg5 = self.ctrl_reg5_a;
        reg5.set(CtrlReg5A::FIFO_EN, mode != FifoMode::Bypass);
        self.iface.write_accel_register(reg5)?;
        self.ctrl_reg5_a = reg5;

        let fifo_ctrl = self
            .fifo_ctrl_reg_a
            .with_mode(mode)
            .with_full_threshold(fth);
        self.iface.write_accel_register(fifo_ctrl)?;
        self.fifo_ctrl_reg_a = fifo_ctrl;

        Ok(())
    }

    /// Enable accelerometer interrupt.
    pub fn acc_enable_interrupt(&mut self, interrupt: Interrupt) -> Result<(), Error<CommE>> {
        let reg3 = self.ctrl_reg3_a.with_interrupt(interrupt);
        self.iface.write_accel_register(reg3)?;
        self.ctrl_reg3_a = reg3;

        Ok(())
    }

    /// Disable accelerometer interrupt.
    pub fn acc_disable_interrupt(&mut self, interrupt: Interrupt) -> Result<(), Error<CommE>> {
        let reg3 = self.ctrl_reg3_a.without_interrupt(interrupt);
        self.iface.write_accel_register(reg3)?;
        self.ctrl_reg3_a = reg3;

        Ok(())
    }

    /// Configure the DRDY pin as a digital output.
    pub fn mag_enable_int(&mut self) -> Result<(), Error<CommE>> {
        let regc = self.cfg_reg_c_m | CfgRegCM::INT_MAG;
        self.iface.write_mag_register(regc)?;
        self.cfg_reg_c_m = regc;

        Ok(())
    }

    /// Enable magnetometer low-pass filter.
    pub fn mag_enable_low_pass_filter(&mut self) -> Result<(), Error<CommE>> {
        let regb = self.cfg_reg_b_m.union(CfgRegBM::LPF);
        self.iface.write_mag_register(regb)?;
        self.cfg_reg_b_m = regb;

        Ok(())
    }

    /// Disable magnetometer low-pass filter.
    pub fn mag_disable_low_pass_filter(&mut self) -> Result<(), Error<CommE>> {
        let regb = self.cfg_reg_b_m.difference(CfgRegBM::LPF);
        self.iface.write_mag_register(regb)?;
        self.cfg_reg_b_m = regb;

        Ok(())
    }

    /// Accelerometer status
    pub fn accel_status(&mut self) -> Result<Status, Error<CommE>> {
        self.iface
            .read_accel_register::<StatusRegA>()
            .map(Status::new)
    }

    /// Get measured acceleration.
    pub fn acceleration(&mut self) -> Result<Acceleration, Error<CommE>> {
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
    pub fn mag_status(&mut self) -> Result<Status, Error<CommE>> {
        self.iface
            .read_mag_register::<StatusRegM>()
            .map(Status::new)
    }

    /// Get the accelerometer device ID.
    pub fn accelerometer_id(&mut self) -> Result<AccelerometerId, Error<CommE>> {
        self.iface.read_accel_register::<WhoAmIA>()
    }

    /// Get the magnetometer device ID.
    pub fn magnetometer_id(&mut self) -> Result<MagnetometerId, Error<CommE>> {
        self.iface.read_mag_register::<WhoAmIM>()
    }

    /// Get measured temperature.
    pub fn temperature(&mut self) -> Result<Temperature, Error<CommE>> {
        self.iface.read_accel_double_register::<Temperature>()
    }

    /// Temperature sensor status
    pub fn temperature_status(&mut self) -> Result<TemperatureStatus, Error<CommE>> {
        self.iface
            .read_accel_register::<StatusRegAuxA>()
            .map(TemperatureStatus::new)
    }
}
