use std::sync::Arc;

use crate::message_processor::TradingViewMessageProcessor;
use crate::client::TradingViewClient;

#[derive(Clone)]
pub struct TradingViewClientConfig {
    pub name: String,
    pub auth_token: String,
    pub chart_symbols: Vec<String>,
    pub quote_symbols: Vec<String>,
    pub indicators: Vec<String>,
    pub timeframe: String,
    pub range: usize,
    pub message_processor: Arc<Box<dyn TradingViewMessageProcessor + Send + Sync>>
}

impl TradingViewClientConfig {
    pub fn to_client(&self) -> TradingViewClient {
        TradingViewClient::new(self.clone())
    }
}
