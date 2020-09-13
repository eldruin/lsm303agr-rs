mod common;
use crate::common::{
    destroy_i2c, new_i2c, BitFlags as BF, Register, ACCEL_ADDR, DEFAULT_CTRL_REG1_A,
};
use embedded_hal_mock::i2c::Transaction as I2cTrans;
use lsm303agr::{AccelMode, UnscaledMeasurement};

#[test]
fn can_get_10_bit_data() {
    let mut sensor = new_i2c(&[I2cTrans::write_read(
        ACCEL_ADDR,
        vec![Register::OUT_X_L_A | 0x80],
        vec![0x10, 0x20, 0x30, 0x40, 0x50, 0x60],
    )]);
    let data = sensor.accel_data().unwrap();
    assert_eq!(
        data,
        UnscaledMeasurement {
            x: 0x2010 >> 6,
            y: 0x4030 >> 6,
            z: 0x6050 >> 6
        }
    );
    destroy_i2c(sensor);
}

#[test]
fn can_get_12_bit_data() {
    let mut sensor = new_i2c(&[
        I2cTrans::write(ACCEL_ADDR, vec![Register::CTRL_REG1_A, DEFAULT_CTRL_REG1_A]),
        I2cTrans::write(ACCEL_ADDR, vec![Register::CTRL_REG4_A, BF::HR]),
        I2cTrans::write_read(
            ACCEL_ADDR,
            vec![Register::OUT_X_L_A | 0x80],
            vec![0x10, 0x20, 0x30, 0x40, 0x50, 0x60],
        ),
    ]);
    sensor.set_accel_mode(AccelMode::HighResolution).unwrap();
    let data = sensor.accel_data().unwrap();
    assert_eq!(
        data,
        UnscaledMeasurement {
            x: 0x2010 >> 4,
            y: 0x4030 >> 4,
            z: 0x6050 >> 4
        }
    );
    destroy_i2c(sensor);
}

#[test]
fn can_get_8_bit_data() {
    let mut sensor = new_i2c(&[
        I2cTrans::write(ACCEL_ADDR, vec![Register::CTRL_REG4_A, 0]),
        I2cTrans::write(
            ACCEL_ADDR,
            vec![Register::CTRL_REG1_A, BF::LP_EN | DEFAULT_CTRL_REG1_A],
        ),
        I2cTrans::write_read(
            ACCEL_ADDR,
            vec![Register::OUT_X_L_A | 0x80],
            vec![0x10, 0x20, 0x30, 0x40, 0x50, 0x60],
        ),
    ]);
    sensor.set_accel_mode(AccelMode::LowPower).unwrap();
    let data = sensor.accel_data().unwrap();
    assert_eq!(
        data,
        UnscaledMeasurement {
            x: 0x20,
            y: 0x40,
            z: 0x60
        }
    );
    destroy_i2c(sensor);
}
