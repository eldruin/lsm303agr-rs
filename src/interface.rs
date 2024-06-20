//! I2C/SPI interfaces

#[cfg(not(feature = "async"))]
use embedded_hal::{i2c, spi};
#[cfg(feature = "async")]
use embedded_hal_async::{i2c, spi};
use maybe_async_cfg::maybe;

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
#[maybe(
    sync(
        cfg(not(feature = "async")),
        keep_self,
    ),
    async (
        cfg(feature = "async"),
        keep_self,
    )
)]
pub trait WriteData: private::Sealed {
    /// Error type
    type Error;

    /// Write to an u8 accelerometer register
    async fn write_accel_register<R: RegWrite>(&mut self, reg: R) -> Result<(), Self::Error>;

    /// Write to an u8 magnetometer register
    async fn write_mag_register<R: RegWrite>(&mut self, reg: R) -> Result<(), Self::Error>;
}

#[maybe(
    sync(
        cfg(not(feature = "async")),
        keep_self,
    ),
    async (
        cfg(feature = "async"),
        keep_self,
    )
)]
impl<I2C, E> WriteData for I2cInterface<I2C>
where
    I2C: i2c::I2c<Error = E>,
{
    type Error = Error<E>;

    async fn write_accel_register<R: RegWrite>(&mut self, reg: R) -> Result<(), Self::Error> {
        let payload: [u8; 2] = [R::ADDR, reg.data()];
        self.i2c.write(ACCEL_ADDR, &payload).await.map_err(Error::Comm)
    }

    async fn write_mag_register<R: RegWrite>(&mut self, reg: R) -> Result<(), Self::Error> {
        let payload: [u8; 2] = [R::ADDR, reg.data()];
        self.i2c.write(MAG_ADDR, &payload).await.map_err(Error::Comm)
    }
}

#[maybe(
    sync(
        cfg(not(feature = "async")),
        keep_self,
    ),
    async (
        cfg(feature = "async"),
        keep_self,
    )
)]
impl<SPIXL, SPIMAG, CommE> WriteData for SpiInterface<SPIXL, SPIMAG>
where
    SPIXL: spi::SpiDevice<u8, Error = CommE>,
    SPIMAG: spi::SpiDevice<u8, Error = CommE>,
{
    type Error = Error<CommE>;

    async fn write_accel_register<R: RegWrite>(&mut self, reg: R) -> Result<(), Self::Error> {
        // note that multiple byte writing needs to set the MS bit
        let payload: [u8; 2] = [R::ADDR, reg.data()];
        self.spi_xl.write(&payload).await.map_err(Error::Comm)
    }

    async fn write_mag_register<R: RegWrite>(&mut self, reg: R) -> Result<(), Self::Error> {
        // note that multiple byte writing needs to set the MS bit
        let payload: [u8; 2] = [R::ADDR, reg.data()];
        self.spi_mag.write(&payload).await.map_err(Error::Comm)
    }
}

/// Read data
#[maybe(
    sync(
        cfg(not(feature = "async")),
        keep_self,
    ),
    async (
        cfg(feature = "async"),
        keep_self,
    )
)]
pub trait ReadData: private::Sealed {
    /// Error type
    type Error;

    /// Read an u8 accelerometer register
    async fn read_accel_register<R: RegRead>(&mut self) -> Result<R::Output, Self::Error>;

    /// Read an u8 magnetometer register
    async fn read_mag_register<R: RegRead>(&mut self) -> Result<R::Output, Self::Error>;

    /// Read an u16 accelerometer register
    async fn read_accel_double_register<R: RegRead<u16>>(&mut self) -> Result<R::Output, Self::Error>;

    /// Read 3 u16 accelerometer registers
    async fn read_accel_3_double_registers<R: RegRead<(u16, u16, u16)>>(
        &mut self,
    ) -> Result<R::Output, Self::Error>;

    /// Read 3 u16 magnetometer registers
    async fn read_mag_3_double_registers<R: RegRead<(u16, u16, u16)>>(
        &mut self,
    ) -> Result<R::Output, Self::Error>;
}

#[maybe(
    sync(
        cfg(not(feature = "async")),
        keep_self,
    ),
    async (
        cfg(feature = "async"),
        keep_self,
    )
)]
impl<I2C, E> ReadData for I2cInterface<I2C>
where
    I2C: i2c::I2c<Error = E>,
{
    type Error = Error<E>;

    async fn read_accel_register<R: RegRead>(&mut self) -> Result<R::Output, Self::Error> {
        self.read_register::<R>(ACCEL_ADDR).await
    }

    async fn read_mag_register<R: RegRead>(&mut self) -> Result<R::Output, Self::Error> {
        self.read_register::<R>(MAG_ADDR).await
    }

    async fn read_accel_double_register<R: RegRead<u16>>(&mut self) -> Result<R::Output, Self::Error> {
        self.read_double_register::<R>(ACCEL_ADDR).await
    }

    async fn read_accel_3_double_registers<R: RegRead<(u16, u16, u16)>>(
        &mut self,
    ) -> Result<R::Output, Self::Error> {
        self.read_3_double_registers::<R>(ACCEL_ADDR).await
    }

    async fn read_mag_3_double_registers<R: RegRead<(u16, u16, u16)>>(
        &mut self,
    ) -> Result<R::Output, Self::Error> {
        self.read_3_double_registers::<R>(MAG_ADDR).await
    }
}

