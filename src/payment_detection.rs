#[derive(Debug)]
pub enum PaymentError {
	InsufficientAmount,
	Timeout,
	BackendError
}

pub trait PaymentDetection<'a> {
	fn new(address: &'a str, amount: u64) -> Self;
	fn wait(&self, ) -> Result<(), PaymentError>;
}