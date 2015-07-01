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
struct Transaction {
	hash: String,
	ver: u8,
	vin_sz: u8,
	vout_sz: u8,
	lock_time: String,
	size: u64,
	relayed_by: String,
	tx_index: u64,
	time: u64,
	inputs: Vec<TransactionInput>,
	out: Vec<TransactionOutput>
}

#[derive(RustcEncodable, RustcDecodable)]
struct TransactionOutput {
	value: u64,
	type_: u8,
	addr: String
}

#[derive(RustcDecodable)]
struct TransactionInput {
	prev_out: TransactionOutput
}

impl Decodable for Transaction {
	fn decode<D: Decoder>(decoder: &mut D) -> Result<Transaction, D::Error> {
		decoder.read_struct("root", 0, |decoder| {

			if decoder.read_struct_field("op", 0, |decoder| Decodable::decode(decoder)) != Some {
				return Err("Ungültiges Feld \"op\"");
			}

			decoder.read_struct_field("x", 0, |decoder| {
				Ok(Transaction{
					hash: try!(decoder.read_struct_field("hash", 0, |decoder| Decodable::decode(decoder))),
					ver: try!(decoder.read_struct_field("ver", 0, |decoder| Decodable::decode(decoder))),
					vin_sz: try!(decoder.read_struct_field("vin_sz", 0, |decoder| Decodable::decode(decoder))),
					vout_sz: try!(decoder.read_struct_field("vout_sz", 0, |decoder| Decodable::decode(decoder))),
					lock_time: try!(decoder.read_struct_field("lock_time", 0, |decoder| Decodable::decode(decoder))),
					size: try!(decoder.read_struct_field("size", 0, |decoder| Decodable::decode(decoder))),
					relayed_by: try!(decoder.read_struct_field("relayed_by", 0, |decoder| Decodable::decode(decoder))),
					tx_index: try!(decoder.read_struct_field("tx_index", 0, |decoder| Decodable::decode(decoder))),
					time: try!(decoder.read_struct_field("time", 0, |decoder| Decodable::decode(decoder))),
					inputs: try!(decoder.read_struct_field("inputs", 0, |decoder| Decodable::decode(decoder))),
					output: try!(decoder.read_struct_field("output", 0, |decoder| Decodable::decode(decoder))),
				})
			})
		})
	}
}
