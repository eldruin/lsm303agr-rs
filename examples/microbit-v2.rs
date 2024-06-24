#![no_main]
#![no_std]

//! Example of using the async feature of the LSM303AGR driver.
//! Uses [embassy](https://embassy.dev) for the HAL and the executor.
//!
//! Make sure [probe-rs](https://probe.rs) is installed.
//!
//! Run me using
//!
//! ```sh
//! cargo run --example microbit-v2 --target thumbv7em-none-eabihf --features async,microbit-example
//! ```

use embassy_nrf::{self as hal, twim::Twim};
use embassy_time::Delay;
use embedded_hal_async::delay::DelayNs;
use hal::twim;
use lsm303agr::Lsm303agr;
use rtt_target::{rprintln, rtt_init_print};

use panic_rtt_target as _; // Panic handler

hal::bind_interrupts!(struct Irqs {
    SPIM0_SPIS0_TWIM0_TWIS0_SPI0_TWI0 => twim::InterruptHandler<hal::peripherals::TWISPI0>;
});

#[embassy_executor::main]
async fn main(_s: embassy_executor::Spawner) -> ! {
    // Init RTT control block
    rtt_init_print!();

    let _cp = cortex_m::Peripherals::take().unwrap();
    // Use ``dp` to get a handle to the peripherals
    let dp = hal::init(Default::default());

    rprintln!("Starting");

    let config = twim::Config::default();
    let twim0 = Twim::new(dp.TWISPI0, Irqs, dp.P0_16, dp.P0_08, config);

    let mut sensor = Lsm303agr::new_with_i2c(twim0);
    let id = sensor.magnetometer_id().await.unwrap();
    rprintln!("{:#02x?}", id);

    sensor.init().await.unwrap();
    sensor
        .set_mag_mode_and_odr(
            &mut Delay,
            lsm303agr::MagMode::HighResolution,
            lsm303agr::MagOutputDataRate::Hz10,
        )
        .await
        .unwrap();

    let Ok(mut sensor) = sensor.into_mag_continuous().await else {
        panic!("Error enabling continuous mode")
    };
    sensor.mag_enable_low_pass_filter().await.unwrap();
    loop {
        if sensor.mag_status().await.unwrap().xyz_new_data() {
            let data = sensor.magnetic_field().await.unwrap();
            rprintln!(
                "Magnetic field: x {} y {} z {}",
                data.x_nt(),
                data.y_nt(),
                data.z_nt()
            );
        } else {
            rprintln!("No data")
        }
        Delay.delay_ms(200).await;
    }
}
