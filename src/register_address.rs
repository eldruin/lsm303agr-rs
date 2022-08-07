use crate::types::{
    AccelOutputDataRate, AccelScale, AccelerometerId, FifoMode, MagOutputDataRate, MagnetometerId,
    StatusFlags,
};

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

register! {
  /// WHO_AM_I_A
  pub type WhoAmIA: 0x0F = AccelerometerId;
}

impl WhoAmIA {
    pub(crate) const ID: u8 = 0b00110011;
}

register! {
  /// TEMP_CFG_REG_A
  #[derive(Default)]
  pub struct TempCfgRegA: 0x1F {
    const TEMP_EN1 = 0b10000000;
    const TEMP_EN0 = 0b01000000;

    const TEMP_EN = Self::TEMP_EN1.bits | Self::TEMP_EN0.bits;
  }
}

register! {
  /// CTRL_REG1_A
  pub struct CtrlReg1A: 0x20 {
    const ODR3 = 0b10000000;
    const ODR2 = 0b01000000;
    const ODR1 = 0b00100000;
    const ODR0 = 0b00010000;
    const LPEN = 0b00001000;
    const ZEN  = 0b00000100;
    const YEN  = 0b00000010;
    const XEN  = 0b00000001;

    const ODR = Self::ODR3.bits | Self::ODR2.bits | Self::ODR1.bits | Self::ODR0.bits;
  }
}

impl Default for CtrlReg1A {
    fn default() -> Self {
        Self::ZEN | Self::YEN | Self::XEN
    }
}

impl CtrlReg1A {
    pub const fn with_odr(self, odr: AccelOutputDataRate) -> Self {
        let reg = self.difference(Self::ODR);

        match odr {
            AccelOutputDataRate::Hz1 => reg.union(Self::ODR0),
            AccelOutputDataRate::Hz10 => reg.union(Self::ODR1),
            AccelOutputDataRate::Hz25 => reg.union(Self::ODR1).union(Self::ODR0),
            AccelOutputDataRate::Hz50 => reg.union(Self::ODR2),
            AccelOutputDataRate::Hz100 => reg.union(Self::ODR2).union(Self::ODR0),
            AccelOutputDataRate::Hz200 => reg.union(Self::ODR2).union(Self::ODR1),
            AccelOutputDataRate::Hz400 => reg.union(Self::ODR2).union(Self::ODR1).union(Self::ODR0),
            AccelOutputDataRate::Khz1_344 => reg
                .union(Self::ODR3)
                .union(Self::ODR0)
                .difference(Self::LPEN),
            AccelOutputDataRate::Khz1_620LowPower => reg.union(Self::ODR3).union(Self::LPEN),
            AccelOutputDataRate::Khz5_376LowPower => {
                reg.union(Self::ODR3).union(Self::ODR0).union(Self::LPEN)
            }
        }
    }
}

register! {
  /// CTRL_REG2_A
  pub struct CtrlReg2A: 0x21 {
    const HPM1    = 0b10000000;
    const HPM0    = 0b01000000;
    const HPCF2   = 0b00100000;
    const HPCF1   = 0b00010000;
    const FDS     = 0b00001000;
    const HPCLICK = 0b00000100;
    const HPIS2   = 0b00000010;
    const HPIS1   = 0b00000001;
  }
}

register! {
  /// CTRL_REG3_A
  pub struct CtrlReg3A: 0x22 {
    const I1_CLICK   = 0b10000000;
    const I1_AOI1    = 0b01000000;
    const I1_AOI2    = 0b00100000;
    const I1_DRDY1   = 0b00010000;
    const I1_DRDY2   = 0b00001000;
    const I1_WTM     = 0b00000100;
    const I1_OVERRUN = 0b00000010;
  }
}

register! {
  /// CTRL_REG4_A
  #[derive(Default)]
  pub struct CtrlReg4A: 0x23 {
    const BDU        = 0b10000000;
    const BLE        = 0b01000000;
    const FS1        = 0b00100000;
    const FS0        = 0b00010000;
    const HR         = 0b00001000;
    const ST1        = 0b00000100;
    const ST0        = 0b00000010;
    const SPI_ENABLE = 0b00000001;

    const FS = Self::FS1.bits | Self::FS0.bits;
    const ST = Self::ST1.bits | Self::ST0.bits;
  }
}

impl CtrlReg4A {
    pub const fn scale(&self) -> AccelScale {
        match self.intersection(Self::FS).bits() >> 4 {
            0b00 => AccelScale::G2,
            0b01 => AccelScale::G4,
            0b10 => AccelScale::G8,
            _ => AccelScale::G16,
        }
    }

    pub const fn with_scale(self, scale: AccelScale) -> Self {
        match scale {
            AccelScale::G2 => self.difference(Self::FS),
            AccelScale::G4 => self.difference(Self::FS1).union(Self::FS0),
            AccelScale::G8 => self.union(Self::FS1).difference(Self::FS0),
            AccelScale::G16 => self.union(Self::FS),
        }
    }
}

