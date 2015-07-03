//#![feature(scoped)]

extern crate rustc_serialize;
extern crate websocket;

pub mod blockchain_info;
pub mod payment_detection;

use blockchain_info::*;
use payment_detection::PaymentDetection;


fn main() {
    let bitcoin_address = "1MtD4wbHnfCtmSg7VFavmfChuWeRrSe9qX";
    let bitcoin_amount = 1000; // satoshi

    let blockchain_info = BlockchainInfo::new(bitcoin_address, bitcoin_amount);
    let _ = blockchain_info.wait();
}

