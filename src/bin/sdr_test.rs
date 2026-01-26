use soapysdr::{Device, Direction};
use std::env;
fn main() {
    println!("Starting SDR test...");
    let filter = env::args().nth(1).unwrap_or_default();
    let devices = soapysdr::enumerate(&filter[..]).expect("Failed to enumerate devices");
    if devices.is_empty() {
        println!("No SDR devices found.");
        return;
    }

    println!("Found {} device(s):", devices.len());

    for devargs in devices {
        println!("Device args: {}", devargs);
        let device = Device::new(devargs).expect("Failed to open device");
        let channels = device
            .num_channels(Direction::Rx)
            .expect("Failed to get number of RX channels");
        println!("Number of RX channels: {}", channels);
    }
}
