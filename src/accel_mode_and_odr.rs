use crate::{
    interface::{ReadData, WriteData},
    AccelMode, AccelOutputDataRate, AccelScale, BitFlags as BF, Error, Lsm303agr, Register,
};

impl<DI, CommE, PinE, MODE> Lsm303agr<DI, MODE>
where
    DI: ReadData<Error = Error<CommE, PinE>> + WriteData<Error = Error<CommE, PinE>>,
{
    /// Set accelerometer output data rate.
    ///
    /// This changes the power mode if the current one is not appropriate.
    /// When changing from a low-power-only output data rate setting into
    /// a high-resolution or normal power mode, it changes into normal mode.
    pub fn set_accel_odr(&mut self, odr: AccelOutputDataRate) -> Result<(), Error<CommE, PinE>> {
        let (mask, lp_only, lp_compat) = match odr {
            AccelOutputDataRate::Hz1 => (1 << 4, false, true),
            AccelOutputDataRate::Hz10 => (2 << 4, false, true),
            AccelOutputDataRate::Hz25 => (3 << 4, false, true),
            AccelOutputDataRate::Hz50 => (4 << 4, false, true),
            AccelOutputDataRate::Hz100 => (5 << 4, false, true),
            AccelOutputDataRate::Hz200 => (6 << 4, false, true),
            AccelOutputDataRate::Hz400 => (7 << 4, false, true),
            AccelOutputDataRate::Khz1_620LowPower => (8 << 4, true, true),
            AccelOutputDataRate::Khz1_344 => (9 << 4, false, false),
            AccelOutputDataRate::Khz5_376LowPower => (9 << 4, true, true),
        };
        let lp_enabled = self.ctrl_reg1_a.is_high(BF::LP_EN);
        let hr_enabled = self.ctrl_reg4_a.is_high(BF::HR);
        let mut should_lp_be_enabled = lp_enabled;
        if lp_enabled {
            if !lp_compat {
                // HR could not have been enabled.
                should_lp_be_enabled = false;
            }
        } else {
            // Currently normal or HR mode
            if lp_only {
                if hr_enabled {
                    self.disable_hr()?;
                }
                // power mode is (now) normal
                should_lp_be_enabled = true;
            }
        }
        let lp_flag = if should_lp_be_enabled { BF::LP_EN } else { 0 };
        let reg1 = (self.ctrl_reg1_a.bits & !(BF::LP_EN | (0x7 << 4))) | mask | lp_flag;
        self.iface
            .write_accel_register(Register::CTRL_REG1_A, reg1)?;
        self.ctrl_reg1_a = reg1.into();
        self.accel_odr = Some(odr);
        Ok(())
    }

    /// Set accelerometer power/resolution mode
    ///
    /// Returns `Error::InvalidInputData` if the mode is incompatible with the current
    /// accelerometer output data rate.
    pub fn set_accel_mode(&mut self, mode: AccelMode) -> Result<(), Error<CommE, PinE>> {
        check_accel_odr_is_compatible_with_mode(self.accel_odr, mode)?;

        match mode {
            AccelMode::HighResolution => {
                self.disable_lp()?;
                self.enable_hr()?;
            }
            AccelMode::Normal => {
                self.disable_lp()?;
                self.disable_hr()?;
            }
            AccelMode::LowPower => {
                self.disable_hr()?;
                self.enable_lp()?;
            }
            AccelMode::PowerDown => {
                let reg1 = self.ctrl_reg1_a.bits & !(0xf << 4);
                self.iface
                    .write_accel_register(Register::CTRL_REG1_A, reg1)?;
                self.ctrl_reg1_a = reg1.into();
                self.accel_odr = None;
            }
        }
        Ok(())
    }

    /// Get the accelerometer mode
    pub fn get_accel_mode(&mut self) -> AccelMode {
        let power_down = (self.ctrl_reg1_a.bits >> 4 & 0xf) == 0;
        let lp_enabled = self.ctrl_reg1_a.is_high(BF::LP_EN);
        let hr_enabled = self.ctrl_reg4_a.is_high(BF::HR);

        if power_down {
            AccelMode::PowerDown
        } else if hr_enabled {
            AccelMode::HighResolution
        } else if lp_enabled {
            AccelMode::LowPower
        } else {
            AccelMode::Normal
        }
    }

    /// Set accelerometer scaling factor
    ///
    /// This changes the scale at which the acceleration is read.
    /// `AccelScale::G2` for example can return values between -2g and +2g
    /// where g is the gravity of the earth (~9.82 m/s²).
    pub fn set_accel_scale(&mut self, scale: AccelScale) -> Result<(), Error<CommE, PinE>> {
        let fs = match scale {
            AccelScale::G2 => 0b00,
            AccelScale::G4 => 0b01,
            AccelScale::G8 => 0b10,
            AccelScale::G16 => 0b11,
        };
        let reg4 = self.ctrl_reg4_a.bits & !(0b11 << 4) | (fs << 4);
        self.iface
            .write_accel_register(Register::CTRL_REG4_A, reg4)?;
        self.ctrl_reg4_a = reg4.into();
        Ok(())
    }

    /// Get accelerometer scaling factor
    pub fn get_accel_scale(&self) -> AccelScale {
        let fs = (self.ctrl_reg4_a.bits & (0b11 << 4)) >> 4;
        match fs {
            0b00 => AccelScale::G2,
            0b01 => AccelScale::G4,
            0b10 => AccelScale::G8,
            0b11 => AccelScale::G16,
            _ => unreachable!("bit shift above means we cannot be here"),
        }
    }

    fn enable_hr(&mut self) -> Result<(), Error<CommE, PinE>> {
        let reg4 = self.ctrl_reg4_a.with_high(BF::HR);
        self.iface
            .write_accel_register(Register::CTRL_REG4_A, reg4.bits)?;
        self.ctrl_reg4_a = reg4;
        Ok(())
    }

    fn disable_hr(&mut self) -> Result<(), Error<CommE, PinE>> {
        let reg4 = self.ctrl_reg4_a.with_low(BF::HR);
        self.iface
            .write_accel_register(Register::CTRL_REG4_A, reg4.bits)?;
        self.ctrl_reg4_a = reg4;
        Ok(())
    }

    fn enable_lp(&mut self) -> Result<(), Error<CommE, PinE>> {
        let reg1 = self.ctrl_reg1_a.with_high(BF::LP_EN);
        self.iface
            .write_accel_register(Register::CTRL_REG1_A, reg1.bits)?;
        self.ctrl_reg1_a = reg1;
        Ok(())
    }

    fn disable_lp(&mut self) -> Result<(), Error<CommE, PinE>> {
        let reg1 = self.ctrl_reg1_a.with_low(BF::LP_EN);
        self.iface
            .write_accel_register(Register::CTRL_REG1_A, reg1.bits)?;
        self.ctrl_reg1_a = reg1;
        Ok(())
    }
}

