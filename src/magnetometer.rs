use crate::{
    interface::{ReadData, WriteData},
    mode,
    register_address::BitFlags,
    Error, Lsm303agr, MagOutputDataRate, MagneticField, Register,
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

impl<DI, CommE, PinE> Lsm303agr<DI, mode::MagContinuous>
where
    DI: ReadData<Error = Error<CommE, PinE>> + WriteData<Error = Error<CommE, PinE>>,
{
    /// Get the measured magnetic field.
    pub fn magnetic_field(&mut self) -> Result<MagneticField, Error<CommE, PinE>> {
        let (x, y, z) = self
            .iface
            .read_mag_3_double_registers(Register::OUTX_L_REG_M)?;

        Ok(MagneticField { x, y, z })
    }

    /// Enable the magnetometer's built in offset cancellation.
    ///
    /// Offset cancellation is **automatically** managed by the device in **continuous** mode.
    ///
    /// To later disable offset cancellation, use the [`disable_mag_offset_cancellation`](Lsm303agr::disable_mag_offset_cancellation) function
    pub fn enable_mag_offset_cancellation(&mut self) -> Result<(), Error<CommE, PinE>> {
        let reg_b = self.cfg_reg_b_m.with_high(BitFlags::MAG_OFF_CANC);

        self.iface
            .write_mag_register(Register::CFG_REG_B_M, reg_b.bits)?;
        self.cfg_reg_b_m = reg_b;

        Ok(())
    }

    /// Disable the magnetometer's built in offset cancellation.
    pub fn disable_mag_offset_cancellation(&mut self) -> Result<(), Error<CommE, PinE>> {
        let reg_b = self.cfg_reg_b_m.with_low(BitFlags::MAG_OFF_CANC);

        self.iface
            .write_mag_register(Register::CFG_REG_B_M, reg_b.bits)?;
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
            let (x, y, z) = self
                .iface
                .read_mag_3_double_registers(Register::OUTX_L_REG_M)?;

            Ok(MagneticField { x, y, z })
        } else {
            let cfg = self.iface.read_mag_register(Register::CFG_REG_A_M)?;
            if (cfg & 0x3) != 0x1 {
                // start one-shot measurement
                let cfg = (self.cfg_reg_a_m.bits & 0xFC) | 0x1;
                self.iface.write_mag_register(Register::CFG_REG_A_M, cfg)?;
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
        let reg_b = self
            .cfg_reg_b_m
            .with_high(BitFlags::MAG_OFF_CANC)
            .with_high(BitFlags::MAG_OFF_CANC_ONE_SHOT);

        self.iface
            .write_mag_register(Register::CFG_REG_B_M, reg_b.bits)?;
        self.cfg_reg_b_m = reg_b;

        Ok(())
    }

    /// Disable the magnetometer's built in offset cancellation.
    pub fn disable_mag_offset_cancellation(&mut self) -> Result<(), Error<CommE, PinE>> {
        let reg_b = self
            .cfg_reg_b_m
            .with_low(BitFlags::MAG_OFF_CANC)
            .with_low(BitFlags::MAG_OFF_CANC_ONE_SHOT);

        self.iface
            .write_mag_register(Register::CFG_REG_B_M, reg_b.bits)?;
        self.cfg_reg_b_m = reg_b;

        Ok(())
    }
}
