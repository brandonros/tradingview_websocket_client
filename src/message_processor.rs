use async_trait::async_trait;

use crate::parsed_message::ParsedTradingViewMessage;

#[async_trait]
pub trait TradingViewMessageProcessor {
    async fn process_message(&self, name: String, message: ParsedTradingViewMessage);
}
