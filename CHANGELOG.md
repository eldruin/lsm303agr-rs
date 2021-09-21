# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

## [Unreleased]

...
## [0.2.2] - 2021-09-21

### Fixed
- Document that the scaled magnetometer data is returned in nanoteslas (nT),
  not milligauss (mG). Thanks to @hargoniX.

## [0.2.1] - 2021-09-01

### Changed
- `Measurement` and `UnscaledMeasurement` now implement `Copy`.

## [0.2.0] - 2021-07-13

### Added
- Methods to verify device IDs. Thanks to @robyoung.
- Support setting accelerometer scale via `set_accel_scale()`. Thanks to @robyoung.

### Changed
- [breaking-change] `accel_data()` and `mag_data()` now return scaled measurements.
  To get unscaled data, use the methods `accel_data_unscaled()` and `mag_data_unscaled()`.
  Thanks to @robyoung.

### Fixed
- Reset all ODR bits on powerdown. Thanks to @robyoung.

## [0.1.1] - 2021-03-27

### Added
- Support setting magnetometer output data rate.
- Support magnetometer mode change.
- Support reading magnetometer data.

### Fixed
- Derive `Debug` for device operation `mode` markers. Thanks to @chrysn.

## [0.1.0] - 2020-09-13

Initial release to crates.io.

[Unreleased]: https://github.com/eldruin/lsm303agr-rs/compare/v0.2.2...HEAD
[0.2.2]: https://github.com/eldruin/lsm303agr-rs/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/eldruin/lsm303agr-rs/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/eldruin/lsm303agr-rs/compare/v0.1.1...v0.2.0
[0.1.1]: https://github.com/eldruin/lsm303agr-rs/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/eldruin/lsm303agr-rs/releases/tag/v0.1.0