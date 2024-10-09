use std::sync::Arc;

use miniserde::Deserialize;

use crate::message_processor::TradingViewMessageProcessor;
use crate::client::TradingViewClient;

#[derive(Deserialize, Clone)]
pub enum TradingViewClientMode {
    Standard,
    Streaming
}

#[derive(Deserialize, Clone)]
pub struct TradingViewClientConfig {
    pub name: String,
    pub auth_token: String,
    pub chart_symbols: Vec<String>,
    pub quote_symbols: Vec<String>,
    pub indicators: Vec<String>,
    pub timeframe: String,
    pub range: usize,
    pub mode: TradingViewClientMode
}

impl TradingViewClientConfig {
    pub fn to_client(&self, message_processor: Arc<Box<dyn TradingViewMessageProcessor + Send + Sync>>) -> TradingViewClient {
        TradingViewClient::new(self.clone(), message_processor)
    }
}
