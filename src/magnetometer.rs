use embedded_hal::delay::DelayNs;

use crate::{
    interface::{ReadData, WriteData},
    mode,
    register_address::{CfgRegAM, CfgRegBM},
    Error, Lsm303agr, MagMode, MagOutputDataRate, MagneticField,
};

impl<DI, CommE, MODE> Lsm303agr<DI, MODE>
where
    DI: ReadData<Error = Error<CommE>> + WriteData<Error = Error<CommE>>,
{
    /// Set magnetometer power/resolution mode and output data rate.
    ///
    #[doc = include_str!("delay.md")]
    pub fn set_mag_mode_and_odr<D: DelayNs>(
        &mut self,
        delay: &mut D,
        mode: MagMode,
        odr: MagOutputDataRate,
    ) -> Result<(), Error<CommE>> {
        let rega = self.cfg_reg_a_m;

        let old_mode = rega.mode();
        let old_odr = rega.odr();

        let rega = rega.with_mode(mode).with_odr(odr);
        self.iface.write_mag_register(rega)?;
        self.cfg_reg_a_m = rega;

        let offset_cancellation = self.cfg_reg_b_m.offset_cancellation();
        if old_mode != mode {
            delay.delay_us(rega.turn_on_time_us(offset_cancellation));
        } else if old_odr != odr && offset_cancellation {
            // Mode did not change, so only wait for 1/ODR ms.
            delay.delay_us(odr.turn_on_time_us_frac_1());
        }

        Ok(())
    }

    /// Get magnetometer power/resolution mode.
    pub fn get_mag_mode(&self) -> MagMode {
        self.cfg_reg_a_m.mode()
    }
}

impl<DI, CommE> Lsm303agr<DI, mode::MagContinuous>
where
    DI: ReadData<Error = Error<CommE>> + WriteData<Error = Error<CommE>>,
{
    /// Get the measured magnetic field.
    pub fn magnetic_field(&mut self) -> Result<MagneticField, Error<CommE>> {
        self.iface.read_mag_3_double_registers::<MagneticField>()
    }

    /// Enable the magnetometer's built in offset cancellation.
    ///
    /// Offset cancellation is **automatically** managed by the device in **continuous** mode.
    ///
    /// To later disable offset cancellation, use the [`disable_mag_offset_cancellation`](Lsm303agr::disable_mag_offset_cancellation) function
    pub fn enable_mag_offset_cancellation(&mut self) -> Result<(), Error<CommE>> {
        let reg_b = self.cfg_reg_b_m | CfgRegBM::OFF_CANC;

        self.iface.write_mag_register(reg_b)?;
        self.cfg_reg_b_m = reg_b;

        Ok(())
    }

    /// Disable the magnetometer's built in offset cancellation.
    pub fn disable_mag_offset_cancellation(&mut self) -> Result<(), Error<CommE>> {
        let reg_b = self.cfg_reg_b_m & !CfgRegBM::OFF_CANC;

        self.iface.write_mag_register(reg_b)?;
        self.cfg_reg_b_m = reg_b;

        Ok(())
    }
}

impl<DI, CommE> Lsm303agr<DI, mode::MagOneShot>
where
    DI: ReadData<Error = Error<CommE>> + WriteData<Error = Error<CommE>>,
{
    /// Get the measured magnetic field.
    pub fn magnetic_field(&mut self) -> nb::Result<MagneticField, Error<CommE>> {
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
    pub fn enable_mag_offset_cancellation(&mut self) -> Result<(), Error<CommE>> {
        let reg_b = self.cfg_reg_b_m | CfgRegBM::OFF_CANC | CfgRegBM::OFF_CANC_ONE_SHOT;

        self.iface.write_mag_register(reg_b)?;
        self.cfg_reg_b_m = reg_b;

        Ok(())
    }

    /// Disable the magnetometer's built in offset cancellation.
    pub fn disable_mag_offset_cancellation(&mut self) -> Result<(), Error<CommE>> {
        let reg_b = self.cfg_reg_b_m & !(CfgRegBM::OFF_CANC | CfgRegBM::OFF_CANC_ONE_SHOT);

        self.iface.write_mag_register(reg_b)?;
        self.cfg_reg_b_m = reg_b;

        Ok(())
    }
}
