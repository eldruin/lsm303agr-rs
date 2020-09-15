mod common;
use crate::common::{destroy_i2c, new_i2c, Register, DEFAULT_CFG_REG_A_M, MAG_ADDR};
use embedded_hal_mock::i2c::Transaction as I2cTrans;
use lsm303agr::{MagOutputDataRate as ODR, UnscaledMeasurement};
use nb;

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

#[test]
fn can_take_one_shot_measurement() {
    let mut sensor = new_i2c(&[
        I2cTrans::write_read(MAG_ADDR, vec![Register::STATUS_REG_M], vec![0]),
        I2cTrans::write_read(MAG_ADDR, vec![Register::CFG_REG_A_M], vec![0]), // idle
        I2cTrans::write(MAG_ADDR, vec![Register::CFG_REG_A_M, 1]),            // start measurement
        I2cTrans::write_read(MAG_ADDR, vec![Register::STATUS_REG_M], vec![0]),
        I2cTrans::write_read(MAG_ADDR, vec![Register::CFG_REG_A_M], vec![1]), // continue waiting
        I2cTrans::write_read(MAG_ADDR, vec![Register::STATUS_REG_M], vec![0xFF]),
        I2cTrans::write_read(
            MAG_ADDR,
            vec![Register::OUTX_L_REG_M | 0x80],
            vec![0x10, 0x20, 0x30, 0x40, 0x50, 0x60],
        ),
    ]);
    let data = nb::block!(sensor.mag_data()).unwrap();
    assert_eq!(
        data,
        UnscaledMeasurement {
            x: 0x2010,
            y: 0x4030,
            z: 0x6050
        }
    );
    destroy_i2c(sensor);
}

#[test]
fn can_take_continuous_measurement() {
    let sensor = new_i2c(&[
        I2cTrans::write(MAG_ADDR, vec![Register::CFG_REG_A_M, 0]),
        I2cTrans::write_read(
            MAG_ADDR,
            vec![Register::OUTX_L_REG_M | 0x80],
            vec![0x10, 0x20, 0x30, 0x40, 0x50, 0x60],
        ),
    ]);
    let mut sensor = sensor.into_mag_continuous().ok().unwrap();
    let data = sensor.mag_data().unwrap();
    assert_eq!(
        data,
        UnscaledMeasurement {
            x: 0x2010,
            y: 0x4030,
            z: 0x6050
        }
    );
    destroy_i2c(sensor);
}
