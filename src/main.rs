//#![feature(scoped)]

extern crate rustc_serialize;
extern crate websocket;
extern crate hyper;

pub mod blockchain_info;
pub mod payment_detection;
pub mod server;

use hyper::server::Server;

use server::detect_payment;


fn main() {
    let server = Server::http("127.0.0.1:1337").unwrap();
    let _guard = server.handle(detect_payment);
    println!("Listening on http://127.0.0.1:1337");
}
