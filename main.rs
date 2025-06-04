use std::io::{Write, stdin, stdout};

use tokio::sync::mpsc::channel;
use usb_driver::UsbReader;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let device = nusb::list_devices()
        .unwrap()
        .find(|x| x.vendor_id() == 0x1234)
        .unwrap();
    println!("{:#?}", device);
    let device = device.open().unwrap();
    println!("{:#?}", device.active_configuration());
    device.detach_and_claim_interface(0x01).unwrap();
    let interface = device.claim_interface(0x01).unwrap();
    let mut out = stdout();
    let in_ = stdin();
    let interface_in = interface.clone();
    let (tx, mut rx) = channel(8);
    let reader = UsbReader::new(interface_in, tx);
    tokio::spawn(async { reader.run().await });
    loop {
        let mut cmd = String::new();

        out.write_all(b">").unwrap();
        out.flush().unwrap();
        in_.read_line(&mut cmd).unwrap();
        println!("{:?}",cmd.as_bytes());
        interface
            .bulk_out(0x02, cmd.into_bytes())
            .await
            .into_result()
            .unwrap();
        let _ = rx.recv().await.unwrap();
    }
}
