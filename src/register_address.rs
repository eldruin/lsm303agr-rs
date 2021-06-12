pub struct Register;
impl Register {
    pub const WHO_AM_I_A: u8 = 0x0F;
    pub const CTRL_REG1_A: u8 = 0x20;
    pub const CTRL_REG4_A: u8 = 0x23;
    pub const STATUS_REG_A: u8 = 0x27;
    pub const OUT_X_L_A: u8 = 0x28;
    pub const WHO_AM_I_M: u8 = 0x4F;
    pub const CFG_REG_A_M: u8 = 0x60;
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

    pub const XDR: u8 = 1;
    pub const YDR: u8 = 1 << 1;
    pub const ZDR: u8 = 1 << 2;
    pub const XYZDR: u8 = 1 << 3;
    pub const XOR: u8 = 1 << 4;
    pub const YOR: u8 = 1 << 5;
    pub const ZOR: u8 = 1 << 6;
    pub const XYZOR: u8 = 1 << 7;
}
