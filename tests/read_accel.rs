mod common;
use crate::common::{
    default_cs, destroy_i2c, destroy_spi, new_i2c, new_spi_accel, BitFlags as BF, Register,
    ACCEL_ADDR, DEFAULT_CTRL_REG1_A,
};
use embedded_hal_mock::{i2c::Transaction as I2cTrans, spi::Transaction as SpiTrans};
use lsm303agr::{AccelMode, Measurement};

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
        Measurement {
            x: 512, // ~= 0x2010 / (1 << 4)
            y: 1024, // ~= 0x4030 / (1 << 4)
            z: 1540, // ~= 0x6050 / (1 << 4)
        }
    );
    destroy_i2c(sensor);
}

#[test]
fn can_get_10_bit_data_spi() {
    let mut sensor = new_spi_accel(
        &[SpiTrans::transfer(
            vec![
                Register::OUT_X_L_A | BF::SPI_RW | BF::SPI_MS,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
            vec![0, 0x10, 0x20, 0x30, 0x40, 0x50, 0x60],
        )],
        default_cs(),
    );
    let data = sensor.accel_data().unwrap();
    assert_eq!(
        data,
        Measurement {
            x: 512, // ~= 0x2010 / (1 << 4),
            y: 1024, // ~= 0x4030 / (1 << 4),
            z: 1540, // ~= 0x6050 / (1 << 4),
        }
    );
    destroy_spi(sensor);
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
        Measurement {
            x: 513, // == 0x2010 / (1 << 4),
            y: 1027, // == 0x4030 / (1 << 4),
            z: 1541, // == 0x6050 / (1 << 4),
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
        Measurement {
            x: 512, // ~= 0x2010 / (1 << 4),
            y: 1024, // ~= 0x4030 / (1 << 4),
            z: 1536, // ~= 0x6050 / (1 << 4),
        }
    );
    destroy_i2c(sensor);
}
