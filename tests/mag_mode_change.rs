mod common;
use crate::common::{destroy_i2c, new_i2c, Register, MAG_ADDR};
use embedded_hal_mock::i2c::Transaction as I2cTrans;

#[test]
fn can_change_into_continuous() {
    let sensor = new_i2c(&[I2cTrans::write(MAG_ADDR, vec![Register::CFG_REG_A_M, 0])]);
    let sensor = sensor.into_mag_continuous().ok().unwrap();
    destroy_i2c(sensor);
}

#[test]
fn can_change_into_one_shot() {
    let sensor = new_i2c(&[
        I2cTrans::write(MAG_ADDR, vec![Register::CFG_REG_A_M, 0x0]),
        I2cTrans::write(MAG_ADDR, vec![Register::CFG_REG_A_M, 0x3]),
    ]);
    let sensor = sensor.into_mag_continuous().ok().unwrap();
    let sensor = sensor.into_mag_one_shot().ok().unwrap();
    destroy_i2c(sensor);
}
