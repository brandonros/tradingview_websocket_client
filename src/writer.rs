use websocket_client::WebSocketWriter;

use crate::frame::TradingViewFrame;
use crate::futures_provider::io::AsyncWrite;
use crate::types::Result;

/// TradingViewWriter handles writing TradingView frames.
pub struct TradingViewWriter<W>
where
    W: AsyncWrite + Unpin,
{
    writer: WebSocketWriter<W>,
}

impl<W> TradingViewWriter<W>
where
    W: AsyncWrite + Unpin,
{
    /// Creates a new `TradingViewWriter` with the given `WebSocketWriter`.
    pub fn new(writer: WebSocketWriter<W>) -> Self {
        Self { writer }
    }

    /// Writes a message to the TradingView server.
    pub async fn write_frame(&mut self, message: &str) -> Result<()> {
        let tv_message = TradingViewFrame::serialize(message);
        log::debug!("write_frame: tv_message = {tv_message}");
        self.writer.write_frame(&tv_message).await
    }

    pub async fn set_auth_token(&mut self, auth_token: &str) -> Result<()> {
        let message = format!(r#"{{"m":"set_auth_token","p":["{auth_token}"]}}"#);
        self
            .write_frame(&message)
            .await
    }

    pub async fn set_locale(&mut self, language_code: &str, region_code: &str) -> Result<()> {
        let message = format!(r#"{{"m":"set_locale","p":["{language_code}", "{region_code}"]}}"#);
        self
            .write_frame(&message)
            .await
    }

    pub async fn chart_create_session(&mut self, chart_session_id: &str) -> Result<()> {
        let message = format!(r#"{{"m":"chart_create_session","p":["{chart_session_id}",""]}}"#);
        self
            .write_frame(&message)
            .await
    }

    pub async fn switch_timezone(&mut self, chart_session_id: &str, timezone: &str) -> Result<()> {
        let message = format!(r#"{{"m":"switch_timezone","p":["{chart_session_id}","{timezone}"]}}"#);
        self
            .write_frame(&message)
            .await
    }

    pub async fn quote_create_session(&mut self, quote_session_id: &str) -> Result<()> {
        let message = format!(r#"{{"m":"quote_create_session","p":["{quote_session_id}",""]}}"#);
        self
            .write_frame(&message)
            .await
    }

    pub async fn quote_add_symbols(&mut self, quote_session_id: &str, symbol: &str) -> Result<()> {
        let message = format!(r#"{{"m":"quote_add_symbols","p":["{quote_session_id}","{symbol}"]}}"#);
        self
            .write_frame(&message)
            .await
    }

    pub async fn resolve_symbol(&mut self, chart_session_id: &str, symbol_id: &str, symbol: &str) -> Result<()> {
        let message = format!(r#"{{"m":"resolve_symbol","p":["{chart_session_id}","{symbol_id}", "{symbol}"]}}"#);
        self
            .write_frame(&message)
            .await
    }

    pub async fn create_series(&mut self, chart_session_id: &str, series_id: &str, unk1: &str, symbol_id: &str, timeframe: &str, range: usize) -> Result<()> {
        let message = format!(r#"{{"m":"create_series","p":["{chart_session_id}","{series_id}","{unk1}","{symbol_id}","{timeframe}",{range},""]}}"#);
        self
            .write_frame(&message)
            .await
    }

    pub async fn request_more_tickmarks(&mut self, chart_session_id: &str, series_id: &str, range: usize) -> Result<()> {
        let message = format!(r#"{{"m":"request_more_tickmarks","p":["{chart_session_id}","{series_id}",{range}]}}"#);
        self
            .write_frame(&message)
            .await
    }

    pub async fn quote_fast_symbols(&mut self, quote_session_id: &str, symbol: &str) -> Result<()> {
        let message = format!(r#"{{"m":"quote_fast_symbols","p":["{quote_session_id}","{symbol}"]}}"#);
        self
            .write_frame(&message)
            .await
    }   

    pub async fn quote_set_fields(&mut self, quote_session_id: &str) -> Result<()> {
        // TODO: make fields configurable
        let message = format!(r#"{{"m":"quote_set_fields","p":["{quote_session_id}","base-currency-logoid","ch","chp","currency-logoid","currency_code","currency_id","base_currency_id","current_session","description","exchange","format","fractional","is_tradable","language","local_description","listed_exchange","logoid","lp","lp_time","minmov","minmove2","original_name","pricescale","pro_name","short_name","type","typespecs","update_mode","volume","variable_tick_size","value_unit_id","unit_id","measure"]}}"#);
        self
            .write_frame(&message)
            .await
    }  

    pub async fn create_study(&mut self, chart_session_id: &str, study_id: &str, session_id: &str, series_id: &str, name: &str, value: &str) -> Result<()> {
        // TODO: make fields configurable
        let message = format!(r#"{{
            "m":"create_study",
            "p":[
                "{chart_session_id}",
                "{study_id}",
                "{session_id}",
                "{series_id}",
                "{name}",
                {value}
            ]
        }}"#);
        self
            .write_frame(&message)
            .await
    }   

    pub async fn pong(&mut self, nonce: usize) -> Result<()> {
        let message = format!("~h~{nonce}");
        self
            .write_frame(&message)
            .await
    }
}
