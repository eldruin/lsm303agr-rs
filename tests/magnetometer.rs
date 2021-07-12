mod common;
use crate::common::{
    destroy_i2c, destroy_spi, new_i2c, new_spi_mag, BitFlags as BF, Register, DEFAULT_CFG_REG_A_M,
    MAG_ADDR,
};
use embedded_hal_mock::{
    i2c::Transaction as I2cTrans,
    pin::{Mock as PinMock, State as PinState, Transaction as PinTrans},
    spi::Transaction as SpiTrans,
};
use lsm303agr::{MagOutputDataRate as ODR, Measurement, UnscaledMeasurement};

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
        Measurement {
            x: 0x2010 * 150,
            y: 0x4030 * 150,
            z: 0x6050 * 150,
        }
    );
    destroy_i2c(sensor);
}

#[test]
fn can_take_one_shot_unscaled_measurement() {
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
    let data = nb::block!(sensor.mag_data_unscaled()).unwrap();
    assert_eq!(
        data,
        UnscaledMeasurement {
            x: 0x2010,
            y: 0x4030,
            z: 0x6050,
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
        Measurement {
            x: 0x2010 * 150,
            y: 0x4030 * 150,
            z: 0x6050 * 150,
        }
    );
    destroy_i2c(sensor);
}

#[test]
fn can_take_continuous_unscaled_measurement() {
    let sensor = new_i2c(&[
        I2cTrans::write(MAG_ADDR, vec![Register::CFG_REG_A_M, 0]),
        I2cTrans::write_read(
            MAG_ADDR,
            vec![Register::OUTX_L_REG_M | 0x80],
            vec![0x10, 0x20, 0x30, 0x40, 0x50, 0x60],
        ),
    ]);
    let mut sensor = sensor.into_mag_continuous().ok().unwrap();
    let data = sensor.mag_data_unscaled().unwrap();
    assert_eq!(
        data,
        UnscaledMeasurement {
            x: 0x2010,
            y: 0x4030,
            z: 0x6050,
        }
    );
    destroy_i2c(sensor);
}

#[test]
fn can_take_continuous_measurement_spi() {
    let sensor = new_spi_mag(
        &[
            SpiTrans::write(vec![Register::CFG_REG_A_M, 0]),
            SpiTrans::transfer(
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
        ],
        PinMock::new(&[
            PinTrans::set(PinState::Low),
            PinTrans::set(PinState::High),
            PinTrans::set(PinState::Low),
            PinTrans::set(PinState::High),
        ]),
    );
    let mut sensor = sensor.into_mag_continuous().ok().unwrap();
    let data = sensor.mag_data().unwrap();
    assert_eq!(
        data,
        Measurement {
            x: 0x2010 * 150,
            y: 0x4030 * 150,
            z: 0x6050 * 150,
        }
    );
    destroy_spi(sensor);
}
