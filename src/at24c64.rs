use embedded_hal_async::i2c::ErrorType;

/// driver for at24c64
///  default addr is 0x50;
/// 
/// 
///  A0 +----+ Vcc
/// 
/// 
///  A1 |------| WP  
/// 
/// 
///  A2 |------| SCL  
/// 
/// 
/// GND +------+ SDA  
pub struct At24c64Gen<T: embedded_hal_async::i2c::I2c> {
    slave_address: u8,
    i2c: T,
    buf: [u8; 128],
}
impl<T: embedded_hal_async::i2c::I2c> At24c64Gen<T> {
    pub fn new(i2c: T, slave_address: u8) -> Self {
        // use embedded_hal_async::i2c::I2c;
        Self {
            slave_address,
            i2c,
            buf: [0; 128],
        }
    }
    pub async fn write(
        &mut self,
        adr_high: u8,
        adr_low: u8,
        data: &[u8],
    ) -> Result<(), <T as ErrorType>::Error> {
        self.buf[0] = adr_high;
        self.buf[1] = adr_low;
        for i in 0..data.len() {
            self.buf[2 + i] = data[i];
        }
        self.i2c
            .write(self.slave_address, &self.buf[..2 + data.len()])
            .await
    }

    pub async fn read(
        &mut self,
        adr_high: u8,
        adr_low: u8,
        data: &mut [u8],
    ) -> Result<(), <T as ErrorType>::Error> {
        self.write(adr_high, adr_low, &[]).await?;

        self.i2c.read(self.slave_address, data).await
    }
}
