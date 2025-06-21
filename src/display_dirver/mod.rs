#![allow(unused)]
pub mod st7789;
pub mod st7789_cmd {
    pub const RESET: u8 = 0x01;
    pub const SLEEP_IN: u8 = 0x10;
    pub const SLEEP_OUT: u8 = 0x11;
    pub const PARTIAL_DISPLAY_MODE: u8 = 0x12;
    pub const NORMAL_DISPLAY_MODE: u8 = 0x13;
    pub const DISPLAY_INVERSION_OFF: u8 = 0x20;
    pub const DISPLAY_INVERSION_ON: u8 = 0x21;

    /// one parameter needed
    /// ### available parameter:
    /// 0x01, 0x02, 0x04, 0x08 G2.2, G1.8, G2.5, G1.0
    pub const GAMMA_SET: u8 = 0x26;
    pub const DISPLAY_OFF: u8 = 0x28;
    pub const DISPLAY_ON: u8 = 0x29;
    /// four parameters needed
    /// ### available parameter: 
    /// start_high, start_low, end_high, end_low.
    /// ### example:
    /// 0x00, 0x00, 0x00, 0xef -> from 0x0000 to 0x00ef
    pub const COLUMN_ADDRESS_SET: u8 = 0x2A;
    /// four parameters needed
    /// ### available parameter: 
    /// start_high, start_low, end_high, end_low.
    /// ### example:
    /// 0x00, 0x00, 0x00, 0xef -> from 0x0000 to 0x00ef
    pub const ROW_ADDRESS_SET: u8 = 0x2B;
    /// write Data
    pub const MEMORY_WRITE: u8 = 0x2C;
    /// # Interface Pixel Format
    /// 
    /// one param needed
    /// 
    /// 0x55 (RGB565) or 0x66 (RGB666)
    pub const COL_MODE: u8 = 0x3A;
}


