use websocket_client::WebSocketReader;
use bytes::{Buf, BytesMut};
use futures_lite::io::AsyncRead;

use crate::message_wrapper::TradingViewMessageWrapper;
use crate::types::Result;

pub struct TradingViewReader<R>
where
    R: AsyncRead + Unpin,
{
    ws_reader: WebSocketReader<R>,
    buffer: BytesMut,
}

impl<R> TradingViewReader<R>
where
    R: AsyncRead + Unpin,
{
    /// Creates a new `TradingViewReader` with the given `WebSocketReader`.
    pub fn new(ws_reader: WebSocketReader<R>) -> Self {
        Self {
            ws_reader,
            buffer: BytesMut::with_capacity(1024 * 1024),
        }
    }

    /// Reads the next TradingView message, handling partial messages and buffering.
    pub async fn read_message(&mut self) -> Result<Option<TradingViewMessageWrapper>> {
        loop {
            // Try to parse a TradingView message from the tv_buffer
            if let Some(message) = self.parse_message()? {
                return Ok(Some(message));
            }

            // Need more data; read the next WebSocket message
            match self.ws_reader.read_message().await? {
                Some(ws_message) => {
                    self.buffer.extend_from_slice(&ws_message.payload_buffer);
                }
                None => {
                    // No more WebSocket messages
                    if self.buffer.is_empty() {
                        return Ok(None);
                    } else {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::UnexpectedEof,
                            "Stream closed with incomplete TradingView message",
                        )
                        .into());
                    }
                }
            }
        }
    }

    /// Parses a TradingView message from the buffer.
    fn parse_message(&mut self) -> Result<Option<TradingViewMessageWrapper>> {
        if self.buffer.is_empty() {
            return Ok(None);
        }

        let input = &self.buffer[..];

        match TradingViewMessageWrapper::parse(input) {
            Ok((remaining, message)) => {
                let parsed_len = input.len() - remaining.len();
                self.buffer.advance(parsed_len);

                Ok(Some(message))
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
}