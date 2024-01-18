//! I2C/SPI interfaces

use embedded_hal::{i2c, spi};

use crate::{
    private,
    register_address::{RegRead, RegWrite},
    Error,
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
pub struct SpiInterface<SPIXL, SPIMAG> {
    pub(crate) spi_xl: SPIXL,
    pub(crate) spi_mag: SPIMAG,
}

/// Write data
pub trait WriteData: private::Sealed {
    /// Error type
    type Error;

    /// Write to an u8 accelerometer register
    fn write_accel_register<R: RegWrite>(&mut self, reg: R) -> Result<(), Self::Error>;

    /// Write to an u8 magnetometer register
    fn write_mag_register<R: RegWrite>(&mut self, reg: R) -> Result<(), Self::Error>;
}

impl<I2C, E> WriteData for I2cInterface<I2C>
where
    I2C: i2c::I2c<Error = E>,
{
    type Error = Error<E>;

    fn write_accel_register<R: RegWrite>(&mut self, reg: R) -> Result<(), Self::Error> {
        let payload: [u8; 2] = [R::ADDR, reg.data()];
        self.i2c.write(ACCEL_ADDR, &payload).map_err(Error::Comm)
    }

    fn write_mag_register<R: RegWrite>(&mut self, reg: R) -> Result<(), Self::Error> {
        let payload: [u8; 2] = [R::ADDR, reg.data()];
        self.i2c.write(MAG_ADDR, &payload).map_err(Error::Comm)
    }
}

impl<SPIXL, SPIMAG, CommE> WriteData for SpiInterface<SPIXL, SPIMAG>
where
    SPIXL: spi::SpiDevice<u8, Error = CommE>,
    SPIMAG: spi::SpiDevice<u8, Error = CommE>,
{
    type Error = Error<CommE>;

    fn write_accel_register<R: RegWrite>(&mut self, reg: R) -> Result<(), Self::Error> {
        // note that multiple byte writing needs to set the MS bit
        let payload: [u8; 2] = [R::ADDR, reg.data()];
        self.spi_xl.write(&payload).map_err(Error::Comm)
    }

    fn write_mag_register<R: RegWrite>(&mut self, reg: R) -> Result<(), Self::Error> {
        // note that multiple byte writing needs to set the MS bit
        let payload: [u8; 2] = [R::ADDR, reg.data()];
        self.spi_mag.write(&payload).map_err(Error::Comm)
    }
}

/// Read data
pub trait ReadData: private::Sealed {
    /// Error type
    type Error;

    /// Read an u8 accelerometer register
    fn read_accel_register<R: RegRead>(&mut self) -> Result<R::Output, Self::Error>;

    /// Read an u8 magnetometer register
    fn read_mag_register<R: RegRead>(&mut self) -> Result<R::Output, Self::Error>;

    /// Read an u16 accelerometer register
    fn read_accel_double_register<R: RegRead<u16>>(&mut self) -> Result<R::Output, Self::Error>;

    /// Read 3 u16 accelerometer registers
    fn read_accel_3_double_registers<R: RegRead<(u16, u16, u16)>>(
        &mut self,
    ) -> Result<R::Output, Self::Error>;

    /// Read 3 u16 magnetometer registers
    fn read_mag_3_double_registers<R: RegRead<(u16, u16, u16)>>(
        &mut self,
    ) -> Result<R::Output, Self::Error>;
}

impl<I2C, E> ReadData for I2cInterface<I2C>
where
    I2C: i2c::I2c<Error = E>,
{
    type Error = Error<E>;

    fn read_accel_register<R: RegRead>(&mut self) -> Result<R::Output, Self::Error> {
        self.read_register::<R>(ACCEL_ADDR)
    }

    fn read_mag_register<R: RegRead>(&mut self) -> Result<R::Output, Self::Error> {
        self.read_register::<R>(MAG_ADDR)
    }

    fn read_accel_double_register<R: RegRead<u16>>(&mut self) -> Result<R::Output, Self::Error> {
        self.read_double_register::<R>(ACCEL_ADDR)
    }

    fn read_accel_3_double_registers<R: RegRead<(u16, u16, u16)>>(
        &mut self,
    ) -> Result<R::Output, Self::Error> {
        self.read_3_double_registers::<R>(ACCEL_ADDR)
    }

    fn read_mag_3_double_registers<R: RegRead<(u16, u16, u16)>>(
        &mut self,
    ) -> Result<R::Output, Self::Error> {
        self.read_3_double_registers::<R>(MAG_ADDR)
    }
}

impl<I2C, E> I2cInterface<I2C>
where
    I2C: i2c::I2c<Error = E>,
{
    fn read_register<R: RegRead>(&mut self, address: u8) -> Result<R::Output, Error<E>> {
        let mut data = [0];
        self.i2c
            .write_read(address, &[R::ADDR], &mut data)
            .map_err(Error::Comm)?;

        Ok(R::from_data(data[0]))
    }

    fn read_double_register<R: RegRead<u16>>(
        &mut self,
        address: u8,
    ) -> Result<R::Output, Error<E>> {
        let mut data = [0; 2];
        self.i2c
            .write_read(address, &[R::ADDR | 0x80], &mut data)
            .map_err(Error::Comm)?;

        Ok(R::from_data(u16::from_le_bytes(data)))
    }

    fn read_3_double_registers<R: RegRead<(u16, u16, u16)>>(
        &mut self,
        address: u8,
    ) -> Result<R::Output, Error<E>> {
        let mut data = [0; 6];
        self.i2c
            .write_read(address, &[R::ADDR | 0x80], &mut data)
            .map_err(Error::Comm)?;

        Ok(R::from_data((
            u16::from_le_bytes([data[0], data[1]]),
            u16::from_le_bytes([data[2], data[3]]),
            u16::from_le_bytes([data[4], data[5]]),
        )))
    }
}

impl<SPIXL, SPIMAG, CommE> ReadData for SpiInterface<SPIXL, SPIMAG>
where
    SPIXL: spi::SpiDevice<u8, Error = CommE>,
    SPIMAG: spi::SpiDevice<u8, Error = CommE>,
{
    type Error = Error<CommE>;

    fn read_accel_register<R: RegRead>(&mut self) -> Result<R::Output, Self::Error> {
        spi_read_register::<R, _, _>(&mut self.spi_xl)
    }

    fn read_mag_register<R: RegRead>(&mut self) -> Result<R::Output, Self::Error> {
        spi_read_register::<R, _, _>(&mut self.spi_mag)
    }

    fn read_accel_double_register<R: RegRead<u16>>(&mut self) -> Result<R::Output, Self::Error> {
        spi_read_double_register::<R, _, _>(&mut self.spi_xl)
    }

    fn read_accel_3_double_registers<R: RegRead<(u16, u16, u16)>>(
        &mut self,
    ) -> Result<R::Output, Self::Error> {
        spi_read_3_double_registers::<R, _, _>(&mut self.spi_xl)
    }

    fn read_mag_3_double_registers<R: RegRead<(u16, u16, u16)>>(
        &mut self,
    ) -> Result<R::Output, Self::Error> {
        spi_read_3_double_registers::<R, _, _>(&mut self.spi_mag)
    }
}

const SPI_RW: u8 = 1 << 7;
const SPI_MS: u8 = 1 << 6;

fn spi_read_register<R: RegRead, SPI: spi::SpiDevice<u8, Error = CommE>, CommE>(
    spi: &mut SPI,
) -> Result<R::Output, Error<CommE>> {
    let mut data = [SPI_RW | R::ADDR, 0];
    spi.transfer_in_place(&mut data).map_err(Error::Comm)?;

    Ok(R::from_data(data[1]))
}

fn spi_read_double_register<R: RegRead<u16>, SPI: spi::SpiDevice<u8, Error = CommE>, CommE>(
    spi: &mut SPI,
) -> Result<R::Output, Error<CommE>> {
    let mut data = [SPI_RW | SPI_MS | R::ADDR, 0, 0];
    spi.transfer_in_place(&mut data).map_err(Error::Comm)?;

    Ok(R::from_data(u16::from_le_bytes([data[1], data[2]])))
}

fn spi_read_3_double_registers<
    R: RegRead<(u16, u16, u16)>,
    SPI: spi::SpiDevice<u8, Error = CommE>,
    CommE,
>(
    spi: &mut SPI,
) -> Result<R::Output, Error<CommE>> {
    let mut data = [SPI_RW | SPI_MS | R::ADDR, 0, 0, 0, 0, 0, 0];
    spi.transfer_in_place(&mut data).map_err(Error::Comm)?;

    Ok(R::from_data((
        u16::from_le_bytes([data[1], data[2]]),
        u16::from_le_bytes([data[3], data[4]]),
        u16::from_le_bytes([data[5], data[6]]),
    )))
}
