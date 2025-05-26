use embedded_hal_async::i2c::I2c;

static ADDR: u8 = 0x3C; // SSD1306 通常是 0x3C 或 0x3D
static CONTROL_CMD: u8 = 0x00; // 控制字节：写命令
static CONTROL_DATA: u8 = 0x40; // 控制字节：写数据
pub struct Ssd1315<T: I2c> {
    i2c: T,
    display_cache: [u8; 128 * 8 + 1], // 128 * 64 bit
}

impl<T: I2c> Ssd1315<T> {
    pub fn new(i2c: T) -> Self {
        Self {
            i2c,
            display_cache: [0; 128 * 8 + 1],
        }
    }
    pub fn add_square(&mut self,start:usize) {
        for i in start..start + 2 {
            for j in 0..64 {
                self.display_cache[i * 128 + j + 1] = 0xff
            }
        }
    }

    pub async fn clear(&mut self) {
        self.display_cache.fill(0);
        self.draw().await
    }

    pub async fn draw(&mut self) {
        self.display_cache[0] = CONTROL_DATA;
        self.i2c.write(ADDR, &self.display_cache).await.unwrap()
    }

    pub async fn init(&mut self) {
        let init_cmds = [
            CONTROL_CMD, // 控制字节，表示后面是命令
            0xAE,
            0x20,
            0x00, // horizional
            0xA1,
            0xC8,
            0x81,
            0x7F,
            0xA4,
            0xA6,
            0xD3,
            0x00,
            0xD5,
            0x80,
            0xD9,
            0xF1,
            0xDA,
            0x12,
            0xDB,
            0x40,
            0x8D,
            0x14,
            0xAF,
        ];
        self.i2c.write(ADDR, &init_cmds).await.unwrap(); // 设备地址可能是 0x3C
        let set_addr_cmds: &[u8] = &[
            CONTROL_CMD,
            0x21,
            0,
            127, // 设置列地址
            0x22,
            0,
            7, // 设置页地址
        ];
        self.i2c.write(ADDR, set_addr_cmds).await.unwrap();
    }
}
