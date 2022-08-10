mod common;
use crate::common::{
    default_cs_n, destroy_i2c, destroy_spi, new_i2c, new_spi_accel, BitFlags as BF, Register,
    ACCEL_ADDR, DEFAULT_CTRL_REG1_A, HZ50,
};
use embedded_hal_mock::{
    delay::MockNoop as Delay, i2c::Transaction as I2cTrans, spi::Transaction as SpiTrans,
};
use lsm303agr::{AccelMode, AccelOutputDataRate, AccelScale};

fn i2c_mode_txns(mode: &AccelMode) -> Vec<I2cTrans> {
    match mode {
        AccelMode::Normal => vec![],
        AccelMode::LowPower => vec![
            I2cTrans::write(ACCEL_ADDR, vec![Register::CTRL_REG4_A, 0]),
            I2cTrans::write(
                ACCEL_ADDR,
                vec![
                    Register::CTRL_REG1_A,
                    DEFAULT_CTRL_REG1_A | BF::LP_EN | HZ50,
                ],
            ),
        ],
        AccelMode::HighResolution => vec![
            I2cTrans::write(
                ACCEL_ADDR,
                vec![Register::CTRL_REG1_A, DEFAULT_CTRL_REG1_A | HZ50],
            ),
            I2cTrans::write(ACCEL_ADDR, vec![Register::CTRL_REG4_A, BF::HR]),
        ],
        AccelMode::PowerDown => panic!("cannot read i2c in power down mode"),
    }
}

fn i2c_scale_txns(mode: &AccelMode, scale: &AccelScale) -> Vec<I2cTrans> {
    let base_reg = match mode {
        AccelMode::HighResolution => BF::HR,
        _ => 0,
    };
    match scale {
        AccelScale::G2 => vec![],
        AccelScale::G4 => vec![I2cTrans::write(
            ACCEL_ADDR,
            vec![Register::CTRL_REG4_A, base_reg | (0b01 << 4)],
        )],
        AccelScale::G8 => vec![I2cTrans::write(
            ACCEL_ADDR,
            vec![Register::CTRL_REG4_A, base_reg | (0b10 << 4)],
        )],
        AccelScale::G16 => vec![I2cTrans::write(
            ACCEL_ADDR,
            vec![Register::CTRL_REG4_A, base_reg | (0b11 << 4)],
        )],
    }
}

macro_rules! can_get_i2c {
    ( $name:ident, $mode:ident, $scale:ident, $expected_x:expr, $expected_y:expr, $expected_z:expr ) => {
        #[test]
        fn $name() {
            let mode = AccelMode::$mode;
            let scale = AccelScale::$scale;
            let mut txns: Vec<I2cTrans> = vec![I2cTrans::write(
                ACCEL_ADDR,
                vec![Register::CTRL_REG1_A, DEFAULT_CTRL_REG1_A | HZ50],
            )];
            txns.append(&mut i2c_mode_txns(&mode));
            txns.append(&mut i2c_scale_txns(&mode, &scale));
            txns.push(I2cTrans::write_read(
                ACCEL_ADDR,
                vec![Register::OUT_X_L_A | 0x80],
                vec![0x10, 0x20, 0x30, 0x40, 0x50, 0x60],
            ));
            let mut sensor = new_i2c(&txns);
            sensor
                .set_accel_odr(&mut Delay, AccelOutputDataRate::Hz50)
                .unwrap();

            if let AccelMode::LowPower | AccelMode::HighResolution = mode {
                sensor.set_accel_mode(&mut Delay, mode).unwrap();
            }
            if let AccelScale::G2 = scale {
            } else {
                sensor.set_accel_scale(scale).unwrap();
            }

            let data = sensor.acceleration().unwrap();

            assert_eq!(data.x_mg(), $expected_x);
            assert_eq!(data.y_mg(), $expected_y);
            assert_eq!(data.z_mg(), $expected_z);

            destroy_i2c(sensor);
        }
    };
}

#[rustfmt::skip]
mod can_get_i2c {
    use super::*;

