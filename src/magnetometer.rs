use crate::{
    interface::{ReadData, WriteData},
    mode, Error, Lsm303agr, MagOutputDataRate, Measurement, Register,
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
    /// Magnetometer data
    ///
    /// Returned in mg (milli-gauss).
    pub fn mag_data(&mut self) -> Result<Measurement, Error<CommE, PinE>> {
        let data = self
            .iface
            .read_mag_3_double_registers(Register::OUTX_L_REG_M)?;

        Ok(Measurement {
            x: scale_measurement(data.0 as i16),
            y: scale_measurement(data.1 as i16),
            z: scale_measurement(data.2 as i16),
        })
    }
}

impl<DI, CommE, PinE> Lsm303agr<DI, mode::MagOneShot>
where
    DI: ReadData<Error = Error<CommE, PinE>> + WriteData<Error = Error<CommE, PinE>>,
{
    /// Magnetometer data
    pub fn mag_data(&mut self) -> nb::Result<Measurement, Error<CommE, PinE>> {
        let status = self.mag_status()?;
        if status.xyz_new_data {
            let data = self
                .iface
                .read_mag_3_double_registers(Register::OUTX_L_REG_M)?;
            Ok(Measurement {
                x: scale_measurement(data.0 as i16),
                y: scale_measurement(data.1 as i16),
                z: scale_measurement(data.2 as i16),
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
}

const SCALING_FACTOR: i32 = 150;

fn scale_measurement(unscaled: i16) -> i32 {
    unscaled as i32 * SCALING_FACTOR
}
