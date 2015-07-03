pub trait PaymentDetection {
	fn new(address: &'static str, amount: u64) -> Self;
	fn wait(&self, );
}