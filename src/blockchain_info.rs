use std::sync::mpsc::channel;
use std::str::from_utf8;

use rustc_serialize::*;

use websocket::Message;
use websocket::Sender as WebSocketSender;
use websocket::Receiver as WebSocketReceiver;
use websocket::message::Type;

use payment_detection::PaymentDetection;


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

enum PaymentError {
	InsufficientAmount,
	Timeout,
	InvalidJsonResponse,
	WebSocketError
}

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

	fn wait(&self) -> Result<(), &'static str> {
		use std::thread;

		use websocket::Client;
		use websocket::client::request::Url;

		let websocket_url = Url::parse(&self.websocket_url).unwrap();
		println!("Connecting to {}", websocket_url);

		let request = Client::connect(websocket_url).unwrap();

		let response = request.send().unwrap(); // Send the request and retrieve a response

		println!("Validating response...");

		response.validate().unwrap(); // Validate the response

		println!("Successfully connected");

		let (mut ws_sender, mut ws_receiver) = response.begin().split();

		let (tx, rx) = channel();

		let tx_1 = tx.clone();

		let send_loop = thread::spawn(move || {
			loop {
				let message: Message = match rx.recv() {
					Ok(m) => m,
					Err(e) => {
						println!("Send Loop: {:?}", e);
						return Err(PaymentError::InvalidJsonResponse);
					}
				};

				match message.opcode {
					Type::Close => {
						let _ = ws_sender.send_message(&message);
						// If it's a close message, just send it and then return.
						return Ok(());
					}
					_ => (),
				}

				match ws_sender.send_message(&message) {
					Ok(()) => (),
					Err(e) => {
						println!("Send Loop: {:?}", e);
						let _ = ws_sender.send_message(&Message::close());
						return Err(PaymentError::WebSocketError);
					}
				}
			}
		});

		let address = self.address.to_string();
		let amount = self.amount.clone();
		let websocket_msg = self.websocket_msg.to_string().clone();

		let receive_loop = thread::spawn(move || {
			let mut amount_payed = 0;

			for message in ws_receiver.incoming_messages() {
				let message: Message = match message {
					Ok(m) => m,
					Err(e) => {
						println!("Receive Loop: {:?}", e);
						let _ = tx.send(Message::close());
						return;
						//return Err(PaymentError::WebSocketError);
					}
				};

				match message.opcode {
					Type::Close => {
						// Got a close message, so send a close message and return
						let _ = tx.send(Message::close());
						return;
						//return Err(PaymentError::Timeout);
					}

					Type::Text => {
						let data = from_utf8(&*message.payload);

						let data = match data {
					        Ok(v) => v,
					        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
					    };

						let address_event: AddressEvent = match json::decode(&data) {
							Ok(ae) => ae,
							Err(e) => {
								println!("JSON Decoder: {}", e);
								return;
								//return Err(PaymentError::InvalidJsonResponse)
							}
						};

						let transaction: Transaction = address_event.x;

						for output in &transaction.out {
							if output.addr == address {
								println!("received {} satoshis from {}", output.value, &transaction.inputs[0].prev_out.addr);
								amount_payed = amount_payed + output.value;
							}
						}

						if amount_payed >= amount {
							println!("payment complete. exiting...");
							let _ = tx.send(Message::close());
							return;
							//return Ok(());
						} else {
							return;
							//return Err(PaymentError::InsufficientAmount);
						}
					}

					_ => {
						println!("Receive Loop: unhandled websocket message: {:?}", message);
						return;
						//return Err(PaymentError::InvalidJsonResponse);
					}
				}
			}
		});

		let _ = tx_1.send(Message::text(websocket_msg));

		println!("Waiting for child threads to exit");

		let _ = send_loop.join();
		println!("send loop finished");

		let r_result = receive_loop.join();
		println!("receive loop finished");

		match r_result {
			Ok(_) => Ok(()),
			Err(_) => Err("receive loop error {}")
		}
	}
}

