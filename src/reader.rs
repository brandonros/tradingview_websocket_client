use websocket_client::WebSocketReader;
use bytes::{Buf, BytesMut};

use crate::futures_provider::io::AsyncRead;
use crate::frame::TradingViewFrame;
use crate::types::Result;

pub struct TradingViewReader<R>
where
    R: AsyncRead + Unpin,
{
    reader: WebSocketReader<R>,
    buffer: BytesMut,
}

impl<R> TradingViewReader<R>
where
    R: AsyncRead + Unpin,
{
    /// Creates a new `TradingViewReader` with the given `WebSocketReader`.
    pub fn new(reader: WebSocketReader<R>) -> Self {
        Self {
            reader,
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
            match self.reader.read_frame().await? {
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
}