mod common;
use crate::common::{
    default_cs, default_cs_n, destroy_i2c, destroy_spi, new_i2c, new_spi, new_spi_accel,
    new_spi_mag, BitFlags as BF, Register, ACCEL_ADDR, MAG_ADDR,
};
use embedded_hal_mock::{
    i2c::Transaction as I2cTrans, pin::Mock as PinMock, spi::Transaction as SpiTrans,
};

#[test]
fn can_create_and_destroy_i2c() {
    let sensor = new_i2c(&[]);
    destroy_i2c(sensor);
}

#[test]
fn can_create_and_destroy_spi() {
    let sensor = new_spi_accel(&[], PinMock::new(&[]));
    destroy_spi(sensor);
}

#[test]
fn i2c_acc_id_is_not_correct() {
    let acc_id = 0xAB;
    let mut sensor = new_i2c(&[I2cTrans::write_read(
        ACCEL_ADDR,
        vec![Register::WHO_AM_I_A],
        vec![acc_id],
    )]);
    let id = sensor.accelerometer_id().unwrap();

    assert_eq!(id.raw(), acc_id);
    assert!(!id.is_whoami());

    destroy_i2c(sensor);
}

#[test]
fn i2c_acc_id_is_correct() {
    let acc_id = 0x33;
    let mut sensor = new_i2c(&[I2cTrans::write_read(
        ACCEL_ADDR,
        vec![Register::WHO_AM_I_A],
        vec![acc_id],
    )]);
    let id = sensor.accelerometer_id().unwrap();

    assert_eq!(id.raw(), acc_id);
    assert!(id.is_whoami());

    destroy_i2c(sensor);
}

#[test]
fn i2c_mag_id_is_not_correct() {
    let mag_id = 0xAB;
    let mut sensor = new_i2c(&[I2cTrans::write_read(
        MAG_ADDR,
        vec![Register::WHO_AM_I_M],
        vec![mag_id],
    )]);
    let id = sensor.magnetometer_id().unwrap();

    assert_eq!(id.raw(), mag_id);
    assert!(!id.is_correct());

    destroy_i2c(sensor);
}

#[test]
fn i2c_mag_id_is_whoami() {
    let mag_id = 0x40;
    let mut sensor = new_i2c(&[I2cTrans::write_read(
        MAG_ADDR,
        vec![Register::WHO_AM_I_M],
        vec![mag_id],
    )]);
    let id = sensor.magnetometer_id().unwrap();

    assert_eq!(id.raw(), mag_id);
    assert!(id.is_whoami());

    destroy_i2c(sensor);
}

#[test]
fn spi_acc_id_is_not_whoami() {
    let acc_id = 0xAB;
    let mut sensor = new_spi_accel(
        &[SpiTrans::transfer(
            vec![BF::SPI_RW | Register::WHO_AM_I_A, 0],
            vec![0, acc_id],
        )],
        default_cs(),
    );
    let id = sensor.accelerometer_id().unwrap();

    assert_eq!(id.raw(), acc_id);
    assert!(!id.is_whoami());

    destroy_spi(sensor);
}

#[test]
fn spi_acc_id_is_whoami() {
    let acc_id = 0x33;
    let mut sensor = new_spi_accel(
        &[SpiTrans::transfer(
            vec![BF::SPI_RW | Register::WHO_AM_I_A, 0],
            vec![0, acc_id],
        )],
        default_cs(),
    );
    let id = sensor.accelerometer_id().unwrap();

    assert_eq!(id.raw(), acc_id);
    assert!(id.is_correct());

    destroy_spi(sensor);
}

#[test]
fn spi_mag_id_is_not_correct() {
    let mag_id = 0xAB;
    let mut sensor = new_spi_mag(
        &[SpiTrans::transfer(
            vec![BF::SPI_RW | Register::WHO_AM_I_M, 0],
            vec![0, mag_id],
        )],
        default_cs(),
    );
    let id = sensor.magnetometer_id().unwrap();

    assert_eq!(id.raw(), mag_id);
    assert!(!id.is_correct());

    destroy_spi(sensor);
}

#[test]
fn spi_mag_id_is_correct() {
    let mag_id = 0x40;
    let mut sensor = new_spi_mag(
        &[SpiTrans::transfer(
            vec![BF::SPI_RW | Register::WHO_AM_I_M, 0],
            vec![0, mag_id],
        )],
        default_cs(),
    );
    let id = sensor.magnetometer_id().unwrap();

    assert_eq!(id.raw(), mag_id);
    assert!(id.is_whoami());

    destroy_spi(sensor);
}

#[test]
fn can_init_i2c() {
    let mut sensor = new_i2c(&[
        I2cTrans::write(ACCEL_ADDR, vec![Register::CTRL_REG4_A, BF::ACCEL_BDU]),
        I2cTrans::write(
            ACCEL_ADDR,
            vec![Register::TEMP_CFG_REG_A, BF::TEMP_EN1 | BF::TEMP_EN0],
        ),
        I2cTrans::write(MAG_ADDR, vec![Register::CFG_REG_C_M, BF::MAG_BDU]),
    ]);
    sensor.init().unwrap();
    destroy_i2c(sensor);
}

#[test]
fn can_init_spi() {
    let mut sensor = new_spi(
        &[
            SpiTrans::write(vec![Register::CTRL_REG4_A, BF::ACCEL_BDU]),
            SpiTrans::write(vec![Register::TEMP_CFG_REG_A, BF::TEMP_EN1 | BF::TEMP_EN0]),
            SpiTrans::write(vec![Register::CFG_REG_C_M, BF::MAG_BDU]),
        ],
        default_cs_n(2),
        default_cs(),
    );
    sensor.init().unwrap();
    destroy_spi(sensor);
}
