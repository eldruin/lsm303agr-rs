mod common;
use crate::common::{
    default_cs_n, destroy_i2c, destroy_spi, new_i2c, new_spi_accel, BitFlags as BF, Register,
    ACCEL_ADDR, DEFAULT_CTRL_REG1_A, HZ50,
};
use embedded_hal_mock::{i2c::Transaction as I2cTrans, spi::Transaction as SpiTrans};
use lsm303agr::AccelOutputDataRate;

#[test]
fn can_read_temp_has_new_data() {
    let mut sensor = new_i2c(&[I2cTrans::write_read(
        ACCEL_ADDR,
        vec![Register::STATUS_REG_AUX_A],
        vec![BF::TDA],
    )]);

    assert!(sensor.temp_has_new_data().unwrap());
    destroy_i2c(sensor);
}

#[test]
fn can_read_temp_has_no_new_data() {
    let mut sensor = new_i2c(&[I2cTrans::write_read(
        ACCEL_ADDR,
        vec![Register::STATUS_REG_AUX_A],
        vec![0x00],
    )]);

    assert!(!sensor.temp_has_new_data().unwrap());
    destroy_i2c(sensor);
}

#[test]
fn can_read_raw_temp_data() {
    let mut sensor = new_i2c(&[
        I2cTrans::write(
            ACCEL_ADDR,
            vec![Register::CTRL_REG1_A, DEFAULT_CTRL_REG1_A | HZ50],
        ),
        I2cTrans::write_read(
            ACCEL_ADDR,
            vec![Register::OUT_TEMP_L_A | 0x80],
            vec![0x10, 0x20],
        ),
    ]);

    sensor.set_accel_odr(AccelOutputDataRate::Hz50).unwrap();
    let data = sensor.temp_data().unwrap();
    assert_eq!(data, 0x2010);
    destroy_i2c(sensor);
}

#[test]
fn can_read_celsius_temp_data() {
    let mut sensor = new_i2c(&[
        I2cTrans::write(
            ACCEL_ADDR,
            vec![Register::CTRL_REG1_A, DEFAULT_CTRL_REG1_A | HZ50],
        ),
        I2cTrans::write_read(
            ACCEL_ADDR,
            vec![Register::OUT_TEMP_L_A | 0x80],
            vec![0x10, 0x20],
        ),
    ]);

    sensor.set_accel_odr(AccelOutputDataRate::Hz50).unwrap();
    let data = sensor.temp_celsius().unwrap();
    assert_eq!(
        data.round() as i32,
        ((0x2010 as f64 / 256.0) + 25.0).round() as i32
    );
    destroy_i2c(sensor);
}

#[test]
fn can_read_raw_temp_data_spi() {
    let mut sensor = new_spi_accel(
        &[
            SpiTrans::write(vec![Register::CTRL_REG1_A, DEFAULT_CTRL_REG1_A | HZ50]),
            SpiTrans::transfer(
                vec![Register::OUT_TEMP_L_A | BF::SPI_RW | BF::SPI_MS, 0, 0],
                vec![0, 0x10, 0x20],
            ),
        ],
        default_cs_n(2),
    );

    sensor.set_accel_odr(AccelOutputDataRate::Hz50).unwrap();
    let data = sensor.temp_data().unwrap();
    assert_eq!(data, 0x2010);
    destroy_spi(sensor);
}

#[test]
fn can_read_celsius_temp_data_spi() {
    let mut sensor = new_spi_accel(
        &[
            SpiTrans::write(vec![Register::CTRL_REG1_A, DEFAULT_CTRL_REG1_A | HZ50]),
            SpiTrans::transfer(
                vec![Register::OUT_TEMP_L_A | BF::SPI_RW | BF::SPI_MS, 0, 0],
                vec![0, 0x10, 0x20],
            ),
        ],
        default_cs_n(2),
    );

    sensor.set_accel_odr(AccelOutputDataRate::Hz50).unwrap();
    let data = sensor.temp_celsius().unwrap();
    assert_eq!(
        data.round() as i32,
        ((0x2010 as f64 / 256.0) + 25.0).round() as i32
    );
    destroy_spi(sensor);
}
