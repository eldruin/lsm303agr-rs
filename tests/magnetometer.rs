#[macro_use]
mod common;
use crate::common::{
    destroy_i2c, destroy_spi, new_i2c, new_spi_mag, BitFlags as BF, Register, DEFAULT_CFG_REG_A_M,
    MAG_ADDR,
};
use embedded_hal_mock::eh1::{
    delay::NoopDelay as Delay, i2c::Transaction as I2cTrans, spi::Transaction as SpiTrans,
};
use lsm303agr::{MagMode, MagOutputDataRate as ODR};

macro_rules! set_mag_odr {
    ($name:ident, $hz:ident, $value:expr) => {
        #[test]
        fn $name() {
            let mut sensor = new_i2c(&[I2cTrans::write(
                MAG_ADDR,
                vec![Register::CFG_REG_A_M, $value | DEFAULT_CFG_REG_A_M],
            )]);
            sensor
                .set_mag_mode_and_odr(&mut Delay, MagMode::HighResolution, ODR::$hz)
                .unwrap();
            destroy_i2c(sensor);
        }
    };
}
set_mag_odr!(set_mag_odr_hz10, Hz10, 0);
set_mag_odr!(set_mag_odr_hz20, Hz20, 1 << 2);
set_mag_odr!(set_mag_odr_hz50, Hz50, 2 << 2);
set_mag_odr!(set_mag_odr_hz100, Hz100, 3 << 2);

#[test]
fn can_change_mode() {
    let mut sensor = new_i2c(&[
        // Set low-power mode
        I2cTrans::write(
            MAG_ADDR,
            vec![Register::CFG_REG_A_M, DEFAULT_CFG_REG_A_M | 0b00011100],
        ),
        // Set high-resolution mode
        I2cTrans::write(MAG_ADDR, vec![Register::CFG_REG_A_M, DEFAULT_CFG_REG_A_M]),
    ]);
    assert_eq!(sensor.get_mag_mode(), MagMode::HighResolution);

    sensor
        .set_mag_mode_and_odr(&mut Delay, MagMode::LowPower, ODR::Hz100)
        .unwrap();
    assert_eq!(sensor.get_mag_mode(), MagMode::LowPower);

    sensor
        .set_mag_mode_and_odr(&mut Delay, MagMode::HighResolution, ODR::Hz10)
        .unwrap();
    assert_eq!(sensor.get_mag_mode(), MagMode::HighResolution);

    destroy_i2c(sensor);
}

macro_rules! assert_eq_xyz_nt {
    ($data:expr) => {{
        crate::assert_eq_xyz!($data, x_nt, y_nt, z_nt, xyz_nt);
    }};
}

#[test]
fn can_take_one_shot_measurement_i2c() {
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
    let data = nb::block!(sensor.magnetic_field()).unwrap();

    assert_eq_xyz_nt!(data);

    assert_eq!(data.x_raw(), 0x2010);
    assert_eq!(data.y_raw(), 0x4030);
    assert_eq!(data.z_raw(), 0x6050);

    assert_eq!(data.x_unscaled(), 0x2010);
    assert_eq!(data.y_unscaled(), 0x4030);
    assert_eq!(data.z_unscaled(), 0x6050);

    assert_eq!(data.x_nt(), 0x2010 * 150);
    assert_eq!(data.y_nt(), 0x4030 * 150);
    assert_eq!(data.z_nt(), 0x6050 * 150);

    destroy_i2c(sensor);
}

#[test]
fn can_take_continuous_measurement_i2c() {
    let sensor = new_i2c(&[
        I2cTrans::write(MAG_ADDR, vec![Register::CFG_REG_A_M, 0]),
        I2cTrans::write_read(
            MAG_ADDR,
            vec![Register::OUTX_L_REG_M | 0x80],
            vec![0x10, 0x20, 0x30, 0x40, 0x50, 0x60],
        ),
    ]);
    let mut sensor = sensor.into_mag_continuous().ok().unwrap();
    let data = sensor.magnetic_field().unwrap();

    assert_eq_xyz_nt!(data);

    assert_eq!(data.x_raw(), 0x2010);
    assert_eq!(data.y_raw(), 0x4030);
    assert_eq!(data.z_raw(), 0x6050);

    assert_eq!(data.x_unscaled(), 0x2010);
    assert_eq!(data.y_unscaled(), 0x4030);
    assert_eq!(data.z_unscaled(), 0x6050);

    assert_eq!(data.x_nt(), 0x2010 * 150);
    assert_eq!(data.y_nt(), 0x4030 * 150);
    assert_eq!(data.z_nt(), 0x6050 * 150);

    destroy_i2c(sensor);
}

