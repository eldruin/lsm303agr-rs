use lsm303agr::Status;
mod common;
use crate::common::{destroy_i2c, new_i2c, BitFlags as BF, Register, ACCEL_ADDR, MAG_ADDR};
use embedded_hal_mock::i2c::Transaction as I2cTrans;

macro_rules! status_eq {
    ($st:expr, $xyz_overrun:expr, $x_overrun:expr, $y_overrun:expr, $z_overrun:expr,
    $xyz_new_data:expr, $x_new_data:expr, $y_new_data:expr, $z_new_data:expr) => {
        assert_eq!(
            Status {
                xyz_overrun: $xyz_overrun,
                x_overrun: $x_overrun,
                y_overrun: $y_overrun,
                z_overrun: $z_overrun,
                xyz_new_data: $xyz_new_data,
                x_new_data: $x_new_data,
                y_new_data: $y_new_data,
                z_new_data: $z_new_data
            },
            $st
        );
    };
}

macro_rules! get_st_test {
    ($name:ident, $st:expr, $xyz_overrun:expr,
        $x_overrun:expr,
        $y_overrun:expr,
        $z_overrun:expr,
        $xyz_new_data:expr,
        $x_new_data:expr,
        $y_new_data:expr,
        $z_new_data:expr) => {
        mod $name {
            use super::*;
            mod accel {
                use super::*;
                #[test]
                fn $name() {
                    let mut sensor = new_i2c(&[I2cTrans::write_read(
                        ACCEL_ADDR,
                        vec![Register::STATUS_REG_A],
                        vec![$st],
                    )]);
                    let st = sensor.accel_status().unwrap();
                    status_eq!(
                        st,
                        $xyz_overrun,
                        $x_overrun,
                        $y_overrun,
                        $z_overrun,
                        $xyz_new_data,
                        $x_new_data,
                        $y_new_data,
                        $z_new_data
                    );
                    destroy_i2c(sensor);
                }
            }
            mod mag {
                use super::*;
                #[test]
                fn $name() {
                    let mut sensor = new_i2c(&[I2cTrans::write_read(
                        MAG_ADDR,
                        vec![Register::STATUS_REG_M],
                        vec![$st],
                    )]);
                    let st = sensor.mag_status().unwrap();
                    status_eq!(
                        st,
                        $xyz_overrun,
                        $x_overrun,
                        $y_overrun,
                        $z_overrun,
                        $xyz_new_data,
                        $x_new_data,
                        $y_new_data,
                        $z_new_data
                    );
                    destroy_i2c(sensor);
                }
            }
        }
    };
}

get_st_test!(
    xyz_overrun,
    BF::XYZOR,
    true,
    false,
    false,
    false,
    false,
    false,
    false,
    false
);

get_st_test!(
    x_overrun,
    BF::XOR,
    false,
    true,
    false,
    false,
    false,
    false,
    false,
    false
);

get_st_test!(
    y_overrun,
    BF::YOR,
    false,
    false,
    true,
    false,
    false,
    false,
    false,
    false
);

get_st_test!(
    z_overrun,
    BF::ZOR,
    false,
    false,
    false,
    true,
    false,
    false,
    false,
    false
);

get_st_test!(
    xyz_data_ready,
    BF::XYZDR,
    false,
    false,
    false,
    false,
    true,
    false,
    false,
    false
);

get_st_test!(
    x_data_ready,
    BF::XDR,
    false,
    false,
    false,
    false,
    false,
    true,
    false,
    false
);

get_st_test!(
    y_data_ready,
    BF::YDR,
    false,
    false,
    false,
    false,
    false,
    false,
    true,
    false
);

get_st_test!(
    z_data_ready,
    BF::ZDR,
    false,
    false,
    false,
    false,
    false,
    false,
    false,
    true
);

get_st_test!(all, 0xFF, true, true, true, true, true, true, true, true);
