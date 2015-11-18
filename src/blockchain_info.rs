use std::str::from_utf8;

use rustc_serialize::*;

use websocket::Message;
use websocket::Sender as WebSocketSender;
use websocket::Receiver as WebSocketReceiver;
use websocket::message::Type;
use websocket::Client;
use websocket::client::request::Url;
use websocket::result::WebSocketError;

use payment_detection::{PaymentDetection, PaymentError};


#[derive(RustcDecodable)]
pub struct AddressEvent {
	pub op: String,
	pub x: Transaction,
}

#[derive(RustcDecodable)]
pub struct Transaction {
	pub hash: String,
	pub ver: u8,
	pub vin_sz: u8,
	pub vout_sz: u8,
	pub lock_time: u64,
	pub size: u64,
	pub relayed_by: String,
	pub tx_index: u64,
	pub time: u64,
	pub inputs: Vec<TransactionInput>,
	pub out: Vec<TransactionOutput>
}

#[derive(RustcDecodable)]
pub struct TransactionOutput {
	pub value: u64,
	//pub type_: u8, too lazy to write a custom decoder just because "type" appears to be a rust keyword
	pub addr: String,
	pub tx_index: u64,
	pub spent: bool,
	pub n: u32,
	pub script: String,
}

#[derive(RustcDecodable)]
pub struct TransactionInput {
	pub prev_out: TransactionOutput,
	pub sequence: u64,
	pub script: String
}

#[derive(RustcEncodable)]
pub struct AddressSubscription<'a> {
	op: &'a str,
	addr: &'a str
}

const ADDRESS_SUBCRIPTION_OP: &'static str = "addr_sub";

impl<'a> AddressSubscription<'a> {
	pub fn new(address: &str) -> AddressSubscription {
		AddressSubscription {
			op: ADDRESS_SUBCRIPTION_OP,
			addr: address
		}
	}
}

pub struct BlockchainInfo<'a> {
	websocket_url: String,
	websocket_msg: String,
	address: &'a str,
	amount: u64
}

const WEBSOCKET_URL: &'static str = "wss://ws.blockchain.info/inv";


impl<'a> PaymentDetection<'a> for BlockchainInfo<'a> {
	fn new(address: &'a str, amount: u64) -> Self {
		let address_subscription = AddressSubscription::new(address);
		let json_request = json::encode(&address_subscription).unwrap();

		BlockchainInfo {
			websocket_url: WEBSOCKET_URL.to_string(),
			websocket_msg: json_request,
			address: &address,
			amount: amount
		}
	}

	fn wait(&self) -> Result<(), PaymentError> {
		let websocket_url = Url::parse(&self.websocket_url).unwrap();
		println!("Connecting to {}", websocket_url);

		let request = Client::connect(websocket_url).unwrap();

		let response = request.send().unwrap(); // Send the request and retrieve a response

		let mut client = response.begin(); // Get a Client

		println!("Successfully connected");

		client.send_message(&Message::text(self.websocket_msg.to_string())).unwrap(); // Send message

		let mut amount_sum = 0;
		for message in client.incoming_messages() {
		    let message: Message = match message {
		    	Ok(m) => m,
		    	Err(e) => match e as WebSocketError {
		    		WebSocketError::NoDataAvailable => return Err(PaymentError::Timeout),
		    		_ => return Err(PaymentError::BackendError)
		    	}
		    };

			match message.opcode {
				Type::Text => {
					match incoming_amount(&*message.payload, &self.address.to_string()) {
						Ok(amount) => {
							amount_sum = amount_sum + amount;
							if amount_sum >= self.amount {
								println!("PAYMENT COMPLETE");
								break;
							}
						},
						Err(e) => return Err(e)
					}
				}

				Type::Close => {
					if amount_sum > 0 {
						return Err(PaymentError::InsufficientAmount);
					} else {
					    return Err(PaymentError::Timeout);
					}
				}

				_ => {
					println!("Backend error (unhandled websocket message): {:?}", message);
					return Err(PaymentError::BackendError);
				}
			}
		}

		Ok(())
	}
}

fn incoming_amount(data: &[u8], address: &str) -> Result<u64, PaymentError> {
	let data = match from_utf8(data) {
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
		if output.addr == address {
			println!("received {} satoshis from {}", output.value, &transaction.inputs[0].prev_out.addr);
			amount_payed = amount_payed + output.value;
		}
	}

	Ok(amount_payed)
}

