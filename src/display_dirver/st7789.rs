use defmt::info;

use embassy_time::Timer;
use embedded_hal::digital::OutputPin;
use embedded_hal_async::spi::SpiBus;

use crate::display_dirver::st7789_cmd;

pub trait Timer_ {
    fn delay(&self) -> impl Future<Output = ()>;
    fn delay_blocking(&self) {
        let mut count = 1_000;
        while count > 0 {
            core::hint::spin_loop();
            count -= 1;
        }
    }
}

pub struct St7789<SPI, OUTPUT: OutputPin, DELAY: Timer_> {
    spi: SPI,
    delay_ms: u64,
    cs: OUTPUT,
    dc: OUTPUT,
    is_initiated: bool,
    delay: DELAY,
}

impl<T: SpiBus + embedded_hal::spi::SpiBus, OUTPUT: OutputPin, DELAY: Timer_>
    St7789<T, OUTPUT, DELAY>
{
    pub fn new(spi: T, mut cs: OUTPUT, mut dc: OUTPUT, delay: DELAY) -> Self {
        cs.set_high();
        dc.set_high();
        Self {
            spi,
            delay_ms: 1,
            cs,
            dc,
            is_initiated: false,
            delay,
        }
    }
    pub async fn init(&mut self) -> Result<(), T::Error> {
        self.is_initiated = true;
        self.delay.delay().await;
        // Timer::after_millis(self.delay_ms).await;
        self.cs.set_low();
        self.delay.delay().await;
        // Timer::after_millis(self.delay_ms).await;
        self.write_command(&[st7789_cmd::RESET]).await?;

        self.write_command(&[
            st7789_cmd::SLEEP_OUT,
            st7789_cmd::DISPLAY_ON,
            st7789_cmd::DISPLAY_INVERSION_ON,
        ])
        .await?;

        self.write_command(&[st7789_cmd::COL_MODE]).await?;
        self.write_data(&[0x55_u8]).await?;

        Ok(())
    }
    /// 0..=319
    pub async fn set_row(&mut self, start: u16, end: u16) -> Result<(), T::Error> {
        let start_hight = (start >> 8) as u8;
        let start_low = (start & 0x00ff) as u8;
        let end_hight = (end >> 8) as u8;
        let end_low = (end & 0x00ff) as u8;
        info!("{:?}", &[start_hight, start_low, end_hight, end_low]);

        self.write_command(&[st7789_cmd::ROW_ADDRESS_SET]).await?;
        self.write_data(&[start_hight, start_low, end_hight, end_low])
            .await?;
        Ok(())
    }
    /// 0..=319
    pub fn set_row_blocking(&mut self, start: u16, end: u16) -> Result<(), T::Error> {
        let start_hight = (start >> 8) as u8;
        let start_low = (start & 0x00ff) as u8;
        let end_hight = (end >> 8) as u8;
        let end_low = (end & 0x00ff) as u8;
        // info!("{:?}", &[start_hight, start_low, end_hight, end_low]);

        self.write_command_blocking(&[st7789_cmd::ROW_ADDRESS_SET])?;
        self.write_data_blocking(&[start_hight, start_low, end_hight, end_low])?;
        Ok(())
    }
    /// 0..=239
    pub async fn set_col(&mut self, start: u16, end: u16) -> Result<(), T::Error> {
        let start_hight = (start >> 8) as u8;
        let start_low = (start & 0x00ff) as u8;
        let end_hight = (end >> 8) as u8;
        let end_low = (end & 0x00ff) as u8;
        info!("{:?}", &[start_hight, start_low, end_hight, end_low]);
        self.write_command(&[st7789_cmd::COLUMN_ADDRESS_SET])
            .await?;
        self.write_data(&[start_hight, start_low, end_hight, end_low])
            .await?;
        Ok(())
    }

    /// 0..=239
    pub fn set_col_blocking(&mut self, start: u16, end: u16) -> Result<(), T::Error> {
        let start_hight = (start >> 8) as u8;
        let start_low = (start & 0x00ff) as u8;
        let end_hight = (end >> 8) as u8;
        let end_low = (end & 0x00ff) as u8;
        // info!("{:?}", &[start_hight, start_low, end_hight, end_low]);
        self.write_command_blocking(&[st7789_cmd::COLUMN_ADDRESS_SET])?;
        self.write_data_blocking(&[start_hight, start_low, end_hight, end_low])?;
        Ok(())
    }

    pub async fn write_memory(&mut self) -> Result<(), T::Error> {
        self.write_command(&[st7789_cmd::MEMORY_WRITE]).await?;
        // self.write_data(data).await?;
        Ok(())
    }
    pub fn write_memory_blocking(&mut self) -> Result<(), T::Error> {
        self.write_command_blocking(&[st7789_cmd::MEMORY_WRITE])?;
        // self.write_data(data).await?;
        Ok(())
    }
    pub async fn write_data(&mut self, data: &[u8]) -> Result<(), T::Error> {
        if !self.is_initiated {
            panic!("init first!");
        }
        self.dc.set_high();
        // Timer::after_millis(self.delay_ms).await;
        embedded_hal_async::spi::SpiBus::write(&mut self.spi, data).await;
        Ok(())
    }
    pub fn write_data_blocking(&mut self, data: &[u8]) -> Result<(), T::Error> {
        if !self.is_initiated {
            panic!("init first!");
        }
        self.dc.set_high();
        // Timer::after_millis(self.delay_ms).await;
        embedded_hal::spi::SpiBus::write(&mut self.spi, data);
        Ok(())
    }
    async fn write_command(&mut self, data: &[u8]) -> Result<(), T::Error> {
        if !self.is_initiated {
            panic!("init first!");
        }
        self.dc.set_low();
        self.delay.delay().await;

        // Timer::after_millis(self.delay_ms).await;
        embedded_hal_async::spi::SpiBus::write(&mut self.spi, data).await?;
        self.delay.delay().await;
        // Timer::after_millis(self.delay_ms).await;
        Ok(())
    }
    fn write_command_blocking(&mut self, data: &[u8]) -> Result<(), T::Error> {
        if !self.is_initiated {
            panic!("init first!");
        }
        self.dc.set_low();
        self.delay.delay_blocking();

        // Timer::after_millis(self.delay_ms).await;
        embedded_hal::spi::SpiBus::write(&mut self.spi, data)?;
        self.delay.delay_blocking();
        // Timer::after_millis(self.delay_ms).await;
        Ok(())
    }
}
