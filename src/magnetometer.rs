use crate::{
    interface::{ReadData, WriteData},
    mode, Error, Lsm303agr, MagOutputDataRate, Measurement, Register, UnscaledMeasurement,
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

    /// Internal function used by `enable_mag_offset_cancellation` and
    /// `disable_mag_offset_cancellation` to reduce code duplication
    fn offset_cancellation(
        &mut self,
        enable: bool,
        one_shot: bool,
    ) -> Result<(), Error<CommE, PinE>> {
        let cfg = self.cfg_reg_b_m.bits & 0b11101101;

        // In the LSM303AGR offset cancellation is enabled by setting bit OFF_CANC = 1 (and bit
        // OFF_CANC_ONE_SHOT = 1 in single measurement mode) in CFG_REG_B_M (61h)
        let off_canc = if enable { 1 << 1 } else { 0 };
        let off_canc_one_shot = if enable && one_shot { 1 << 4 } else { 0 };

        // Combine the two masks
        let mask = off_canc | off_canc_one_shot;

        self.iface
            .write_mag_register(Register::CFG_REG_B_M, cfg | mask)?;

        self.cfg_reg_b_m = (cfg | mask).into();

        Ok(())
    }
}

impl<DI, CommE, PinE> Lsm303agr<DI, mode::MagContinuous>
where
    DI: ReadData<Error = Error<CommE, PinE>> + WriteData<Error = Error<CommE, PinE>>,
{
    /// Magnetometer data
    ///
    /// Returned in nT (nanotesla)
    ///
    /// If you need the raw unscaled measurement see [`Lsm303agr::mag_data_unscaled`].
    pub fn mag_data(&mut self) -> Result<Measurement, Error<CommE, PinE>> {
        let unscaled = self.mag_data_unscaled()?;

        Ok(Measurement {
            x: scale_measurement(unscaled.x),
            y: scale_measurement(unscaled.y),
            z: scale_measurement(unscaled.z),
        })
    }

    /// Unscaled magnetometer data
    pub fn mag_data_unscaled(&mut self) -> Result<UnscaledMeasurement, Error<CommE, PinE>> {
        let data = self
            .iface
            .read_mag_3_double_registers(Register::OUTX_L_REG_M)?;

        Ok(UnscaledMeasurement {
            x: data.0 as i16,
            y: data.1 as i16,
            z: data.2 as i16,
        })
    }

    /// Enable the magnetometer's built in offset cancellation.
    ///
    /// Offset cancellation is **automatically** managed by the device in **continuous** mode.
    pub fn enable_mag_offset_cancellation(&mut self) -> Result<(), Error<CommE, PinE>> {
        self.offset_cancellation(true, false)
    }

    /// Disable the magnetometer's built in offset cancellation.
    pub fn disable_mag_offset_cancellation(&mut self) -> Result<(), Error<CommE, PinE>> {
        self.offset_cancellation(false, false)
    }
}

impl<DI, CommE, PinE> Lsm303agr<DI, mode::MagOneShot>
where
    DI: ReadData<Error = Error<CommE, PinE>> + WriteData<Error = Error<CommE, PinE>>,
{
    /// Magnetometer data in nT (nanoteslas)
    pub fn mag_data(&mut self) -> nb::Result<Measurement, Error<CommE, PinE>> {
        let unscaled = self.mag_data_unscaled()?;
        Ok(Measurement {
            x: scale_measurement(unscaled.x),
            y: scale_measurement(unscaled.y),
            z: scale_measurement(unscaled.z),
        })
    }

    /// Unscaled magnetometer data
    pub fn mag_data_unscaled(&mut self) -> nb::Result<UnscaledMeasurement, Error<CommE, PinE>> {
        let status = self.mag_status()?;
        if status.xyz_new_data {
            let data = self
                .iface
                .read_mag_3_double_registers(Register::OUTX_L_REG_M)?;
            Ok(UnscaledMeasurement {
                x: data.0 as i16,
                y: data.1 as i16,
                z: data.2 as i16,
            })
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

    /// Enable the magnetometer's built in offset cancellation
    ///
    /// Offset cancellation has to be **managed by the user** in **single measurement** (OneShot) mode averaging
    /// two consecutive measurements H<sub>n</sub> and H<sub>n-1</sub>.
    pub fn enable_mag_offset_cancellation(&mut self) -> Result<(), Error<CommE, PinE>> {
        self.offset_cancellation(true, true)
    }

    /// Disable the magnetometer's built in offset cancellation
    pub fn disable_mag_offset_cancellation(&mut self) -> Result<(), Error<CommE, PinE>> {
        self.offset_cancellation(false, true)
    }
}

const SCALING_FACTOR: i32 = 150;

fn scale_measurement(unscaled: i16) -> i32 {
    unscaled as i32 * SCALING_FACTOR
}
