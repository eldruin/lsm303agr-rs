use crate::{
    interface::{ReadData, WriteData},
    Error, Lsm303agr, MagOutputDataRate, Register,
};

impl<DI, CommE, PinE, MODE> Lsm303agr<DI, MODE>
where
    DI: ReadData<Error = Error<CommE, PinE>> + WriteData<Error = Error<CommE, PinE>>,
{
    /// Set magnetometer output data rate
    pub fn set_mag_odr(&mut self, odr: MagOutputDataRate) -> Result<(), Error<CommE, PinE>> {
        let cfg = self.cfg_reg_a_m.bits & 0xF3; // !(3 << 2);
        let mask = match odr {
            MagOutputDataRate::Hz10 => 0,
            MagOutputDataRate::Hz20 => 1 << 2,
            MagOutputDataRate::Hz50 => 2 << 2,
            MagOutputDataRate::Hz100 => 3 << 2,
        };
        self.iface
            .write_mag_register(Register::CFG_REG_A_M, cfg | mask)?;
        self.cfg_reg_a_m = (cfg | mask).into();
        Ok(())
    }
}
