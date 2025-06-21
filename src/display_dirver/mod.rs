
pub mod st7789;
#[repr(u8)]
enum ST7789Cmd {
    Reset = 0x01 as u8,
    SleepIn = 0x10,
    SleepOut = 0x11,
    PartialDisplayMode = 0x12,
    NormalDisplayMode = 0x13,
    DsiplayInversionOff = 0x20,
    DisplayInversionOn = 0x21,

    /// one parameter needed
    /// ### availibale papameter:
    /// 0x01, 0x02, 0x04, 0x08 G2.2, G1.8,G2.5, G1.0
    GamaSet = 0x26,
    DisplayOff = 0x28,
    DisplayOn = 0x29,
    /// four paramater needed
    /// ### availibale papameter: 
    /// start_hight,start_low, end_high,end_low.
    /// ### example:
    /// 0x00, 0x00, 0x00,0xef -> from 0x0000 to 0x00ef
    ColumnAddressSet = 0x2a,
    /// four paramater needed
    /// ### availibale papameter: 
    /// start_hight,start_low, end_high,end_low.
    /// ### example:
    /// 0x00, 0x00, 0x00,0xef -> from 0x0000 to 0x00ef
    RowAddressSet = 0x2B,
    /// write Data
    MemoryWrite = 0x2C,
    /// # Interface Pixel Format
    /// 
    /// one param needed
    /// 
    /// 0x55 (RGB565)or 0x66(RGB(666))
    ColMode = 0x3A,
}


