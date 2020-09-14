mod common;
use crate::common::{destroy_i2c, new_i2c, Register, DEFAULT_CFG_REG_A_M, MAG_ADDR};
use embedded_hal_mock::i2c::Transaction as I2cTrans;
use lsm303agr::MagOutputDataRate as ODR;

macro_rules! set_mag_odr {
    ($name:ident, $hz:ident, $value:expr) => {
        #[test]
        fn $name() {
            let mut sensor = new_i2c(&[I2cTrans::write(
                MAG_ADDR,
                vec![Register::CFG_REG_A_M, $value | DEFAULT_CFG_REG_A_M],
            )]);
            sensor.set_mag_odr(ODR::$hz).unwrap();
            destroy_i2c(sensor);
        }
    };
}
set_mag_odr!(set_mag_odr_hz10, Hz10, 0);
set_mag_odr!(set_mag_odr_hz20, Hz20, 1 << 2);
set_mag_odr!(set_mag_odr_hz50, Hz50, 2 << 2);
set_mag_odr!(set_mag_odr_hz100, Hz100, 3 << 2);
