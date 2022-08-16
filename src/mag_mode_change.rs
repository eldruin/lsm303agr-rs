use crate::{
    interface::{ReadData, WriteData},
    mode, Error, Lsm303agr, ModeChangeError, PhantomData,
};

impl<DI, CommE, PinE> Lsm303agr<DI, mode::MagOneShot>
where
    DI: ReadData<Error = Error<CommE, PinE>> + WriteData<Error = Error<CommE, PinE>>,
{
    /// Change the magnetometer to continuous measurement mode
    pub fn into_mag_continuous(
        mut self,
    ) -> Result<Lsm303agr<DI, mode::MagContinuous>, ModeChangeError<CommE, PinE, Self>> {
        let cfg = self.cfg_reg_a_m.continuous_mode();
        match self.iface.write_mag_register(cfg) {
            Err(error) => Err(ModeChangeError { error, dev: self }),
            Ok(_) => Ok(Lsm303agr {
                iface: self.iface,
                ctrl_reg1_a: self.ctrl_reg1_a,
                ctrl_reg4_a: self.ctrl_reg4_a,
                cfg_reg_a_m: cfg,
                cfg_reg_b_m: self.cfg_reg_b_m,
                cfg_reg_c_m: self.cfg_reg_c_m,
                temp_cfg_reg_a: self.temp_cfg_reg_a,
                accel_odr: None,
                _mag_mode: PhantomData,
            }),
        }
    }
}

impl<DI, CommE, PinE> Lsm303agr<DI, mode::MagContinuous>
where
    DI: ReadData<Error = Error<CommE, PinE>> + WriteData<Error = Error<CommE, PinE>>,
{
    /// Change the magnetometer to one-shot mode
    ///
    /// After this the magnetometer is in idle mode until a one-shot measurement
    /// is started.
    pub fn into_mag_one_shot(
        mut self,
    ) -> Result<Lsm303agr<DI, mode::MagOneShot>, ModeChangeError<CommE, PinE, Self>> {
        let cfg = self.cfg_reg_a_m.idle_mode();
        match self.iface.write_mag_register(cfg) {
            Err(error) => Err(ModeChangeError { error, dev: self }),
            Ok(_) => Ok(Lsm303agr {
                iface: self.iface,
                ctrl_reg1_a: self.ctrl_reg1_a,
                ctrl_reg4_a: self.ctrl_reg4_a,
                cfg_reg_a_m: cfg,
                cfg_reg_b_m: self.cfg_reg_b_m,
                cfg_reg_c_m: self.cfg_reg_c_m,
                temp_cfg_reg_a: self.temp_cfg_reg_a,
                accel_odr: None,
                _mag_mode: PhantomData,
            }),
        }
    }
}