register! {
  /// CTRL_REG5_A
  #[derive(Default)]
  pub struct CtrlReg5A: 0x24 {
    const BOOT     = 0b10000000;
    const FIFO_EN  = 0b01000000;
    const LIR_INT1 = 0b00001000;
    const D4D_INT1 = 0b00000100;
    const LIR_INT2 = 0b00000010;
    const D4D_INT2 = 0b00000001;
  }
}

register! {
  /// CTRL_REG6_A
  pub struct CtrlReg6A: 0x25 {
    const I2_CLICK_EN = 0b10000000;
    const I2_INT1     = 0b01000000;
    const I2_INT2     = 0b00100000;
    const BOOT_I2     = 0b00010000;
    const P2_ACT      = 0b00001000;
    const H_LACTIVE   = 0b00000010;
  }
}

register! {
  /// STATUS_REG_A
  pub type StatusRegA: 0x27 = StatusFlags;
}

register! {
  /// FIFO_CTRL_REG_A
  #[derive(Default)]
  pub struct FifoCtrlRegA: 0x2E {
    const FM1  = 0b10000000;
    const FM0  = 0b01000000;
    const TR   = 0b00100000;
    const FTH4 = 0b00010000;
    const FTH3 = 0b00001000;
    const FTH2 = 0b00000100;
    const FTH1 = 0b00000010;
    const FTH0 = 0b00000001;

    const FM = Self::FM1.bits | Self::FM0.bits;
    const FTH = Self::FTH4.bits | Self::FTH3.bits | Self::FTH2.bits | Self::FTH1.bits | Self::FTH0.bits;
  }
}

impl FifoCtrlRegA {
    pub const fn with_mode(self, mode: FifoMode) -> Self {
        match mode {
            FifoMode::Bypass => self.difference(Self::FM),
            FifoMode::Fifo => self.difference(Self::FM1).union(Self::FM0),
            FifoMode::Stream => self.union(Self::FM1).difference(Self::FM0),
            FifoMode::StreamToFifo => self.union(Self::FM),
        }
    }

    pub const fn with_full_threshold(self, n: u8) -> Self {
        let n = if n > Self::FTH.bits {
            Self::FTH.bits
        } else {
            n
        };
        self.difference(Self::FTH)
            .union(Self::from_bits_truncate(n))
    }
}

register! {
  /// FIFO_SRC_REG_A
  pub struct FifoSrcRegA: 0x2F {
    const WTM       = 0b10000000;
    const OVRN_FIFO = 0b01000000;
    const EMPTY     = 0b00100000;
    const FSS4      = 0b00010000;
    const FSS3      = 0b00001000;
    const FSS2      = 0b00000100;
    const FSS1      = 0b00000010;
    const FSS0      = 0b00000001;
  }
}

register! {
  /// INT1_CFG_A
  pub struct Int1CfgA: 0x30 {
    const AOI       = 0b10000000;
    const D6        = 0b01000000;
    const ZHIE      = 0b00100000;
    const ZUPE      = Self::ZHIE.bits;
    const ZLIE      = 0b00010000;
    const ZDOWNE    = Self::ZLIE.bits;
    const YHIE      = 0b00001000;
    const YUPE      = Self::YHIE.bits;
    const YLIE      = 0b00000100;
    const YDOWNE    = Self::YLIE.bits;
    const XHIE      = 0b00000010;
    const XUPE      = Self::XHIE.bits;
    const XLIE      = 0b00000001;
    const XDOWNE    = Self::XLIE.bits;
  }
}

register! {
  /// INT1_SRC_A
  pub struct Int1SrcA: 0x31 {
    const IA = 0b01000000;
    const ZH = 0b00100000;
    const ZL = 0b00010000;
    const YH = 0b00001000;
    const YL = 0b00000100;
    const XH = 0b00000010;
    const XL = 0b00000001;
  }
}

register! {
  /// WHO_AM_I_A_M
  pub type WhoAmIM: 0x4F = MagnetometerId;
}

impl WhoAmIM {
    pub(crate) const ID: u8 = 0b01000000;
}

register! {
  /// CFG_REG_A_M
  pub struct CfgRegAM: 0x60 {
    const COMP_TEMP_EN = 0b10000000;
    const REBOOT       = 0b01000000;
    const SOFT_RST     = 0b00100000;
    const LP           = 0b00010000;
    const ODR1         = 0b00001000;
    const ODR0         = 0b00000100;
    const MD1          = 0b00000010;
    const MD0          = 0b00000001;

    const ODR = Self::ODR1.bits | Self::ODR0.bits;
    const MD = Self::MD1.bits | Self::MD0.bits;
  }
}

impl Default for CfgRegAM {
    fn default() -> Self {
        Self::MD1 | Self::MD0
    }
}

impl CfgRegAM {
    pub const fn continuous_mode(self) -> Self {
        self.difference(Self::MD1).difference(Self::MD0) // 0b00
    }

