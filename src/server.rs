#![deny(warnings)]
extern crate hyper;

use std::io::Read;

use rustc_serialize::*;

use hyper::{Get, Post};
use hyper::server::{Request, Response};
use hyper::uri::RequestUri::AbsolutePath;
use hyper::header::{ContentType};
use hyper::mime::{Mime, TopLevel, SubLevel, Attr, Value};

use payment_detection::PaymentDetection;
use blockchain_info::*;
use qr;


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

pub fn routes(req: Request, res: Response) {
	let handler: fn(Request, Response) = match req.uri {
		AbsolutePath(ref path) => {
			let url_path = path.split("?").next().unwrap();

			match (&req.method, url_path) {
				(&Post, "/detect_payment") => detect_payment,
				(&Get, "/qr_code") => qr_code,
				_ => not_found
			}
		},
		_ => not_found
		
	};

	handler(req, res);
}

pub fn not_found(_: Request, mut res: Response) {
	*res.status_mut() = hyper::NotFound;
	
}

pub fn detect_payment(mut req: Request, mut res: Response) {
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

pub fn qr_code(req: Request, mut res: Response) {
	res.headers_mut().set(
		ContentType(
			Mime(
				TopLevel::Image,
				SubLevel::Png,
				vec![(Attr::Charset, Value::Utf8)]
			)
		)
	);

	*res.status_mut() = hyper::Ok;

	let uri: &str = match req.uri {
		AbsolutePath(ref uri) => uri,
		_ => {
			*res.status_mut() = hyper::NotFound;
			return
		}
	};

	let re_btc_amount = regex!(r"[\?&]btc_amount=([0-9\.]+)");
	let re_btc_receiver_address = regex!(r"[\?|&]btc_receiver_address=([0-9a-zA-Z]{26,34})");

	let btc_amount_captures = re_btc_amount.captures(uri).unwrap();
	let btc_amount = btc_amount_captures.at(1).unwrap();
	
	let btc_receiver_address_captures = re_btc_receiver_address.captures(uri).unwrap();
	let btc_receiver_address = btc_receiver_address_captures.at(1).unwrap();

	let bitcoin_payment_request = format!("bitcoin:{}?amount={}", btc_receiver_address, btc_amount);
	
	let png_data = qr::create(&bitcoin_payment_request);

	try_return!(res.send(&png_data));
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