#[test]
fn can_take_continuous_measurement_spi() {
    let sensor = new_spi_mag(&[
        SpiTrans::transaction_start(),
        SpiTrans::write_vec(vec![Register::CFG_REG_A_M, 0]),
        SpiTrans::transaction_end(),
        SpiTrans::transaction_start(),
        SpiTrans::transfer_in_place(
            vec![
                Register::OUTX_L_REG_M | BF::SPI_MS | BF::SPI_RW,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
            vec![0, 0x10, 0x20, 0x30, 0x40, 0x50, 0x60],
        ),
        SpiTrans::transaction_end(),
    ]);
    let mut sensor = sensor.into_mag_continuous().ok().unwrap();
    let data = sensor.magnetic_field().unwrap();

    assert_eq_xyz_nt!(data);

    assert_eq!(data.x_raw(), 0x2010);
    assert_eq!(data.y_raw(), 0x4030);
    assert_eq!(data.z_raw(), 0x6050);

    assert_eq!(data.x_unscaled(), 0x2010);
    assert_eq!(data.y_unscaled(), 0x4030);
    assert_eq!(data.z_unscaled(), 0x6050);

    assert_eq!(data.x_nt(), 0x2010 * 150);
    assert_eq!(data.y_nt(), 0x4030 * 150);
    assert_eq!(data.z_nt(), 0x6050 * 150);

    destroy_spi(sensor);
}

#[test]
fn can_enable_mag_offset_cancellation_continuous() {
    let sensor = new_i2c(&[
        // Mag continuous mode
        I2cTrans::write(MAG_ADDR, vec![Register::CFG_REG_A_M, 0]),
        // Enable offset cancellation
        I2cTrans::write(MAG_ADDR, vec![Register::CFG_REG_B_M, BF::MAG_OFF_CANC]),
    ]);

    let mut sensor = sensor
        .into_mag_continuous()
        .expect("failed to set magnetometer into continuous mode");

    sensor
        .enable_mag_offset_cancellation()
        .expect("failed to enable offset cancellation");

    destroy_i2c(sensor);
}

#[test]
fn can_disable_mag_offset_cancellation_continuous() {
    let sensor = new_i2c(&[
        // Mag continuous mode
        I2cTrans::write(MAG_ADDR, vec![Register::CFG_REG_A_M, 0]),
        // Disable offset cancellation
        I2cTrans::write(MAG_ADDR, vec![Register::CFG_REG_B_M, 0]),
    ]);

    let mut sensor = sensor
        .into_mag_continuous()
        .expect("failed to set magnetometer into continuous mode");

    sensor
        .disable_mag_offset_cancellation()
        .expect("failed to disable offset cancellation");

    destroy_i2c(sensor);
}

#[test]
fn can_enable_mag_offset_cancellation_one_shot() {
    let mut sensor = new_i2c(&[
        // Enable offset cancellation
        I2cTrans::write(
            MAG_ADDR,
            vec![
                Register::CFG_REG_B_M,
                BF::MAG_OFF_CANC | BF::MAG_OFF_CANC_ONE_SHOT,
            ],
        ),
    ]);

    sensor
        .enable_mag_offset_cancellation()
        .expect("failed to disable offset cancellation");

    destroy_i2c(sensor);
}

#[test]
fn can_disable_mag_offset_cancellation_one_shot() {
    let mut sensor = new_i2c(&[
        // Disable offset cancellation
        I2cTrans::write(MAG_ADDR, vec![Register::CFG_REG_B_M, 0]),
    ]);

    sensor
        .disable_mag_offset_cancellation()
        .expect("failed to disable offset cancellation");

    destroy_i2c(sensor);
}

#[test]
fn can_enable_mag_low_pass_filter() {
    let mut sensor = new_i2c(&[
        // Enable low-pass filter
        I2cTrans::write(MAG_ADDR, vec![Register::CFG_REG_B_M, 0b1]),
    ]);

    sensor
        .mag_enable_low_pass_filter()
        .expect("failed to enable low-pass filter");

    destroy_i2c(sensor);
}

#[test]
fn can_disable_mag_low_pass_filter() {
    let mut sensor = new_i2c(&[
        // Disable low-pass filter
        I2cTrans::write(MAG_ADDR, vec![Register::CFG_REG_B_M, 0b0]),
    ]);

    sensor
        .mag_disable_low_pass_filter()
        .expect("failed to disable low-pass filter");

    destroy_i2c(sensor);
}
