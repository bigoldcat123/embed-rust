#![no_std]

use esp_hal::{
    dma::{DmaRxBuf, DmaTxBuf},
    dma_buffers,
    peripherals::{DMA_CH0, GPIO10, GPIO2, GPIO3, GPIO7, SPI2},
    spi::{
        master::{Config, Spi, SpiDmaBus},
        Mode,
    },
    time::Rate,
    Async,
};

pub fn init_spi<'d: 'static>(
    sclk: GPIO2<'d>,
    mosi: Option<GPIO3<'d>>,
    miso: Option<GPIO10<'d>>,
    cs: Option<GPIO7<'d>>,
    spi2: SPI2<'d>,
    dma_ch0: DMA_CH0<'d>,
) -> SpiDmaBus<'d, Async> {
    let (rx_buffer, rx_descriptors, tx_buffer, tx_descriptors) = dma_buffers!(32000);
    let dma_rx_buf = DmaRxBuf::new(rx_descriptors, rx_buffer).unwrap();
    let dma_tx_buf = DmaTxBuf::new(tx_descriptors, tx_buffer).unwrap();
    let cfg = Config::default();
    let mut spi = Spi::new(
        spi2,
        cfg.with_mode(Mode::_2).with_frequency(Rate::from_mhz(10)),
    )
    .unwrap()
    .with_sck(sclk);
    // .with_dma(dma_ch0)
    // .with_buffers(dma_rx_buf, dma_tx_buf);
    // .into_async();
    if let Some(mosi) = mosi {
        spi = spi.with_mosi(mosi);
    }
    if let Some(miso) = miso {
        spi = spi.with_miso(miso);
    }
    if let Some(cs) = cs {
        spi = spi.with_cs(cs);
    }
    let spi: SpiDmaBus<'_, Async> = spi
        .with_dma(dma_ch0)
        .with_buffers(dma_rx_buf, dma_tx_buf)
        .into_async();
    spi
}
