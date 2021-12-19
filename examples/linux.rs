#[cfg(not(target_os = "linux"))]
fn main() {
    // This exists to let `cargo test` succeed on non-linux platforms
    panic!("This example can only run on linux");
}

#[cfg(target_os = "linux")]
fn main() {
    use linux_embedded_hal::I2cdev;
    use lsm303agr::{AccelOutputDataRate, Lsm303agr};

    let dev = I2cdev::new("/dev/i2c-1").unwrap();
    let mut sensor = Lsm303agr::new_with_i2c(dev);

    sensor.init().unwrap();
    sensor.set_accel_odr(AccelOutputDataRate::Hz50).unwrap();

    loop {
        if sensor.accel_status().unwrap().xyz_new_data {
            let data = sensor.accel_data().unwrap();
            println!("Acceleration: x {} y {} z {}", data.x, data.y, data.z);
        }
    }
}
