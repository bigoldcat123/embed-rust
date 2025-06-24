use defmt::info;
use embassy_stm32::{
    Peripheral,
    gpio::{Level, Output, Pin, Speed},
    mode::Async,
    spi::{Error, Spi, Word},
};
use embassy_time::Timer;

use crate::display_dirver::st7789_cmd;


pub struct St7789<'a> {
    spi: Spi<'a, Async>,
    delay_ms: u64,
    cs: Output<'a>,
    dc: Output<'a>,
    is_initiated:bool
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
            delay_ms: 1,
            cs,
            dc,
            is_initiated:false
        }
    }
    pub async fn init(&mut self) -> Result<(), Error> {
        self.is_initiated = true;
        Timer::after_millis(self.delay_ms).await;
        self.cs.set_low();
        Timer::after_millis(self.delay_ms).await;
        self.write_command(&[st7789_cmd::RESET]).await?;

        self.write_command(&[st7789_cmd::SLEEP_OUT, st7789_cmd::DISPLAY_ON,st7789_cmd::DISPLAY_INVERSION_ON]).await?;

        self.write_command(&[st7789_cmd::COL_MODE]).await?;
        self.write_data(&[0x55_u8]).await?;

        Ok(())
    }
    /// 0..=319
    pub async fn set_row(&mut self, start: u16, end: u16) -> Result<(), Error> {
        let start_hight = (start >> 8) as u8;
        let start_low = (start & 0x00ff) as u8;
        let end_hight = (end >> 8) as u8;
        let end_low = (end & 0x00ff) as u8;
        info!("{:?}",&[start_hight, start_low, end_hight, end_low]);

        self.write_command(&[st7789_cmd::ROW_ADDRESS_SET]).await?;
        self.write_data(&[start_hight, start_low, end_hight, end_low])
            .await?;
        Ok(())
    }
    /// 0..=239
    pub async fn set_col(&mut self, start: u16, end: u16) -> Result<(), Error> {
        let start_hight = (start >> 8) as u8;
        let start_low = (start & 0x00ff) as u8;
        let end_hight = (end >> 8) as u8;
        let end_low = (end & 0x00ff) as u8;
        info!("{:?}",&[start_hight, start_low, end_hight, end_low]);
        self.write_command(&[st7789_cmd::COLUMN_ADDRESS_SET])
            .await?;
        self.write_data(&[start_hight, start_low, end_hight, end_low])
            .await?;
        Ok(())
    }
    pub async fn write_memory(&mut self) -> Result<(), Error> {
        self.write_command(&[st7789_cmd::MEMORY_WRITE]).await?;
        // self.write_data(data).await?;
        Ok(())
    }
    pub async fn write_data<W: Word>(&mut self, data: &[W]) -> Result<(), Error> {
        if !self.is_initiated {
            panic!("init first!");
        }
        self.dc.set_high();
        Timer::after_millis(self.delay_ms).await;
        self.spi.write(data).await?;
        Ok(())
    }
    async fn write_command<W: Word>(&mut self, data: &[W]) -> Result<(), Error> {
        if !self.is_initiated {
            panic!("init first!");
        }
        self.dc.set_low();
        Timer::after_millis(self.delay_ms).await;
        self.spi.write(data).await?;
        Ok(())
    }
}
