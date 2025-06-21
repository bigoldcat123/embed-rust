use embassy_stm32::{
    Peripheral,
    gpio::{Level, Output, Pin, Speed},
    mode::Async,
    spi::{Error, Spi, Word},
};
use embassy_time::Timer;

use crate::display_dirver::ST7789Cmd;

pub struct St7789<'a> {
    spi: Spi<'a, Async>,
    delay_ms: u64,
    cs: Output<'a>,
    dc: Output<'a>,
}
impl<'a> St7789<'a> {
    pub fn new(
        spi: Spi<'a, Async>,
        cs_pin: impl Peripheral<P = impl Pin> + 'a,
        dc_pin: impl Peripheral<P = impl Pin> + 'a,
    ) -> Self {
        let cs = Output::new(cs_pin, Level::High, Speed::Medium);
        let dc = Output::new(dc_pin, Level::High, Speed::Medium);

        Self {
            spi,
            delay_ms: 50,
            cs,
            dc,
        }
    }
    pub async fn init(&mut self) -> Result<(), Error> {
        Timer::after_millis(self.delay_ms).await;
        self.cs.set_low();
        Timer::after_millis(self.delay_ms).await;
        self.write_command(&[
            ST7789Cmd::Reset as u8,
            ST7789Cmd::SleepOut as u8,
            ST7789Cmd::DisplayOn as u8,
        ])
        .await?;

        self.write_command(&[ST7789Cmd::ColMode as u8]).await?;
        self.write_data(&[0x55_u8]).await?;

        Ok(())
    }
    /// 0..=319
    pub async fn set_row(&mut self,start:u16,end:u16) -> Result<(),Error>{
        let start_hight = (start >> 8 )as u8;
        let start_low = (start & 0x0011) as u8;
        let end_hight = (end >> 8 )as u8;
        let end_low = (end & 0x0011) as u8;
        self.write_command(&[ST7789Cmd::RowAddressSet as u8]).await?;
        self.write_data(&[start_hight,start_low,end_hight,end_low]).await?;
        Ok(())
    }
    /// 0..=239
    pub async fn set_col(&mut self,start:u16,end:u16) -> Result<(),Error>{
        let start_hight = (start >> 8 )as u8;
        let start_low = (start & 0x0011) as u8;
        let end_hight = (end >> 8 )as u8;
        let end_low = (end & 0x0011) as u8;
        self.write_command(&[ST7789Cmd::ColumnAddressSet as u8]).await?;
        self.write_data(&[start_hight,start_low,end_hight,end_low]).await?;
        Ok(())
    }
    pub async fn write_memory<W:Word>(&mut self, data: &[W]) -> Result<(),Error>{
        self.write_command(&[ST7789Cmd::MemoryWrite as u8]).await?;
        self.write_data(data).await?;
        Ok(())
    }
     async fn write_data<W: Word>(&mut self, data: &[W]) -> Result<(), Error> {
        self.dc.set_high();Timer::after_millis(self.delay_ms).await;
        self.spi.write(data).await?;
        Ok(())
    }
     async fn write_command<W: Word>(&mut self, data: &[W]) -> Result<(), Error> {
        self.dc.set_low();Timer::after_millis(self.delay_ms).await;
        self.spi.write(data).await?;
        Ok(())
    }
}
