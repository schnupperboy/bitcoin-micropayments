#[derive(Debug)]
pub enum ExchangeError {
    BackendError
}

pub trait EuroExchangeRate {
    fn convert(euro: f64) -> Result<f64, ExchangeError>;
}