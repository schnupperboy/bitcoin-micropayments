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

use server::routes;

fn main() {
	if env::args().count() != 2 {
		println!("Usage: cr-payment-bitcoin <ip-address> <port>");
		process::exit(1);
	}

	let mut sock_addr = env::args().nth(1).unwrap();
	sock_addr.push_str(":");
	sock_addr.push_str(&*env::args().nth(2).unwrap());


	let server = Server::http(&*sock_addr).unwrap();
	let _guard = server.handle(routes);

	println!("Listening on http://{}", sock_addr);
}
