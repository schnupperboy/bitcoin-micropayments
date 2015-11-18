#![feature(plugin)]
#![plugin(regex_macros)]

extern crate rustc_serialize;
extern crate websocket;
extern crate hyper;
extern crate image;
extern crate qrcode;
extern crate regex;

pub mod blockchain_info;
pub mod payment_detection;
pub mod server;
pub mod qr;

use std::env;
use std::process;

use hyper::server::Server;

use server::{routes};

fn main() {
	if env::args().count() != 2 {
		println!("Usage: cr-payment-bitcoin <ip-address> <port>");
		process::exit(1);
	}

	let mut ip = env::args().nth(1).unwrap();
	ip.push_str(":");
	ip.push_str(&*env::args().nth(2).unwrap());


	let server = Server::http(&*ip).unwrap();
	let _guard = server.handle(routes);

	println!("Listening on http://{}", ip);
}