    can_get_i2c!(low_power_2g,        LowPower,       G2,  512 * 1, 1024 * 1, 1536 * 1);
    can_get_i2c!(high_resolution_2g,  HighResolution, G2,  513 * 1, 1027 * 1, 1541 * 1);
    can_get_i2c!(normal_2g,           Normal,         G2,  512 * 1, 1024 * 1, 1540 * 1);
    can_get_i2c!(low_power_4g,        LowPower,       G4,  512 * 2, 1024 * 2, 1536 * 2);
    can_get_i2c!(high_resolution_4g,  HighResolution, G4,  513 * 2, 1027 * 2, 1541 * 2);
    can_get_i2c!(normal_4g,           Normal,         G4,  512 * 2, 1024 * 2, 1540 * 2);
    can_get_i2c!(low_power_8g,        LowPower,       G8,  512 * 4, 1024 * 4, 1536 * 4);
    can_get_i2c!(high_resolution_8g,  HighResolution, G8,  513 * 4, 1027 * 4, 1541 * 4);
    can_get_i2c!(normal_8g,           Normal,         G8,  512 * 4, 1024 * 4, 1540 * 4);
    can_get_i2c!(low_power_16g,       LowPower,       G16, 512 * 8, 1024 * 8, 1536 * 8);
    can_get_i2c!(high_resolution_16g, HighResolution, G16, 513 * 8, 1027 * 8, 1541 * 8);
    can_get_i2c!(normal_16g,          Normal,         G16, 512 * 8, 1024 * 8, 1540 * 8);
}

macro_rules! measurement_almost_eq {
    ( $m:expr, $x:expr, $y:expr, $z:expr, $tolerance:expr ) => {{
        let x_mg = $m.x_mg();
        let y_mg = $m.y_mg();
        let z_mg = $m.z_mg();

        assert!(
            (x_mg - $x).abs() < $tolerance,
            "x values {} and {} must be within {}",
            x_mg,
            $x,
            $tolerance
        );
        assert!(
            (y_mg - $y).abs() < $tolerance,
            "y values {} and {} must be within {}",
            y_mg,
            $y,
            $tolerance
        );
        assert!(
            (z_mg - $z).abs() < $tolerance,
            "z values {} and {} must be within {}",
            z_mg,
            $z,
            $tolerance
        );
    }};
}

macro_rules! assert_eq_xyz_mg {
    ($data:expr) => {{
        crate::assert_eq_xyz!($data, x_mg, y_mg, z_mg, xyz_mg);
    }};
}

#[test]
fn can_get_8_bit_data_i2c() {
    let mut sensor = new_i2c(&[
        I2cTrans::write(
            ACCEL_ADDR,
            vec![Register::CTRL_REG1_A, DEFAULT_CTRL_REG1_A | HZ50],
        ),
        I2cTrans::write(ACCEL_ADDR, vec![Register::CTRL_REG4_A, 0]),
        I2cTrans::write(
            ACCEL_ADDR,
            vec![
                Register::CTRL_REG1_A,
                BF::LP_EN | DEFAULT_CTRL_REG1_A | HZ50,
            ],
        ),
        I2cTrans::write_read(
            ACCEL_ADDR,
            vec![Register::OUT_X_L_A | 0x80],
            vec![0x10, 0x20, 0x30, 0x40, 0x50, 0x60],
        ),
    ]);
    sensor
        .set_accel_odr(&mut Delay, AccelOutputDataRate::Hz50)
        .unwrap();
    sensor
        .set_accel_mode(&mut Delay, AccelMode::LowPower)
        .unwrap();
    let data = sensor.acceleration().unwrap();

    assert_eq_xyz_mg!(data);

    assert_eq!(data.x_raw(), 0x2010);
    assert_eq!(data.y_raw(), 0x4030);
    assert_eq!(data.z_raw(), 0x6050);

    assert_eq!(data.x_unscaled(), 0x2010 / (1 << 8));
    assert_eq!(data.y_unscaled(), 0x4030 / (1 << 8));
    assert_eq!(data.z_unscaled(), 0x6050 / (1 << 8));

    // At 2g scale and 8 bit resolution there is 16 milli-g per
    // significant digit so we expect the result to be within 16
    // of the true result.
    measurement_almost_eq!(
        data,
        0x2010 / (1 << 4),
        0x4030 / (1 << 4),
        0x6050 / (1 << 4),
        16
    );
    destroy_i2c(sensor);
}

