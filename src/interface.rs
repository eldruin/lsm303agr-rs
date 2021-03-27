//! I2C/SPI interfaces

use crate::{private, BitFlags, Error};
use embedded_hal::{
    blocking::{i2c, spi},
    digital::v2::OutputPin,
};

pub(crate) const ACCEL_ADDR: u8 = 0b001_1001;
pub(crate) const MAG_ADDR: u8 = 0b001_1110;

/// I2C interface
#[derive(Debug)]
pub struct I2cInterface<I2C> {
    pub(crate) i2c: I2C,
}

/// SPI interface
#[derive(Debug)]
pub struct SpiInterface<SPI, CSXL, CSMAG> {
    pub(crate) spi: SPI,
    pub(crate) cs_xl: CSXL,
    pub(crate) cs_mag: CSMAG,
}

/// Write data
pub trait WriteData: private::Sealed {
    /// Error type
    type Error;
    /// Write to an u8 accelerometer register
    fn write_accel_register(&mut self, register: u8, data: u8) -> Result<(), Self::Error>;
    /// Write to an u8 magnetometer register
    fn write_mag_register(&mut self, register: u8, data: u8) -> Result<(), Self::Error>;
}

impl<I2C, E> WriteData for I2cInterface<I2C>
where
    I2C: i2c::Write<Error = E>,
{
    type Error = Error<E, ()>;

    fn write_accel_register(&mut self, register: u8, data: u8) -> Result<(), Self::Error> {
        let payload: [u8; 2] = [register, data];
        self.i2c.write(ACCEL_ADDR, &payload).map_err(Error::Comm)
    }

    fn write_mag_register(&mut self, register: u8, data: u8) -> Result<(), Self::Error> {
        let payload: [u8; 2] = [register, data];
        self.i2c.write(MAG_ADDR, &payload).map_err(Error::Comm)
    }
}

impl<SPI, CSXL, CSMAG, CommE, PinE> WriteData for SpiInterface<SPI, CSXL, CSMAG>
where
    SPI: spi::Write<u8, Error = CommE>,
    CSXL: OutputPin<Error = PinE>,
    CSMAG: OutputPin<Error = PinE>,
{
    type Error = Error<CommE, PinE>;

    fn write_accel_register(&mut self, register: u8, data: u8) -> Result<(), Self::Error> {
        self.cs_xl.set_low().map_err(Error::Pin)?;

        // note that multiple byte writing needs to set the MS bit
        let payload: [u8; 2] = [register, data];
        let result = self.spi.write(&payload).map_err(Error::Comm);

        self.cs_xl.set_high().map_err(Error::Pin)?;
        result
    }

    fn write_mag_register(&mut self, register: u8, data: u8) -> Result<(), Self::Error> {
        self.cs_mag.set_low().map_err(Error::Pin)?;

        // note that multiple byte writing needs to set the MS bit
        let payload: [u8; 2] = [register, data];
        let result = self.spi.write(&payload).map_err(Error::Comm);

        self.cs_mag.set_high().map_err(Error::Pin)?;
        result
    }
}

/// Read data
pub trait ReadData: private::Sealed {
    /// Error type
    type Error;
    /// Read an u8 accelerometer register
    fn read_accel_register(&mut self, register: u8) -> Result<u8, Self::Error>;
    /// Read an u8 magnetometer register
    fn read_mag_register(&mut self, register: u8) -> Result<u8, Self::Error>;

    /// Read 3 u16 accelerometer registers
    fn read_accel_3_double_registers(
        &mut self,
        register: u8,
    ) -> Result<(u16, u16, u16), Self::Error>;

    /// Read 3 u16 magnetometer registers
    fn read_mag_3_double_registers(&mut self, register: u8)
        -> Result<(u16, u16, u16), Self::Error>;
}

