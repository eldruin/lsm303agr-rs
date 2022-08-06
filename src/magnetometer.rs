use crate::{
    interface::{ReadData, WriteData},
    mode,
    register_address::{CfgRegAM, CfgRegBM},
    Error, Lsm303agr, MagOutputDataRate, MagneticField,
};

impl<DI, CommE, PinE, MODE> Lsm303agr<DI, MODE>
where
    DI: ReadData<Error = Error<CommE, PinE>> + WriteData<Error = Error<CommE, PinE>>,
{
    /// Set magnetometer output data rate
    pub fn set_mag_odr(&mut self, odr: MagOutputDataRate) -> Result<(), Error<CommE, PinE>> {
        let cfg = self.cfg_reg_a_m.with_odr(odr);
        self.iface.write_mag_register(cfg)?;
        self.cfg_reg_a_m = cfg;
        Ok(())
    }
}

impl<DI, CommE, PinE> Lsm303agr<DI, mode::MagContinuous>
where
    DI: ReadData<Error = Error<CommE, PinE>> + WriteData<Error = Error<CommE, PinE>>,
{
    /// Get the measured magnetic field.
    pub fn magnetic_field(&mut self) -> Result<MagneticField, Error<CommE, PinE>> {
        self.iface.read_mag_3_double_registers::<MagneticField>()
    }

    /// Enable the magnetometer's built in offset cancellation.
    ///
    /// Offset cancellation is **automatically** managed by the device in **continuous** mode.
    ///
    /// To later disable offset cancellation, use the [`disable_mag_offset_cancellation`](Lsm303agr::disable_mag_offset_cancellation) function
    pub fn enable_mag_offset_cancellation(&mut self) -> Result<(), Error<CommE, PinE>> {
        let reg_b = self.cfg_reg_b_m | CfgRegBM::OFF_CANC;

        self.iface.write_mag_register(reg_b)?;
        self.cfg_reg_b_m = reg_b;

        Ok(())
    }

    /// Disable the magnetometer's built in offset cancellation.
    pub fn disable_mag_offset_cancellation(&mut self) -> Result<(), Error<CommE, PinE>> {
        let reg_b = self.cfg_reg_b_m & !CfgRegBM::OFF_CANC;

        self.iface.write_mag_register(reg_b)?;
        self.cfg_reg_b_m = reg_b;

        Ok(())
    }
}

impl<DI, CommE, PinE> Lsm303agr<DI, mode::MagOneShot>
where
    DI: ReadData<Error = Error<CommE, PinE>> + WriteData<Error = Error<CommE, PinE>>,
{
    /// Get the measured magnetic field.
    pub fn magnetic_field(&mut self) -> nb::Result<MagneticField, Error<CommE, PinE>> {
        let status = self.mag_status()?;
        if status.xyz_new_data() {
            Ok(self.iface.read_mag_3_double_registers::<MagneticField>()?)
        } else {
            let cfg = self.iface.read_mag_register::<CfgRegAM>()?;
            if !cfg.is_single_mode() {
                // Switch to single mode.
                let cfg = self.cfg_reg_a_m.single_mode();
                self.iface.write_mag_register(cfg)?;
                self.cfg_reg_a_m = cfg;
            }
            Err(nb::Error::WouldBlock)
        }
    }

    /// Enable the magnetometer's built in offset cancellation.
    ///
    /// Offset cancellation has to be **managed by the user** in **single measurement** (OneShot) mode averaging
    /// two consecutive measurements H<sub>n</sub> and H<sub>n-1</sub>.
    ///
    /// To later disable offset cancellation, use the [`disable_mag_offset_cancellation`](Lsm303agr::disable_mag_offset_cancellation) function
    pub fn enable_mag_offset_cancellation(&mut self) -> Result<(), Error<CommE, PinE>> {
        let reg_b = self.cfg_reg_b_m | CfgRegBM::OFF_CANC | CfgRegBM::OFF_CANC_ONE_SHOT;

        self.iface.write_mag_register(reg_b)?;
        self.cfg_reg_b_m = reg_b;

        Ok(())
    }

    /// Disable the magnetometer's built in offset cancellation.
    pub fn disable_mag_offset_cancellation(&mut self) -> Result<(), Error<CommE, PinE>> {
        let reg_b = self.cfg_reg_b_m & !(CfgRegBM::OFF_CANC | CfgRegBM::OFF_CANC_ONE_SHOT);

        self.iface.write_mag_register(reg_b)?;
        self.cfg_reg_b_m = reg_b;

        Ok(())
    }
}
