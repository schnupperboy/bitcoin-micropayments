use std::str::from_utf8;
use std::io::Read;

use rustc_serialize::*;

use websocket::Message;
use websocket::Sender as WebSocketSender;
use websocket::Receiver as WebSocketReceiver;
use websocket::message::Type;
use websocket::Client;
use websocket::client::request::Url;
use websocket::result::WebSocketError;

use hyper::Client as HttpClient;
use hyper::header::Connection;

use payment_detection::{PaymentDetection, PaymentError};
use exchange_rates::{EuroExchangeRate, ExchangeError};

// For complete blockchain.info API see: https://blockchain.info/de/api/api_websocket

#[derive(RustcDecodable)]
pub struct AddressEvent {
	pub x: Transaction,
}

#[derive(RustcDecodable)]
pub struct Transaction {
	pub inputs: Vec<TransactionInput>,
	pub out: Vec<TransactionOutput>
}

#[derive(RustcDecodable)]
pub struct TransactionOutput {
	pub value: u64,
	pub addr: String,
}

#[derive(RustcDecodable)]
pub struct TransactionInput {
	pub prev_out: TransactionOutput,
}

#[derive(RustcEncodable)]
pub struct AddressSubscription<'a> {
	op: &'a str,
	addr: &'a str
}

const ADDRESS_SUBCRIPTION_OP: &'static str = "addr_sub";

#[derive(RustcDecodable)]
struct ExchangeRates {
	EUR: ExchangeRate,
}

#[derive(RustcDecodable)]
struct ExchangeRate {
	last: f64,
}

pub struct BlockchainInfo;

const WEBSOCKET_URL: &'static str = "wss://ws.blockchain.info/inv";
const EXCHANGE_RATE_URL: &'static str = "https://blockchain.info/de/ticker";

impl<'a> AddressSubscription<'a> {
	pub fn new(address: &str) -> AddressSubscription {
		AddressSubscription {
			op: ADDRESS_SUBCRIPTION_OP,
			addr: address
		}
	}
}

impl PaymentDetection for BlockchainInfo {
	fn await_payment(address: &str, amount: u64) -> Result<(), PaymentError> {
		let target_amount = amount;

		let address_subscription = AddressSubscription::new(address);
		let json_request = json::encode(&address_subscription).unwrap();

		let websocket_url = Url::parse(WEBSOCKET_URL).unwrap();
		println!("Connecting to {}", WEBSOCKET_URL);

		let request = Client::connect(websocket_url).unwrap();

		let response = request.send().unwrap(); // Send the request and retrieve a response

		let mut client = response.begin(); // Get a Client

		println!("Successfully connected");

		client.send_message(&Message::text(json_request.to_string())).unwrap(); // Send message

		let mut amount_sum = 0;
		for message in client.incoming_messages() {
			match handle_msg(address, &message) {
				Ok(incoming_amount) => {
					amount_sum = amount_sum + incoming_amount;
					if amount_sum >= target_amount {
						println!("PAYMENT COMPLETE");
						break;
					}
				}

				Err(e) => match e {
					PaymentError::Timeout => {
						if amount_sum > 0 {
							return Err(PaymentError::InsufficientAmount);
						} else {
						    return Err(PaymentError::Timeout);
						}
					}
					_ => return Err(e)
				}
			}
		}

		Ok(())
	}
}

fn handle_msg(address: &str, message: &Result<Message, WebSocketError>) -> Result<u64, PaymentError> {
    let message: &Message = match *message {
    	Ok(ref m) => m,
    	Err(ref e) => match e {
    		&WebSocketError::NoDataAvailable => return Err(PaymentError::Timeout),
    		_ => return Err(PaymentError::BackendError)
    	}
    };

	match message.opcode {
		Type::Text => {
			let data = match from_utf8(&message.payload) {
		        Ok(v) => v,
		        Err(e) => {
		        	println!("Backend error (UTF-8 decoding): {:?}", e);
		        	return Err(PaymentError::BackendError);
		        }
		    };

			let address_event: AddressEvent = match json::decode(&data) {
				Ok(ae) => ae,
				Err(e) => {
					println!("Backend error (JSON decoding): {:?}", e);
					return Err(PaymentError::BackendError);
				}
			};

			let transaction: Transaction = address_event.x;

			let mut amount_payed = 0;
			for output in &transaction.out {
				if output.addr == address.to_string() {
					println!("received {} satoshis from {}", output.value, &transaction.inputs[0].prev_out.addr);
					amount_payed = amount_payed + output.value;
				}
			}

			Ok(amount_payed)
		}

		Type::Close => {
			Err(PaymentError::Timeout)
		}

		_ => {
			println!("Backend error (unhandled websocket message): {:?}", message);
			Err(PaymentError::BackendError)
		}
	}
}

impl EuroExchangeRate for BlockchainInfo {
	fn convert(euro: f64) -> Result<f64, ExchangeError> {
	    // Create a client.
	    let client = HttpClient::new();

	    // Creating an outgoing request.
	    let mut res = client.get(EXCHANGE_RATE_URL)
	        // set a header
	        .header(Connection::close())
	        // let 'er go!
	        .send().unwrap();

	    // Read the Response.
	    let mut body = String::new();
	    res.read_to_string(&mut body).unwrap();

		let exchange_rates: ExchangeRates = match json::decode(&body) {
			Ok(er) => er,
			Err(e) => {
				println!("Backend error (JSON decoding): {:?}", e);
				return Err(ExchangeError::BackendError);
			}
		};

		Ok(euro/exchange_rates.EUR.last)
	}
}