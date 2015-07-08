pub trait PaymentDetection<'a> {
	fn new(address: &'a str, amount: u64) -> Self;
	fn wait(&self, );
}