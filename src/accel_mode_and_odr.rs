use embedded_hal::delay::DelayNs;
use maybe_async_cfg::maybe;

use crate::{
    interface::{ReadData, WriteData},
    register_address::{CtrlReg1A, CtrlReg4A},
    AccelMode, AccelOutputDataRate, AccelScale, Error, Lsm303agr,
};

#[maybe(
    sync(
        cfg(not(feature = "async")),
        keep_self,
    ),
    async (
        cfg(feature = "async"),
        keep_self,
    )
)]
impl<DI, CommE, MODE> Lsm303agr<DI, MODE>
where
    DI: ReadData<Error = Error<CommE>> + WriteData<Error = Error<CommE>>,
{
    /// Set accelerometer power/resolution mode and output data rate.
    ///
    /// Returns `Error::InvalidInputData` if the mode is incompatible with
    /// the given output data rate.
    ///
    // #[doc = include_str!("delay.md")]
    pub async fn set_accel_mode_and_odr<D: DelayNs>(
        &mut self,
        delay: &mut D,
        mode: AccelMode,
        odr: impl Into<Option<AccelOutputDataRate>>,
    ) -> Result<(), Error<CommE>> {
        let odr = odr.into();

        check_accel_odr_is_compatible_with_mode(odr, mode)?;

        let old_mode = self.get_accel_mode().await;

        let mut reg1 = self.ctrl_reg1_a.difference(CtrlReg1A::ODR);

        if let Some(odr) = odr {
            reg1 = reg1.with_odr(odr);
        }

        let reg1 = if mode == AccelMode::LowPower {
            reg1.union(CtrlReg1A::LPEN)
        } else {
            reg1.difference(CtrlReg1A::LPEN)
        };

        let reg4 = self.ctrl_reg4_a.difference(CtrlReg4A::HR);

        if mode != AccelMode::HighResolution {
            self.iface.write_accel_register(reg4).await?;
            self.ctrl_reg4_a = reg4;
        }

        self.iface.write_accel_register(reg1).await?;
        self.ctrl_reg1_a = reg1;
        self.accel_odr = odr;

        if mode == AccelMode::HighResolution {
            let reg4 = reg4.union(CtrlReg4A::HR);
            self.iface.write_accel_register(reg4).await?;
            self.ctrl_reg4_a = reg4;
        }

        if let Some(odr) = self.accel_odr {
            let change_time = old_mode.change_time_us(mode, odr);
            delay.delay_us(change_time);
        }

        Ok(())
    }

    /// Get the accelerometer mode
    pub async fn get_accel_mode(&mut self) -> AccelMode {
        let power_down = self.ctrl_reg1_a.intersection(CtrlReg1A::ODR).is_empty();
        let lp_enabled = self.ctrl_reg1_a.contains(CtrlReg1A::LPEN);
        let hr_enabled = self.ctrl_reg4_a.contains(CtrlReg4A::HR);

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
    pub async fn set_accel_scale(&mut self, scale: AccelScale) -> Result<(), Error<CommE>> {
        let reg4 = self.ctrl_reg4_a.with_scale(scale);
        self.iface.write_accel_register(reg4).await?;
        self.ctrl_reg4_a = reg4;
        Ok(())
    }

    /// Get accelerometer scaling factor
    pub async fn get_accel_scale(&self) -> AccelScale {
        self.ctrl_reg4_a.scale()
    }
}

fn check_accel_odr_is_compatible_with_mode<CommE>(
    odr: Option<AccelOutputDataRate>,
    mode: AccelMode,
) -> Result<(), Error<CommE>> {
    match (odr, mode) {
        (None, AccelMode::PowerDown) => Ok(()),
        (None, _) => Err(Error::InvalidInputData),
        (Some(odr), mode) => match (odr, mode) {
            (AccelOutputDataRate::Khz1_344, AccelMode::LowPower)
            | (
                AccelOutputDataRate::Khz1_620LowPower | AccelOutputDataRate::Khz5_376LowPower,
                AccelMode::Normal | AccelMode::HighResolution,
            ) => Err(Error::InvalidInputData),
            _ => Ok(()),
        },
    }
}

#[cfg(test)]
mod accel_odr_mode_tests {
    use super::check_accel_odr_is_compatible_with_mode;
    use super::AccelMode;
    use crate::AccelOutputDataRate as ODR;

    macro_rules! compatible {
        ($odr:ident, $power:ident) => {
            check_accel_odr_is_compatible_with_mode::<()>(Some(ODR::$odr), AccelMode::$power)
                .unwrap();
        };
    }

    macro_rules! not_compatible {
        ($odr:ident, $power:ident) => {
            check_accel_odr_is_compatible_with_mode::<()>(Some(ODR::$odr), AccelMode::$power)
                .expect_err("Should have returned error");
        };
    }

    macro_rules! none_odr_compatible {
        ($power:ident) => {
            check_accel_odr_is_compatible_with_mode::<()>(None, AccelMode::$power).unwrap();
        };
    }

    macro_rules! not_none_odr_compatible {
        ($power:ident) => {
            check_accel_odr_is_compatible_with_mode::<()>(None, AccelMode::$power)
                .expect_err("Shout not be compatible");
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
        not_none_odr_compatible!(LowPower);
        not_none_odr_compatible!(Normal);
        not_none_odr_compatible!(HighResolution);
        none_odr_compatible!(PowerDown);
    }
}
