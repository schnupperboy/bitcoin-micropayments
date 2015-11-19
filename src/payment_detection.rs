#[derive(Debug)]
pub enum PaymentError {
	InsufficientAmount,
	Timeout,
	BackendError
}

pub trait PaymentDetection {
	fn await_payment(address: &str, amount: u64) -> Result<(), PaymentError>;
}