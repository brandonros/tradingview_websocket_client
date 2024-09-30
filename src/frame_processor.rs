use async_trait::async_trait;

use crate::parsed_frame::ParsedTradingViewFrame;

#[async_trait]
pub trait TradingViewFrameProcessor {
    async fn process_frame(&self, name: String, frame: ParsedTradingViewFrame);
}
