//#![feature(scoped)]

extern crate websocket;
extern crate rustc_serialize;

pub mod blockchain_info;

use blockchain_info::*;
use rustc_serialize::json;

fn main() {
    use std::thread;
    use std::sync::mpsc::channel;
    use std::io::stdin;

    use websocket::{Message, Sender, Receiver};
    use websocket::client::request::Url;
    use websocket::Client;

    let url = Url::parse("wss://ws.blockchain.info/inv").unwrap();

    println!("Connecting to {}", url);

    let request = Client::connect(url).unwrap();

    let response = request.send().unwrap(); // Send the request and retrieve a response

    println!("Validating response...");

    response.validate().unwrap(); // Validate the response

    println!("Successfully connected");

    let (mut sender, mut receiver) = response.begin().split();

    let (tx, rx) = channel();

    let tx_1 = tx.clone();


    let send_loop = thread::spawn(move || {
        loop {
            // Send loop
            let message = match rx.recv() {
                Ok(m) => m,
                Err(e) => {
                    println!("Send Loop: {:?}", e);
                    return;
                }
            };
            match message {
                Message::Close(_) => {
                    let _ = sender.send_message(message);
                    // If it's a close message, just send it and then return.
                    return;
                }
                _ => (),
            }
            // Send the message
            match sender.send_message(message) {
                Ok(()) => (),
                Err(e) => {
                    println!("Send Loop: {:?}", e);
                    let _ = sender.send_message(Message::Close(None));
                    return;
                }
            }
        }
    });

    let receive_loop = thread::spawn(move || {
        // Receive loop
        for message in receiver.incoming_messages() {
            let message = match message {
                Ok(m) => m,
                Err(e) => {
                    println!("Receive Loop: {:?}", e);
                    let _ = tx_1.send(Message::Close(None));
                    return;
                }
            };
            match message {
                Message::Close(_) => {
                    // Got a close message, so send a close message and return
                    let _ = tx_1.send(Message::Close(None));
                    return;
                }
                Message::Ping(data) => match tx_1.send(Message::Pong(data)) {
                    // Send a pong in response
                    Ok(()) => (),
                    Err(e) => {
                        println!("Receive Loop: {:?}", e);
                        return;
                    }
                },
                Message::Text(data) => {
                    println!("Message: {}", data);
                    let address_event: AddressEvent = match json::decode(&data) {
                        Ok(ae) => ae,
                        Err(e) => {
                            println!("JSON Decoder: {}", e);
                            return
                        }
                    };

                    let transaction: Transaction = address_event.x;
                    let transaction_output: &TransactionOutput = &transaction.out[0];
                    println!("Received satoshis: {0}", &transaction_output.value);
                }

                // Say what we received
                _ => {
                    println!("Receive Loop: unhandled websocket message: {}", message);
                    //println!("Receive Loop: {:?}", message);
                    //let address_event: AddressEvent = json::decode(message).unwrap();
                }
            }
        }
    });

    let json_msg = json::encode(&AddressSubscription::new("1MtD4wbHnfCtmSg7VFavmfChuWeRrSe9qX")).unwrap();

    // request transactions from out bitcoin address
    let message = Message::Text(json_msg);
    tx.send(message);


    // We're exiting
    
    println!("Waiting for child threads to exit");

    let _ = send_loop.join();
    let _ = receive_loop.join();
    
    println!("Exited");
}

