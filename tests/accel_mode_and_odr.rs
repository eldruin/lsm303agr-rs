mod common;
use crate::common::{
    destroy_i2c, new_i2c, BitFlags as BF, Register, ACCEL_ADDR, DEFAULT_CTRL_REG1_A,
};
use embedded_hal_mock::i2c::Transaction as I2cTrans;
use lsm303agr::{AccelMode as Mode, AccelOutputDataRate as ODR};

macro_rules! normal_pwr {
    ($name:ident, $hz:ident, $value:expr) => {
        #[test]
        fn $name() {
            let mut sensor = new_i2c(&[I2cTrans::write(
                ACCEL_ADDR,
                vec![Register::CTRL_REG1_A, $value | DEFAULT_CTRL_REG1_A],
            )]);
            sensor.set_accel_odr(ODR::$hz).unwrap();
            destroy_i2c(sensor);
        }
    };
}
normal_pwr!(normal_hz1, Hz1, 1 << 4);
normal_pwr!(normal_hz10, Hz10, 2 << 4);
normal_pwr!(normal_hz25, Hz25, 3 << 4);
normal_pwr!(normal_hz50, Hz50, 4 << 4);
normal_pwr!(normal_hz100, Hz100, 5 << 4);
normal_pwr!(normal_hz200, Hz200, 6 << 4);
normal_pwr!(normal_hz400, Hz400, 7 << 4);
normal_pwr!(normal_khz1_344, Khz1_344, 9 << 4);

#[test]
fn normal_pwr_enable_lp_khz_1_620() {
    let mut sensor = new_i2c(&[I2cTrans::write(
        ACCEL_ADDR,
        vec![
            Register::CTRL_REG1_A,
            BF::LP_EN | 8 << 4 | DEFAULT_CTRL_REG1_A,
        ],
    )]);
    sensor.set_accel_odr(ODR::Khz1_620LowPower).unwrap();
    destroy_i2c(sensor);
}

#[test]
fn normal_pwr_enable_lp_khz_5_376() {
    let mut sensor = new_i2c(&[I2cTrans::write(
        ACCEL_ADDR,
        vec![
            Register::CTRL_REG1_A,
            BF::LP_EN | 9 << 4 | DEFAULT_CTRL_REG1_A,
        ],
    )]);
    sensor.set_accel_odr(ODR::Khz5_376LowPower).unwrap();
    destroy_i2c(sensor);
}

#[test]
fn from_high_resolution_to_low_power_only_odr() {
    let mut sensor = new_i2c(&[
        I2cTrans::write(ACCEL_ADDR, vec![Register::CTRL_REG1_A, DEFAULT_CTRL_REG1_A]),
        I2cTrans::write(ACCEL_ADDR, vec![Register::CTRL_REG4_A, BF::HR]),
        I2cTrans::write(ACCEL_ADDR, vec![Register::CTRL_REG4_A, 0]),
        I2cTrans::write(
            ACCEL_ADDR,
            vec![
                Register::CTRL_REG1_A,
                BF::LP_EN | 9 << 4 | DEFAULT_CTRL_REG1_A,
            ],
        ),
    ]);
    sensor.set_accel_mode(Mode::HighResolution).unwrap();
    sensor.set_accel_odr(ODR::Khz5_376LowPower).unwrap();
    destroy_i2c(sensor);
}

#[test]
fn from_normal_to_low_power_only_odr() {
    let mut sensor = new_i2c(&[
        I2cTrans::write(ACCEL_ADDR, vec![Register::CTRL_REG1_A, DEFAULT_CTRL_REG1_A]),
        I2cTrans::write(ACCEL_ADDR, vec![Register::CTRL_REG4_A, 0]),
        I2cTrans::write(
            ACCEL_ADDR,
            vec![
                Register::CTRL_REG1_A,
                BF::LP_EN | 9 << 4 | DEFAULT_CTRL_REG1_A,
            ],
        ),
    ]);
    sensor.set_accel_mode(Mode::Normal).unwrap();
    sensor.set_accel_odr(ODR::Khz5_376LowPower).unwrap();
    destroy_i2c(sensor);
}

#[test]
fn incompatible_accel_mode() {
    let mut sensor = new_i2c(&[I2cTrans::write(
        ACCEL_ADDR,
        vec![Register::CTRL_REG1_A, 9 << 4 | DEFAULT_CTRL_REG1_A],
    )]);
    sensor.set_accel_odr(ODR::Khz1_344).unwrap();
    sensor
        .set_accel_mode(Mode::LowPower)
        .expect_err("should have returned error");
    destroy_i2c(sensor);
}

#[test]
fn can_power_down() {
    let mut sensor = new_i2c(&[I2cTrans::write(
        ACCEL_ADDR,
        vec![Register::CTRL_REG1_A, DEFAULT_CTRL_REG1_A],
    )]);
    sensor.set_accel_mode(Mode::PowerDown).unwrap();
    destroy_i2c(sensor);
}

#[test]
fn can_set_mode_normal() {
    let mut sensor = new_i2c(&[
        I2cTrans::write(ACCEL_ADDR, vec![Register::CTRL_REG1_A, DEFAULT_CTRL_REG1_A]),
        I2cTrans::write(ACCEL_ADDR, vec![Register::CTRL_REG4_A, 0]),
    ]);
    sensor.set_accel_mode(Mode::Normal).unwrap();
    destroy_i2c(sensor);
}

#[test]
fn can_set_mode_high_resolution() {
    let mut sensor = new_i2c(&[
        I2cTrans::write(ACCEL_ADDR, vec![Register::CTRL_REG1_A, DEFAULT_CTRL_REG1_A]),
        I2cTrans::write(ACCEL_ADDR, vec![Register::CTRL_REG4_A, BF::HR]),
    ]);
    sensor.set_accel_mode(Mode::HighResolution).unwrap();
    destroy_i2c(sensor);
}

#[test]
fn can_set_mode_low_power() {
    let mut sensor = new_i2c(&[
        I2cTrans::write(ACCEL_ADDR, vec![Register::CTRL_REG4_A, 0]),
        I2cTrans::write(
            ACCEL_ADDR,
            vec![Register::CTRL_REG1_A, BF::LP_EN | DEFAULT_CTRL_REG1_A],
        ),
    ]);
    sensor.set_accel_mode(Mode::LowPower).unwrap();
    destroy_i2c(sensor);
}

#[test]
fn can_power_down_after_odr3() {
    let mut sensor = new_i2c(&[
        I2cTrans::write(
            ACCEL_ADDR,
            vec![
                Register::CTRL_REG1_A,
                DEFAULT_CTRL_REG1_A | BF::LP_EN | 8 << 4,
            ],
        ),
        I2cTrans::write(
            ACCEL_ADDR,
            vec![Register::CTRL_REG1_A, DEFAULT_CTRL_REG1_A | BF::LP_EN],
        ),
    ]);
    sensor.set_accel_odr(ODR::Khz1_620LowPower).unwrap();
    sensor.set_accel_mode(Mode::PowerDown).unwrap();
    destroy_i2c(sensor);
}
