use nusb::{Interface, transfer::RequestBuffer};
use tokio::sync::mpsc::Sender;

pub struct UsbReader {
    interface: Interface,
    complete_handle: Sender<u8>,
}
impl UsbReader {
    pub fn new(interface: Interface, complete_handle: Sender<u8>) -> Self {
        Self {
            interface,
            complete_handle,
        }
    }
    pub async fn run(self) {
        loop {
            let data = self
                .interface
                .bulk_in(0x83, RequestBuffer::new(128))
                .await
                .into_result()
                .unwrap();
            println!("{:?}", String::from_utf8_lossy(data.as_ref()));
            self.complete_handle.send(1).await.unwrap();
        }
    }
}