#[maybe(
    sync(
        cfg(not(feature = "async")),
        keep_self,
    ),
    async (
        cfg(feature = "async"),
        keep_self,
    )
)]
impl<I2C, E> I2cInterface<I2C>
where
    I2C: i2c::I2c<Error = E>,
{
    async fn read_register<R: RegRead>(&mut self, address: u8) -> Result<R::Output, Error<E>> {
        let mut data = [0];
        self.i2c
            .write_read(address, &[R::ADDR], &mut data).await
            .map_err(Error::Comm)?;

        Ok(R::from_data(data[0]))
    }

    async fn read_double_register<R: RegRead<u16>>(
        &mut self,
        address: u8,
    ) -> Result<R::Output, Error<E>> {
        let mut data = [0; 2];
        self.i2c
            .write_read(address, &[R::ADDR | 0x80], &mut data).await
            .map_err(Error::Comm)?;

        Ok(R::from_data(u16::from_le_bytes(data)))
    }

    async fn read_3_double_registers<R: RegRead<(u16, u16, u16)>>(
        &mut self,
        address: u8,
    ) -> Result<R::Output, Error<E>> {
        let mut data = [0; 6];
        self.i2c
            .write_read(address, &[R::ADDR | 0x80], &mut data).await
            .map_err(Error::Comm)?;

        Ok(R::from_data((
            u16::from_le_bytes([data[0], data[1]]),
            u16::from_le_bytes([data[2], data[3]]),
            u16::from_le_bytes([data[4], data[5]]),
        )))
    }
}

#[maybe(
    sync(
        cfg(not(feature = "async")),
        keep_self,
    ),
    async (
        cfg(feature = "async"),
        keep_self,
    )
)]
impl<SPIXL, SPIMAG, CommE> ReadData for SpiInterface<SPIXL, SPIMAG>
where
    SPIXL: spi::SpiDevice<u8, Error = CommE>,
    SPIMAG: spi::SpiDevice<u8, Error = CommE>,
{
    type Error = Error<CommE>;

    async fn read_accel_register<R: RegRead>(&mut self) -> Result<R::Output, Self::Error> {
        spi_read_register::<R, _, _>(&mut self.spi_xl).await
    }

    async fn read_mag_register<R: RegRead>(&mut self) -> Result<R::Output, Self::Error> {
        spi_read_register::<R, _, _>(&mut self.spi_mag).await
    }

    async fn read_accel_double_register<R: RegRead<u16>>(&mut self) -> Result<R::Output, Self::Error> {
        spi_read_double_register::<R, _, _>(&mut self.spi_xl).await
    }

    async fn read_accel_3_double_registers<R: RegRead<(u16, u16, u16)>>(
        &mut self,
    ) -> Result<R::Output, Self::Error> {
        spi_read_3_double_registers::<R, _, _>(&mut self.spi_xl).await
    }

    async fn read_mag_3_double_registers<R: RegRead<(u16, u16, u16)>>(
        &mut self,
    ) -> Result<R::Output, Self::Error> {
        spi_read_3_double_registers::<R, _, _>(&mut self.spi_mag).await
    }
}

const SPI_RW: u8 = 1 << 7;
const SPI_MS: u8 = 1 << 6;

#[maybe(
    sync(
        cfg(not(feature = "async")),
        keep_self,
    ),
    async (
        cfg(feature = "async"),
        keep_self,
    )
)]
async fn spi_read_register<R: RegRead, SPI: spi::SpiDevice<u8, Error = CommE>, CommE>(
    spi: &mut SPI,
) -> Result<R::Output, Error<CommE>> {
    let mut data = [SPI_RW | R::ADDR, 0];
    spi.transfer_in_place(&mut data).await.map_err(Error::Comm)?;

    Ok(R::from_data(data[1]))
}

#[maybe(
    sync(
        cfg(not(feature = "async")),
        keep_self,
    ),
    async (
        cfg(feature = "async"),
        keep_self,
    )
)]
async fn spi_read_double_register<R: RegRead<u16>, SPI: spi::SpiDevice<u8, Error = CommE>, CommE>(
    spi: &mut SPI,
) -> Result<R::Output, Error<CommE>> {
    let mut data = [SPI_RW | SPI_MS | R::ADDR, 0, 0];
    spi.transfer_in_place(&mut data).await.map_err(Error::Comm)?;

    Ok(R::from_data(u16::from_le_bytes([data[1], data[2]])))
}

#[maybe(
    sync(
        cfg(not(feature = "async")),
        keep_self,
    ),
    async (
        cfg(feature = "async"),
        keep_self,
    )
)]
async fn spi_read_3_double_registers<
    R: RegRead<(u16, u16, u16)>,
    SPI: spi::SpiDevice<u8, Error = CommE>,
    CommE,
>(
    spi: &mut SPI,
) -> Result<R::Output, Error<CommE>> {
    let mut data = [SPI_RW | SPI_MS | R::ADDR, 0, 0, 0, 0, 0, 0];
    spi.transfer_in_place(&mut data).await.map_err(Error::Comm)?;

    Ok(R::from_data((
        u16::from_le_bytes([data[1], data[2]]),
        u16::from_le_bytes([data[3], data[4]]),
        u16::from_le_bytes([data[5], data[6]]),
    )))
}
