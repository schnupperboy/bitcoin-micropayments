#![deny(warnings)]
extern crate hyper;

use hyper::Get;
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

pub fn routes(req: Request, res: Response) {
	let handler: fn(Request, Response) = match req.uri {
		AbsolutePath(ref path) => {
			let url_path = path.split("?").next().unwrap();

			match (&req.method, url_path) {
				(&Get, "/detect_payment") => detect_payment,
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

pub fn detect_payment(req: Request, mut res: Response) {
	res.headers_mut().set(
		ContentType(
			Mime(
				TopLevel::Text,
				SubLevel::Plain,
				vec![(Attr::Charset, Value::Utf8)]
			)
		)
	);

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
	let btc_amount_str = btc_amount_captures.at(1).unwrap();
	
	let btc_receiver_address_captures = re_btc_receiver_address.captures(uri).unwrap();
	let btc_receiver_address = btc_receiver_address_captures.at(1).unwrap();

	let btc_amount = btc_amount_str.parse::<f64>().unwrap();
	let satoshi_amount = btc_amount * 1000.0 * 1000.0 * 100.0;
	let blockchain_info = BlockchainInfo::new(btc_receiver_address, satoshi_amount as u64);

	*res.status_mut() = hyper::Ok;

	match blockchain_info.wait() {
		Ok(_) => {
			try_return!(res.send(&"Ok".as_bytes().to_vec()));
		}
		Err(e) => {
			try_return!(res.send(&format!("{:?}", e).as_bytes().to_vec()));
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

	*res.status_mut() = hyper::Ok;

	try_return!(res.send(&png_data));
}