impl<I2C, E> ReadData for I2cInterface<I2C>
where
    I2C: i2c::WriteRead<Error = E>,
{
    type Error = Error<E, ()>;

    fn read_accel_register(&mut self, register: u8) -> Result<u8, Self::Error> {
        self.read_register(ACCEL_ADDR, register)
    }

    fn read_mag_register(&mut self, register: u8) -> Result<u8, Self::Error> {
        self.read_register(MAG_ADDR, register)
    }

    fn read_accel_3_double_registers(
        &mut self,
        register: u8,
    ) -> Result<(u16, u16, u16), Self::Error> {
        self.read_3_double_registers(ACCEL_ADDR, register)
    }

    fn read_mag_3_double_registers(
        &mut self,
        register: u8,
    ) -> Result<(u16, u16, u16), Self::Error> {
        self.read_3_double_registers(MAG_ADDR, register)
    }
}

impl<I2C, E> I2cInterface<I2C>
where
    I2C: i2c::WriteRead<Error = E>,
{
    fn read_register(&mut self, address: u8, register: u8) -> Result<u8, Error<E, ()>> {
        let mut data = [0];
        self.i2c
            .write_read(address, &[register], &mut data)
            .map_err(Error::Comm)
            .and(Ok(data[0]))
    }

    fn read_3_double_registers(
        &mut self,
        address: u8,
        start: u8,
    ) -> Result<(u16, u16, u16), Error<E, ()>> {
        let mut data = [0; 6];
        self.i2c
            .write_read(address, &[start | 0x80], &mut data)
            .map_err(Error::Comm)?;
        Ok((
            u16::from(data[0]) | (u16::from(data[1]) << 8),
            u16::from(data[2]) | (u16::from(data[3]) << 8),
            u16::from(data[4]) | (u16::from(data[5]) << 8),
        ))
    }
}

impl<SPI, CSXL, CSMAG, CommE, PinE> ReadData for SpiInterface<SPI, CSXL, CSMAG>
where
    SPI: spi::Transfer<u8, Error = CommE>,
    CSXL: OutputPin<Error = PinE>,
    CSMAG: OutputPin<Error = PinE>,
{
    type Error = Error<CommE, PinE>;

    fn read_accel_register(&mut self, register: u8) -> Result<u8, Self::Error> {
        self.cs_xl.set_low().map_err(Error::Pin)?;
        let result = self.read_register(register);
        self.cs_xl.set_high().map_err(Error::Pin)?;
        result
    }

    fn read_mag_register(&mut self, register: u8) -> Result<u8, Self::Error> {
        self.cs_mag.set_low().map_err(Error::Pin)?;
        let result = self.read_register(register);
        self.cs_mag.set_high().map_err(Error::Pin)?;
        result
    }

    fn read_accel_3_double_registers(
        &mut self,
        register: u8,
    ) -> Result<(u16, u16, u16), Self::Error> {
        self.cs_xl.set_low().map_err(Error::Pin)?;
        let result = self.read_3_double_registers(register);
        self.cs_xl.set_high().map_err(Error::Pin)?;
        result
    }

    fn read_mag_3_double_registers(
        &mut self,
        register: u8,
    ) -> Result<(u16, u16, u16), Self::Error> {
        self.cs_mag.set_low().map_err(Error::Pin)?;
        let result = self.read_3_double_registers(register);
        self.cs_mag.set_high().map_err(Error::Pin)?;
        result
    }
}

impl<SPI, CSXL, CSMAG, CommE, PinE> SpiInterface<SPI, CSXL, CSMAG>
where
    SPI: spi::Transfer<u8, Error = CommE>,
    CSXL: OutputPin<Error = PinE>,
    CSMAG: OutputPin<Error = PinE>,
{
    fn read_register(&mut self, register: u8) -> Result<u8, Error<CommE, PinE>> {
        let mut data = [BitFlags::SPI_RW | register, 0];
        let value = self.spi.transfer(&mut data).map_err(Error::Comm)?;
        Ok(value[1])
    }

    fn read_3_double_registers(
        &mut self,
        register: u8,
    ) -> Result<(u16, u16, u16), Error<CommE, PinE>> {
        let mut data = [
            BitFlags::SPI_RW | BitFlags::SPI_MS | register,
            0,
            0,
            0,
            0,
            0,
            0,
        ];
        let value = self.spi.transfer(&mut data).map_err(Error::Comm)?;
        Ok((
            u16::from(value[1]) | (u16::from(value[2]) << 8),
            u16::from(value[3]) | (u16::from(value[4]) << 8),
            u16::from(value[5]) | (u16::from(value[6]) << 8),
        ))
    }
}
