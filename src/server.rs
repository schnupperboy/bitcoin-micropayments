#![deny(warnings)]
extern crate hyper;

use std::io::Read;

use rustc_serialize::*;

use hyper::{Post};
use hyper::server::{Request, Response};
use hyper::uri::RequestUri::AbsolutePath;
use hyper::header::{ContentType};
use hyper::mime::{Mime, TopLevel, SubLevel, Attr, Value};

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

pub fn gen_resp_json(error: i64) -> Vec<u8> {
	let payment_response = PaymentResponse::new(&error);
	let payment_response_json = json::encode(&payment_response).unwrap();

	payment_response_json.as_bytes().to_vec()
}

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

	res.headers_mut().set(
		ContentType(
			Mime(
				TopLevel::Application,
				SubLevel::Json,
				vec![(Attr::Charset, Value::Utf8)]
			)
		)
	);

	*res.status_mut() = hyper::Ok;

	let mut body = String::new();
	let _ = req.read_to_string(&mut body);
	println!("Request body: {:?}", body);

	let payment_request: PaymentRequest = match json::decode(&body) {
		Ok(pr) => pr,
		Err(e) => {
			try_return!(res.send(&gen_resp_json(-1)));
			panic!("JSON Decoder: {}", e)
		}
	};

	let blockchain_info = BlockchainInfo::new(&payment_request.address, payment_request.amount);

	match blockchain_info.wait() {
		Ok(_) => {
			println!("Payment received!");
			try_return!(res.send(&gen_resp_json(0)));
		}
		Err(e) => {
			println!("Payment not received in time {:?}", e);
			try_return!(res.send(&gen_resp_json(-1)));
		}
	};
}


#[derive(RustcDecodable)]
pub struct PaymentRequest {
	pub address: String,
	pub amount: u64,
}

#[derive(RustcEncodable)]
pub struct PaymentResponse<'a> {
	pub error: &'a i64
}

impl<'a> PaymentResponse<'a> {
	pub fn new(error: &'a i64) -> PaymentResponse {
		PaymentResponse {
			error: error
		}
	}
}


