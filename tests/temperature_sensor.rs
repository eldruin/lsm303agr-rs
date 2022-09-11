mod common;
use crate::common::{
    default_cs_n, destroy_i2c, destroy_spi, new_i2c, new_spi_accel, BitFlags as BF, Register,
    ACCEL_ADDR, DEFAULT_CTRL_REG1_A, HZ50,
};
use embedded_hal_mock::{
    delay::MockNoop as Delay, i2c::Transaction as I2cTrans, spi::Transaction as SpiTrans,
};
use lsm303agr::{AccelMode, AccelOutputDataRate};

#[test]
fn can_read_temperature_has_new_data() {
    let mut sensor = new_i2c(&[I2cTrans::write_read(
        ACCEL_ADDR,
        vec![Register::STATUS_REG_AUX_A],
        vec![BF::TDA],
    )]);

    assert!(sensor.temperature_status().unwrap().new_data());
    destroy_i2c(sensor);
}

#[test]
fn can_read_temperature_has_data_overrun() {
    let mut sensor = new_i2c(&[I2cTrans::write_read(
        ACCEL_ADDR,
        vec![Register::STATUS_REG_AUX_A],
        vec![BF::TOR],
    )]);

    assert!(sensor.temperature_status().unwrap().overrun());
    destroy_i2c(sensor);
}

#[test]
fn can_read_temperature_has_no_new_data() {
    let mut sensor = new_i2c(&[I2cTrans::write_read(
        ACCEL_ADDR,
        vec![Register::STATUS_REG_AUX_A],
        vec![0x00],
    )]);

    assert!(!sensor.temperature_status().unwrap().new_data());
    destroy_i2c(sensor);
}

#[test]
fn can_read_temperature_i2c() {
    let mut sensor = new_i2c(&[
        I2cTrans::write(ACCEL_ADDR, vec![Register::CTRL_REG4_A, 0]),
        I2cTrans::write(
            ACCEL_ADDR,
            vec![Register::CTRL_REG1_A, DEFAULT_CTRL_REG1_A | HZ50],
        ),
        I2cTrans::write_read(
            ACCEL_ADDR,
            vec![Register::OUT_TEMP_L_A | 0x80],
            vec![0xb3, 0xe2],
        ),
    ]);

    sensor
        .set_accel_mode_and_odr(&mut Delay, AccelMode::Normal, AccelOutputDataRate::Hz50)
        .unwrap();
    let data = sensor.temperature().unwrap();

    assert_eq!(data.raw(), 0xe2b3);
    assert_eq!(data.unscaled(), -7501);
    assert_eq!((data.degrees_celsius() * 10.0).round() / 10.0, -4.3);

    destroy_i2c(sensor);
}

#[test]
fn can_read_temperature_spi() {
    let mut sensor = new_spi_accel(
        &[
            SpiTrans::write(vec![Register::CTRL_REG4_A, 0]),
            SpiTrans::write(vec![Register::CTRL_REG1_A, DEFAULT_CTRL_REG1_A | HZ50]),
            SpiTrans::transfer(
                vec![Register::OUT_TEMP_L_A | BF::SPI_RW | BF::SPI_MS, 0, 0],
                vec![0, 0x10, 0x20],
            ),
        ],
        default_cs_n(3),
    );

    sensor
        .set_accel_mode_and_odr(&mut Delay, AccelMode::Normal, AccelOutputDataRate::Hz50)
        .unwrap();
    let data = sensor.temperature().unwrap();

    assert_eq!(data.raw(), 0x2010);
    assert_eq!(data.unscaled(), 8208);
    assert_eq!((data.degrees_celsius() * 10.0).round() / 10.0, 57.1);

    destroy_spi(sensor);
}
