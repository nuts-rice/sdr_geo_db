use crate::source::Source;
use soapysdr::RxStream;
pub struct HackRFSource {
    rx_stream: RxStream<f32>,
}

impl Source for HackRFSource {}

impl HackRFSource {}
