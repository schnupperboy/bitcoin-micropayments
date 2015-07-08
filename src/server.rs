#![deny(warnings)]
extern crate hyper;

use std::io::Read;

use rustc_serialize::*;

use hyper::{Post};
use hyper::server::{Request, Response};
use hyper::uri::RequestUri::AbsolutePath;

use payment_detection::PaymentDetection;
use blockchain_info::*;


macro_rules! try_return(
	($e:expr) => {{
		match $e {
			Ok(v) => v,
			Err(e) => { println!("Error: {}", e); return; }
		}
	}}
);

pub fn detect_payment(mut req: Request, mut res: Response) {
	match req.uri {
		AbsolutePath(ref path) => match (&req.method, &path[..]) {
			(&Post, "/detect_payment") => (), // fall through, fighting mutable borrows
			_ => {
				*res.status_mut() = hyper::NotFound;
				return;
			}
		},
		_ => {
			return;
		}
	};

	let mut body = String::new();
	let _ = req.read_to_string(&mut body);
	println!("Request body: {:?}", body);

	let payment_request: PaymentRequest = match json::decode(&body) {
		Ok(pr) => pr,
		Err(e) => {
			println!("JSON Decoder: {}", e);
			return
		}
	};

	let blockchain_info = BlockchainInfo::new(&payment_request.address, payment_request.amount);
	let _ = blockchain_info.wait();

	try_return!(res.send(b"Payment received"));
}


#[derive(RustcDecodable)]
pub struct PaymentRequest {
	pub address: String,
	pub amount: u64,
}


