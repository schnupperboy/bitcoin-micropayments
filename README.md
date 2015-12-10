# Bitcoin micropayment services
Executable written in the Rust programming language which exposes a few useful interfaces over HTTP which can be used to handle bitcoin micropayments.

**WARNING**: DO NOT USE THIS IN PRODUCTION. This is my playground for learning the Rust programming language and for example does not contain any protection against Bitcoin double-spending attacks.

## Prerequisities
* Rust 1.6 (currently nightly)

## Build
```
$ git clone https://github.com/schnupperboy/bitcoin-micropayments.git
$ cd bitcoin-micropayments

$ cargo build
```

## Usage
Run the executable by specifying an ip address and port to bind the HTTP server to.
```
$ bitcoin-micropayments <ip address> <port>
```

### Example
```
$ bitcoin-micropayments 127.0.0.1 5000
Listening on http://127.0.0.1:5000
```

### HTTP interface

#### QR code
Generates a QR code which can be scanned from a mobile Bitcoin Wallet app. It contains the amount of the transaction as well as the receiving Bitcoin address.

**HTTP method:** `GET`
**URL path:** `/qr_code`
**URL parameters:** `btc_amount` ("float" string), btc_receiver_address ("hex" string)
**Response body:** PNG data (MIME type: image/png)

```
$ curl 'http://127.0.0.1:5000/qr_code?btc_amount=0.002&btc_receiver_address=13cSu17oJ2dFX5mTGeMTh8N3UTPv2pN5CZ'
```

#### Exchange rate
In order to be able to show the price in your local currency on your frontend you can convert to the current Bitcoin prize.

**NOTE:** Currently only Euro is the supported

**HTTP method:** `GET`
**URL path:** `/exchange_rate`
**URL parameters:**`eur_amount` ("float" string) 
**Response body:** corresponding amount in BTC amount

```
$ curl 'http://127.0.0.1:5000/exchange_rate?eur_amount=1.5'
0.003926804366606456
```

#### Payment detection
Only returns a response when the payment is complete, the time has expired or an internal error occurred  

**HTTP method:** `GET`
**URL path:** `/detect_payment`
**URL parameters:** `btc_amount` ("float" string), `btc_receiver_address` ("hex" string)
**Response body:** `InsufficientAmount` or `Timeout` or `BackendError`

```
$ curl 'http://127.0.0.1:5000/detect_payment?btc_amount=0.002&btc_receiver_address=13cSu17oJ2dFX5mTGeMTh8N3UTPv2pN5CZ'
0.003926804366606456
```