fn check_accel_odr_is_compatible_with_mode<CommE, PinE>(
    odr: Option<AccelOutputDataRate>,
    mode: AccelMode,
) -> Result<(), Error<CommE, PinE>> {
    if (odr == Some(AccelOutputDataRate::Khz1_620LowPower)
        || odr == Some(AccelOutputDataRate::Khz5_376LowPower))
        && (mode == AccelMode::Normal || mode == AccelMode::HighResolution)
        || (odr == Some(AccelOutputDataRate::Khz1_344) && mode == AccelMode::LowPower)
    {
        Err(Error::InvalidInputData)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod accel_odr_mode_tests {
    use super::check_accel_odr_is_compatible_with_mode;
    use super::AccelMode;
    use crate::AccelOutputDataRate as ODR;

    macro_rules! compatible {
        ($odr:ident, $power:ident) => {
            check_accel_odr_is_compatible_with_mode::<(), ()>(Some(ODR::$odr), AccelMode::$power)
                .unwrap();
        };
    }

    macro_rules! not_compatible {
        ($odr:ident, $power:ident) => {
            check_accel_odr_is_compatible_with_mode::<(), ()>(Some(ODR::$odr), AccelMode::$power)
                .expect_err("Should have returned error");
        };
    }

    macro_rules! none_odr_compatible {
        ($power:ident) => {
            check_accel_odr_is_compatible_with_mode::<(), ()>(None, AccelMode::$power).unwrap();
        };
    }

    #[test]
    fn all_modes_are_compatible_with_powerdown() {
        compatible!(Hz1, PowerDown);
        compatible!(Hz10, PowerDown);
        compatible!(Hz25, PowerDown);
        compatible!(Hz50, PowerDown);
        compatible!(Hz100, PowerDown);
        compatible!(Hz200, PowerDown);
        compatible!(Hz400, PowerDown);
        compatible!(Khz1_620LowPower, PowerDown);
        compatible!(Khz5_376LowPower, PowerDown);
        compatible!(Khz1_344, PowerDown);
    }

    #[test]
    fn normal_mode_compatibility() {
        compatible!(Hz1, Normal);
        compatible!(Hz10, Normal);
        compatible!(Hz25, Normal);
        compatible!(Hz50, Normal);
        compatible!(Hz100, Normal);
        compatible!(Hz200, Normal);
        compatible!(Hz400, Normal);
        not_compatible!(Khz1_620LowPower, Normal);
        not_compatible!(Khz5_376LowPower, Normal);
        compatible!(Khz1_344, Normal);
    }

    #[test]
    fn high_resolution_mode_compatibility() {
        compatible!(Hz1, HighResolution);
        compatible!(Hz10, HighResolution);
        compatible!(Hz25, HighResolution);
        compatible!(Hz50, HighResolution);
        compatible!(Hz100, HighResolution);
        compatible!(Hz200, HighResolution);
        compatible!(Hz400, HighResolution);
        not_compatible!(Khz1_620LowPower, HighResolution);
        not_compatible!(Khz5_376LowPower, HighResolution);
        compatible!(Khz1_344, HighResolution);
    }

    #[test]
    fn low_power_mode_compatibility() {
        compatible!(Hz1, LowPower);
        compatible!(Hz10, LowPower);
        compatible!(Hz25, LowPower);
        compatible!(Hz50, LowPower);
        compatible!(Hz100, LowPower);
        compatible!(Hz200, LowPower);
        compatible!(Hz400, LowPower);
        compatible!(Khz1_620LowPower, LowPower);
        compatible!(Khz5_376LowPower, LowPower);
        not_compatible!(Khz1_344, LowPower);
    }

    #[test]
    fn none_odr_compatibility() {
        none_odr_compatible!(LowPower);
        none_odr_compatible!(Normal);
        none_odr_compatible!(HighResolution);
        none_odr_compatible!(PowerDown);
    }
}
