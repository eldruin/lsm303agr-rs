The given `delay` is used to wait for the sensor to turn on or change modes,
according to the times specified in Table 14 and Table 15 in the [datasheet].
You can opt out of this by using a no-op delay implementation, see
[`embedded_hal_mock::delay::MockNoop`] for an example of such an
implementation.

[datasheet]: https://www.st.com/resource/en/datasheet/lsm303agr.pdf
[`embedded_hal_mock::delay::MockNoop`]: https://docs.rs/embedded-hal-mock/latest/embedded_hal_mock/delay/struct.MockNoop.html
