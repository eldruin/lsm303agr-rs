# Rust LSM303AGR Ultra-low-power 3D Accelerometer and 3D Magnetometer Driver

[![crates.io](https://img.shields.io/crates/v/lsm303agr.svg)](https://crates.io/crates/lsm303agr)
[![Docs](https://docs.rs/lsm303agr/badge.svg)](https://docs.rs/lsm303agr)
[![Build Status](https://github.com/eldruin/lsm303agr-rs/workflows/Build/badge.svg)](https://github.com/eldruin/lsm303agr-rs/actions?query=workflow%3ABuild)
[![Coverage Status](https://coveralls.io/repos/github/eldruin/lsm303agr-rs/badge.svg?branch=master)](https://coveralls.io/github/eldruin/lsm303agr-rs?branch=master)

This is a platform agnostic Rust driver for the LSM303AGR ultra-compact
high-performance eCompass module: ultra-low-power 3D accelerometer and
3D magnetometer using the [`embedded-hal`] traits.

This driver allows you to:
- Connect through I2C or SPI. See: `new_with_i2c()`.
- Initialize the device. See: `init()`.
- Accelerometer:
    - Read accelerometer data. See: `accel_data()`.
    - Read accelerometer data unscaled. See: `accel_data_unscaled()`.
    - Get accelerometer status. See: `accel_status()`.
    - Set accelerometer output data rate. See: `set_accel_odr()`.
    - Set accelerometer mode. See: `set_accel_mode()`.
    - Set accelerometer scale. See: `set_accel_scale()`.
    - Get accelerometer ID. See: `accelerometer_id()`.
- Magnetometer:
    - Get the magnetometer status. See: `mag_status()`.
    - Change into continuous/one-shot mode. See: `into_mag_continuous()`.
    - Read magnetometer data. See: `mag_data()`.
    - Read magnetometer data unscaled. See: `mag_data_unscaled()`.
    - Set magnetometer output data rate. See: `set_mag_odr()`.
    - Get magnetometer ID. See: `magnetometer_id()`.

<!-- TODO
[Introductory blog post]()
-->

The LSM303AGR is an ultra-low-power high- performance system-in-package featuring
a 3D digital linear acceleration sensor and a 3D digital magnetic sensor.
The LSM303AGR has linear acceleration full scales of ±2g/±4g/±8g/±16g and a magnetic
field dynamic range of ±50 gauss.

The LSM303AGR includes an I2C serial bus interface that supports standard, fast mode,
fast mode plus, and high-speed (100 kHz, 400 kHz, 1 MHz, and 3.4 MHz) and an SPI serial
standard interface.

The system can be configured to generate an interrupt signal for free-fall, motion
detection and magnetic field detection.

The magnetic and accelerometer blocks can be enabled or put into power-down mode separately.

Documents: [Datasheet](https://www.st.com/resource/en/datasheet/lsm303agr.pdf) - [Application note](https://www.st.com/resource/en/application_note/dm00265383-ultracompact-highperformance-ecompass-module-based-on-the-lsm303agr-stmicroelectronics.pdf)

## Usage

To use this driver, import this crate and an `embedded_hal` implementation,
then instantiate the device.

Please find additional examples using hardware in this repository: [driver-examples]

[driver-examples]: https://github.com/eldruin/driver-examples

```rust
use linux_embedded_hal::I2cdev;
use lsm303agr::{AccelOutputDataRate, Lsm303agr};

fn main() {
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
```

## Support

For questions, issues, feature requests, and other changes, please file an
[issue in the github project](https://github.com/eldruin/lsm303agr-rs/issues).

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
   http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   http://opensource.org/licenses/MIT)

at your option.

### Contributing

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.

[`embedded-hal`]: https://github.com/rust-embedded/embedded-hal