    pub const fn is_single_mode(&self) -> bool {
        !self.contains(CfgRegAM::MD1) && self.contains(CfgRegAM::MD0)
    }

    pub const fn single_mode(self) -> Self {
        self.difference(CfgRegAM::MD1).union(CfgRegAM::MD0) // 0b01
    }

    #[cfg(test)]
    pub const fn is_idle_mode(&self) -> bool {
        self.contains(CfgRegAM::MD1) // 0b10 or 0b11
    }

    pub const fn idle_mode(self) -> Self {
        self.union(Self::MD1).union(Self::MD0) // 0b11
    }

    pub const fn with_odr(self, odr: MagOutputDataRate) -> Self {
        match odr {
            MagOutputDataRate::Hz10 => self.difference(Self::ODR1).difference(Self::ODR0), // 0b00
            MagOutputDataRate::Hz20 => self.difference(Self::ODR1).union(Self::ODR0),      // 0b01
            MagOutputDataRate::Hz50 => self.union(Self::ODR1).difference(Self::ODR0),      // 0b10
            MagOutputDataRate::Hz100 => self.union(Self::ODR1).union(Self::ODR0),          // 0b11
        }
    }
}

register! {
  /// CFG_REG_B_M
  #[derive(Default)]
  pub struct CfgRegBM: 0x61 {
    const OFF_CANC_ONE_SHOT = 0b00010000;
    const INT_ON_DATA_OFF   = 0b00001000;
    const SET_FREQ          = 0b00000100;
    const OFF_CANC          = 0b00000010;
    const LPF               = 0b00000001;
  }
}

register! {
  /// CFG_REG_C_M
  #[derive(Default)]
  pub struct CfgRegCM: 0x62 {
    const INT_MAG_PIN = 0b01000000;
    const I2C_DIS     = 0b00100000;
    const BDU         = 0b00010000;
    const BLE         = 0b00001000;
    const SELF_TEST   = 0b00000010;
    const INT_MAG     = 0b00000001;
  }
}

register! {
  /// STATUS_REG_M
  pub type StatusRegM: 0x67 = StatusFlags;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ctrl_reg_1_a() {
        let ctrl = CtrlReg1A::default();

        assert!(ctrl.contains(CtrlReg1A::XEN), "X-axis enabled by default.");
        assert!(ctrl.contains(CtrlReg1A::YEN), "Y-axis enabled by default.");
        assert!(ctrl.contains(CtrlReg1A::ZEN), "Z-axis enabled by default.");

        let check_odr = |odr, value| {
            assert_eq!(
                ctrl.with_odr(odr).intersection(CtrlReg1A::ODR).bits() >> 4,
                value
            );
        };

        check_odr(AccelOutputDataRate::Hz1, 0b0001);
        check_odr(AccelOutputDataRate::Hz10, 0b0010);
        check_odr(AccelOutputDataRate::Hz25, 0b0011);
        check_odr(AccelOutputDataRate::Hz50, 0b0100);
        check_odr(AccelOutputDataRate::Hz100, 0b0101);
        check_odr(AccelOutputDataRate::Hz200, 0b0110);
        check_odr(AccelOutputDataRate::Hz400, 0b0111);
    }

    #[test]
    fn ctrl_reg_4_a() {
        let ctrl = CtrlReg4A::default();
        assert_eq!(ctrl.bits(), 0);

        assert_eq!(ctrl.scale(), AccelScale::G2);

        let ctrl_g4 = ctrl.with_scale(AccelScale::G4);
        assert_eq!(ctrl_g4.scale(), AccelScale::G4);
        let ctrl_g8 = ctrl_g4.with_scale(AccelScale::G8);
        assert_eq!(ctrl_g8.scale(), AccelScale::G8);
        let ctrl_g16 = ctrl_g8.with_scale(AccelScale::G16);
        assert_eq!(ctrl_g16.scale(), AccelScale::G16);
        let ctrl_g2 = ctrl_g16.with_scale(AccelScale::G2);
        assert_eq!(ctrl_g2.scale(), AccelScale::G2);
    }

    #[test]
    fn cfg_reg_a_m() {
        let cfg = CfgRegAM::default();
        assert!(cfg.is_idle_mode(), "Idle mode is default.");

        let cfg = cfg.single_mode();
        assert!(cfg.is_single_mode());

        let check_odr = |odr, value| {
            assert_eq!(
                cfg.with_odr(odr).intersection(CfgRegAM::ODR).bits() >> 2,
                value
            );
        };

        check_odr(MagOutputDataRate::Hz10, 0b00);
        check_odr(MagOutputDataRate::Hz20, 0b01);
        check_odr(MagOutputDataRate::Hz50, 0b10);
        check_odr(MagOutputDataRate::Hz100, 0b11);

        let cfg = cfg.continuous_mode();
        assert!(!cfg.is_single_mode());
        assert!(!cfg.is_idle_mode());
    }
}
