pub struct Register;

impl Register {
    pub const WHO_AM_I_A: u8 = 0x0F;
    pub const TEMP_CFG_REG_A: u8 = 0x1F;
    pub const CTRL_REG1_A: u8 = 0x20;
    pub const CTRL_REG4_A: u8 = 0x23;
    pub const STATUS_REG_A: u8 = 0x27;
    pub const OUT_X_L_A: u8 = 0x28;
    pub const WHO_AM_I_M: u8 = 0x4F;
    pub const CFG_REG_A_M: u8 = 0x60;
    pub const CFG_REG_B_M: u8 = 0x61;
    pub const CFG_REG_C_M: u8 = 0x62;
    pub const STATUS_REG_M: u8 = 0x67;
    pub const OUTX_L_REG_M: u8 = 0x68;
}

pub const WHO_AM_I_A_VAL: u8 = 0x33;
pub const WHO_AM_I_M_VAL: u8 = 0x40;

pub struct BitFlags;

impl BitFlags {
    pub const SPI_RW: u8 = 1 << 7;
    pub const SPI_MS: u8 = 1 << 6;

    pub const LP_EN: u8 = 1 << 3;

    pub const ACCEL_BDU: u8 = 1 << 7;
    pub const HR: u8 = 1 << 3;

    pub const MAG_BDU: u8 = 1 << 4;

    pub const MAG_OFF_CANC: u8 = 1 << 1;
    pub const MAG_OFF_CANC_ONE_SHOT: u8 = 1 << 4;

    pub const TEMP_EN0: u8 = 1 << 6;
    pub const TEMP_EN1: u8 = 1 << 7;
    pub const TEMP_EN: u8 = Self::TEMP_EN0 | Self::TEMP_EN1;
}

pub trait RegRead<D = u8> {
    type Output;

    const ADDR: u8;

    fn from_data(data: D) -> Self::Output;
}

pub trait RegWrite<D = u8>: RegRead<D> {
    fn data(&self) -> D;
}

macro_rules! register {
  (@impl_reg_read $ty:ident, $addr:literal, $output:ident) => {
    impl RegRead for $ty {
      type Output = $output;

      const ADDR: u8 = $addr;

      fn from_data(data: u8) -> Self::Output {
        Self::Output::from_bits_truncate(data)
      }
    }
  };
  (@impl_reg_write $ty:ident, $addr:literal, $output:ident) => {
    register!(@impl_reg_read $ty, $addr, Self);

    impl RegWrite for $ty {
      fn data(&self) -> u8 {
        self.bits()
      }
    }
  };
  (
    #[doc = $name:expr]
    $(#[$meta:meta])*
    $vis:vis type $ty:ident: $addr:literal = $ty2:ident;
  ) => {
    #[doc = concat!($name, " register (`", stringify!($addr), "`)")]
    $(#[$meta])*
    $vis enum $ty {}

    register!(@impl_reg_read $ty, $addr, $ty2);
  };
  (
    #[doc = $name:expr]
    $(#[$meta:meta])*
    $vis:vis struct $ty:ident: $addr:literal {
      $(const $bit_name:ident = $bit_val:expr;)*
  }
  ) => {
    ::bitflags::bitflags! {
      #[doc = concat!($name, " register (`", stringify!($addr), "`)")]
      $(#[$meta])*
      $vis struct $ty: u8 {
        $(const $bit_name = $bit_val;)*
      }
    }

    register!(@impl_reg_write $ty, $addr, Self);
  };
}

register! {
  /// STATUS_REG_AUX_A
  #[derive(Default)]
  pub struct StatusRegAuxA: 0x07 {
    const TOR = 0b01000000;
    const TDA = 0b00000100;
  }
}
