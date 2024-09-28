#[cfg(not(any(feature = "futures", feature = "futures-lite")))]
compile_error!(
    "You must enable either the `futures` or `futures-lite` feature to build this crate."
);

#[cfg(feature = "futures")]
use futures as futures_provider;

#[cfg(feature = "futures-lite")]
use futures_lite as futures_provider;

use futures_provider::io::{AsyncRead, AsyncWrite};
use websocket_client::WebSocketClient;
use bytes::{Buf, BytesMut};
use nom::{
    bytes::streaming::{tag as tag_streaming, take as take_streaming},
    character::streaming::digit1 as digit1_streaming,
    
    bytes::complete::tag as tag_complete,
    character::complete::digit1 as digit1_complete,

    error::ErrorKind,
    IResult,
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[derive(Debug)]
pub struct TradingViewFrame {
    pub payload: Vec<u8>,
}

impl TradingViewFrame {
    /// Serializes a message into the TradingView frame format.
    pub fn serialize(input: &str) -> String {
        let input_len = input.len();
        format!("~m~{input_len}~m~{input}")
    }

    /// Parses a TradingView frame from the input bytes.
    pub fn parse(input: &[u8]) -> IResult<&[u8], TradingViewFrame> {
        // Parse the prefix "~m~"
        let (input, _) = tag_streaming("~m~")(input)?;

        // Parse digits representing the length
        let (input, len_digits) = digit1_streaming(input)?;

        // Parse the next "~m~"
        let (input, _) = tag_streaming("~m~")(input)?;

        // Convert len_digits to usize
        let input_len = std::str::from_utf8(len_digits)
            .map_err(|_| {
                nom::Err::Failure(nom::error::Error::new(len_digits, ErrorKind::Digit))
            })?
            .parse::<usize>()
            .map_err(|_| {
                nom::Err::Failure(nom::error::Error::new(len_digits, ErrorKind::Digit))
            })?;

        // Take input_len bytes as the payload
        let (input, payload) = take_streaming(input_len)(input)?;

        Ok((
            input,
            TradingViewFrame {
                payload: payload.to_vec(),
            },
        ))
    }
}

#[derive(Debug)]
pub struct TradingViewPingFrame {
    pub nonce: usize,
}

impl TradingViewPingFrame {
    /// Serializes a message into the TradingView ping frame format.
    pub fn serialize(nonce: usize) -> String {
        format!("~h~{nonce}")
    }

    /// Parses a TradingView frame from the input bytes.
    pub fn parse(input: &[u8]) -> IResult<&[u8], TradingViewPingFrame> {
        // Parse the prefix "~h~"
        let (input, _) = tag_complete("~h~")(input)?;

        // Parse digits representing the nonce
        let (input, text_digits) = digit1_complete(input)?;

        // Convert text_digits to usize
        let nonce = std::str::from_utf8(text_digits)
            .map_err(|_| {
                nom::Err::Failure(nom::error::Error::new(text_digits, ErrorKind::Digit))
            })?
            .parse::<usize>()
            .map_err(|_| {
                nom::Err::Failure(nom::error::Error::new(text_digits, ErrorKind::Digit))
            })?;

        Ok((
            input,
            TradingViewPingFrame {
                nonce
            },
        ))
    }
}

pub struct TradingViewClient<R, W> {
    ws_client: WebSocketClient<R, W>,
    buffer: BytesMut,
}

impl<R, W> TradingViewClient<R, W>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    /// Creates a new `TradingViewClient` with the given `WebSocketClient`.
    pub fn new(ws_client: WebSocketClient<R, W>) -> Self {
        Self {
            ws_client,
            buffer: BytesMut::with_capacity(1024 * 1024),
        }
    }

    /// Reads the next TradingView frame, handling partial frames and buffering.
    pub async fn read_frame(&mut self) -> Result<Option<TradingViewFrame>> {
        loop {
            // Try to parse a TradingView frame from the tv_buffer
            if let Some(frame) = self.parse_frame()? {
                return Ok(Some(frame));
            }

            // Need more data; read the next WebSocket frame
            match self.ws_client.read_frame().await? {
                Some(ws_frame) => {
                    // Append the payload to the tv_buffer
                    self.buffer.extend_from_slice(&ws_frame.payload);
                }
                None => {
                    // No more WebSocket frames
                    if self.buffer.is_empty() {
                        return Ok(None);
                    } else {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::UnexpectedEof,
                            "Stream closed with incomplete TradingView frame",
                        )
                        .into());
                    }
                }
            }
        }
    }

    /// Parses a TradingView frame from the buffer.
    fn parse_frame(&mut self) -> Result<Option<TradingViewFrame>> {
        if self.buffer.is_empty() {
            return Ok(None);
        }

        let input = &self.buffer[..];

        match TradingViewFrame::parse(input) {
            Ok((remaining, frame)) => {
                let parsed_len = input.len() - remaining.len();
                self.buffer.advance(parsed_len);

                Ok(Some(frame))
            }
            Err(nom::Err::Incomplete(_)) => {
                // Not enough data, need to read more
                Ok(None)
            }
            Err(e) => {
                // Parsing error
                Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Parsing error: {:?}", e),
                )
                .into())
            }
        }
    }

    /// Writes a message to the TradingView server.
    pub async fn write_frame(&mut self, message: &str) -> Result<()> {
        let tv_message = TradingViewFrame::serialize(message);
        log::debug!("write_frame: tv_message = {tv_message}");
        self.ws_client.write_frame(&tv_message).await
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

    pub async fn create_series(&mut self, chart_session_id: &str, symbol_id: &str, series_id: &str, timeframe: &str, range: &str) -> Result<()> {
        let unk1 = "s1"; // TODO: not sure
        let message = format!(r#"{{"m":"create_series","p":["{chart_session_id}","{series_id}","{unk1}","{symbol_id}","{timeframe}",{range},""]}}"#);
        self
            .write_frame(&message)
            .await
    }

    pub async fn request_more_tickmarks(&mut self, chart_session_id: &str, series_id: &str, range: &str) -> Result<()> {
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
        let message = TradingViewPingFrame::serialize(nonce);
        self
            .write_frame(&message)
            .await
    }
}