#[test]
fn can_get_10_bit_data_i2c() {
    let mut sensor = new_i2c(&[
        I2cTrans::write(
            ACCEL_ADDR,
            vec![Register::CTRL_REG1_A, DEFAULT_CTRL_REG1_A | HZ50],
        ),
        I2cTrans::write_read(
            ACCEL_ADDR,
            vec![Register::OUT_X_L_A | 0x80],
            vec![0x10, 0x20, 0x30, 0x40, 0x50, 0x60],
        ),
    ]);
    sensor
        .set_accel_odr(&mut Delay, AccelOutputDataRate::Hz50)
        .unwrap();
    let data = sensor.acceleration().unwrap();

    assert_eq_xyz_mg!(data);

    assert_eq!(data.x_raw(), 0x2010);
    assert_eq!(data.y_raw(), 0x4030);
    assert_eq!(data.z_raw(), 0x6050);

    assert_eq!(data.x_unscaled(), 0x2010 / (1 << 6));
    assert_eq!(data.y_unscaled(), 0x4030 / (1 << 6));
    assert_eq!(data.z_unscaled(), 0x6050 / (1 << 6));

    // at 2g scale and 10 bit resolution there is 4 milli-g per
    // significant digit so we expect the result to be within 4
    // of the true result
    measurement_almost_eq!(
        data,
        0x2010 / (1 << 4),
        0x4030 / (1 << 4),
        0x6050 / (1 << 4),
        4
    );

    destroy_i2c(sensor);
}

#[test]
fn can_get_10_bit_data_spi() {
    let mut sensor = new_spi_accel(
        &[
            SpiTrans::write(vec![Register::CTRL_REG1_A, DEFAULT_CTRL_REG1_A | HZ50]),
            SpiTrans::transfer(
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
            ),
        ],
        default_cs_n(2),
    );
    sensor
        .set_accel_odr(&mut Delay, AccelOutputDataRate::Hz50)
        .unwrap();
    let data = sensor.acceleration().unwrap();

    assert_eq_xyz_mg!(data);

    assert_eq!(data.x_raw(), 0x2010);
    assert_eq!(data.y_raw(), 0x4030);
    assert_eq!(data.z_raw(), 0x6050);

    assert_eq!(data.x_unscaled(), 0x2010 / (1 << 6));
    assert_eq!(data.y_unscaled(), 0x4030 / (1 << 6));
    assert_eq!(data.z_unscaled(), 0x6050 / (1 << 6));

    // At 2g scale there is 4 milli-g per significant digit
    // so we expect the result to be within 4 of the true result.
    measurement_almost_eq!(
        data,
        0x2010 / (1 << 4),
        0x4030 / (1 << 4),
        0x6050 / (1 << 4),
        4
    );
    destroy_spi(sensor);
}

#[test]
fn can_get_12_bit_data_i2c() {
    let mut sensor = new_i2c(&[
        I2cTrans::write(
            ACCEL_ADDR,
            vec![Register::CTRL_REG1_A, DEFAULT_CTRL_REG1_A | HZ50],
        ),
        I2cTrans::write(
            ACCEL_ADDR,
            vec![Register::CTRL_REG1_A, DEFAULT_CTRL_REG1_A | HZ50],
        ),
        I2cTrans::write(ACCEL_ADDR, vec![Register::CTRL_REG4_A, BF::HR]),
        I2cTrans::write_read(
            ACCEL_ADDR,
            vec![Register::OUT_X_L_A | 0x80],
            vec![0x10, 0x20, 0x30, 0x40, 0x50, 0x60],
        ),
    ]);
    sensor
        .set_accel_odr(&mut Delay, AccelOutputDataRate::Hz50)
        .unwrap();
    sensor
        .set_accel_mode(&mut Delay, AccelMode::HighResolution)
        .unwrap();
    let data = sensor.acceleration().unwrap();

    assert_eq_xyz_mg!(data);

    assert_eq!(data.x_raw(), 0x2010);
    assert_eq!(data.y_raw(), 0x4030);
    assert_eq!(data.z_raw(), 0x6050);

    // At 2g scale and 12 bit resolution there is 1 milli-g per
    // significant digit so we expect the result to be exactly
    // equal to the true result.
    assert_eq!(data.x_unscaled(), 0x2010 / (1 << 4));
    assert_eq!(data.y_unscaled(), 0x4030 / (1 << 4));
    assert_eq!(data.z_unscaled(), 0x6050 / (1 << 4));

    assert_eq!(data.x_mg(), 0x2010 / (1 << 4));
    assert_eq!(data.y_mg(), 0x4030 / (1 << 4));
    assert_eq!(data.z_mg(), 0x6050 / (1 << 4));

    destroy_i2c(sensor);
}
