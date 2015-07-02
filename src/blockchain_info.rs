/*{
	"op": "utx",
	"x": {
		"hash": "f6c51463ea867ce58588fec2a77e9056046657b984fd28b1482912cdadd16374",
		"ver": 1,
		"vin_sz": 4,
		"vout_sz": 2,
		"lock_time": "Unavailable",
		"size": 796,
		"relayed_by": "209.15.238.250",
		"tx_index": 3187820,
		"time": 1331300839,
		"inputs": [
			{
				"prev_out": {
					"value": 10000000,
					"type": 0,
					"addr": "12JSirdrJnQ8QWUaGZGiBPBYD19LxSPXho"
				}
			}
		],
		"out": [
			{
				"value": 2800000000,
				"type": 0,
				"addr": "1FzzMfNt46cBeS41r6WHDH1iqxSyzmxChw"
			}
		]
	}
}*/

use rustc_serialize::*;

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
