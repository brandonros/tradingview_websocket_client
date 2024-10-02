use std::sync::Arc;

use crate::message_processor::TradingViewMessageProcessor;
use crate::client::TradingViewClient;

pub struct TradingViewClientConfig {
    pub name: String,
    pub auth_token: String,
    pub chart_symbol: String,
    pub quote_symbols: Vec<String>,
    pub indicators: Vec<String>,
    pub timeframe: String,
    pub range: usize,
    pub message_processor: Arc<Box<dyn TradingViewMessageProcessor + Send + Sync>>
}

impl TradingViewClientConfig {
    pub fn to_client(&self) -> TradingViewClient {
        TradingViewClient::new(
            self.name.to_string(),
            self.auth_token.to_string(),
            self.chart_symbol.to_string(),
            self.quote_symbols.clone(),
            self.indicators.clone(),
            self.timeframe.to_string(),
            self.range,
            self.message_processor.clone()
        )
    }
}
