use std::time::Duration;

use http::{Request, Uri, Version};
use http_client::HttpClient;

use websocket_client::{WebSocketHelpers, WebSocketReader, WebSocketWriter};

use crate::parsed_frame::ParsedTradingViewFrame;
use crate::utilities;
use crate::futures_provider;
use crate::futures_provider::io::{BufReader, BufWriter};
use crate::reader::TradingViewReader;
use crate::writer::TradingViewWriter;
use crate::frame_wrapper::TradingViewFrameWrapper;

pub struct TradingViewClient {
    name: String,
    auth_token: String,
    chart_symbol: String,
    quote_symbol: String,
    indicators: Vec<String>,
    timeframe: String,
    range: usize
}

impl TradingViewClient {
    pub fn new(name: String, auth_token: String, chart_symbol: String, quote_symbol: String, indicators: Vec<String>, timeframe: String, range: usize) -> Self {
        Self {
            name,
            auth_token,
            chart_symbol,
            quote_symbol,
            indicators,
            timeframe,
            range
        }
    }

    pub async fn run(&self) {
        // Build the URI for the request
        let uri: Uri = "wss://data.tradingview.com/socket.io/websocket".parse().expect("Failed to parse URI");

        // Build the GET request
        let request = Request::builder()
            .method("GET")
            .version(Version::HTTP_11)
            .uri(uri)
            .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/129.0.0.0 Safari/537.36")
            .header("Host", "data.tradingview.com")
            .header("Origin", "https://www.tradingview.com")            
            .header("Connection", "Upgrade")
            .header("Upgrade", "websocket")      
            .header("Sec-WebSocket-Version", "13")                        
            .header("Sec-WebSocket-Key", WebSocketHelpers::generate_sec_websocket_key())    
            .body(())
            .expect("Failed to build request");

        // Get the response
        let mut stream = HttpClient::connect(&request).await.expect("connect failed");
        let response = HttpClient::send::<(), String>(&mut stream, &request).await.expect("request failed");
        log::info!("response = {response:?}");

        // split
        let (reader, writer) = futures_provider::io::split(stream);
        let reader = BufReader::new(reader);
        let writer = BufWriter::new(writer);

        // create websocket client
        let ws_reader = WebSocketReader::new(reader);
        let ws_writer = WebSocketWriter::new(writer);        

        // Create the TradingViewClient
        let mut tv_reader = TradingViewReader::new(ws_reader);
        let mut tv_writer = TradingViewWriter::new(ws_writer);

        // channels
        let (frame_tx, frame_rx) = async_channel::unbounded::<TradingViewFrameWrapper>();

        // Spawn the reader task
        let _reader_handle = std::thread::spawn(move || {
            futures_provider::future::block_on(async {
                loop {
                    match tv_reader.read_frame().await {
                        Ok(result) => {
                            match result {
                                Some(frame) => frame_tx.send(frame).await.expect("failed to send frame"),
                                None => panic!("received none"),
                            }
                        },
                        Err(err) => panic!("{err:?}"),
                    }
                }
            })
        });

        // Introduce the buffer
        let mut buffer = Vec::new();
        
        // Wait for server hello frame with timeout
        let server_hello_frame = utilities::run_with_timeout(Duration::from_secs(1), Box::pin(utilities::wait_for_message(&frame_rx, &mut buffer, |frame| frame.payload.contains("javastudies"))))
            .await
            .expect("timed out")
            .expect("failed to get server hello frame");
        log::info!("server_hello_frame = {server_hello_frame:?}");

        // set auth token
        tv_writer.set_auth_token(&self.auth_token).await.expect("failed to set auth token");
        
        // set locale
        tv_writer.set_locale("en", "US").await.expect("failed to set locale");

        // create chart session
        let chart_session_id1 = "cs_000000000001";
        tv_writer.chart_create_session(chart_session_id1).await.expect("failed to create chart session");

        // resolve symbol
        let symbol_id = "sds_sym_1";
        tv_writer.resolve_symbol(chart_session_id1, symbol_id, &self.chart_symbol).await.expect("failed to add symbol to resolve symbol");

        // switch chart timezone
        tv_writer.switch_timezone(chart_session_id1, "exchange").await.expect("failed to switch chart timezone");

        // add symbol to chart session as series
        let series_id = "sds_1";
        tv_writer.create_series(chart_session_id1, series_id, "s1",  symbol_id, &self.timeframe, self.range).await.expect("failed to create series");

         // wait for series loading frame
         let series_loading_frame = utilities::run_with_timeout(Duration::from_secs(1), Box::pin(utilities::wait_for_message(&frame_rx, &mut buffer, |frame| {
            log::info!("frame = {frame:?}");
            frame.payload.contains("series_loading")
        })))
            .await
            .expect("timed out")
            .expect("failed to get series loading frame");
        log::info!("series_loading_frame = {series_loading_frame:?}");

        // wait for series completed frame
        let series_completed_frame = utilities::run_with_timeout(Duration::from_secs(1), Box::pin(utilities::wait_for_message(&frame_rx, &mut buffer, |frame| {
            log::info!("frame = {frame:?}");
            frame.payload.contains("series_completed")
        })))
            .await
            .expect("timed out")
            .expect("failed to get series completed frame");
        log::info!("series_completed_frame = {series_completed_frame:?}");

        /*{
            // create quote session
            let quote_session_id1 = "qs_000000000001";
            tv_writer.quote_create_session(quote_session_id1).await.expect("failed to create quote session");

            // set quote session fields
            tv_writer.quote_set_fields(quote_session_id1).await.expect("failed to set quote fields");

            // add symbol to quote session
            tv_writer.quote_add_symbols(quote_session_id1, &self.chart_symbol).await.expect("failed to add symbol to quote session");

            // turn on quote fast symbols for quote session
            tv_writer.quote_fast_symbols(quote_session_id1, &self.chart_symbol).await.expect("failed to turn on quote fast symbols");

            // wait for quote completed frame
            let quote_completed_frame = utilities::run_with_timeout(Duration::from_secs(1), Box::pin(utilities::wait_for_message(&frame_rx, &mut buffer, |frame| {
                log::info!("frame = {frame:?}");
                frame.payload.contains("quote_completed")
            })))
                .await
                .expect("timed out")
                .expect("failed to get quote completed frame");
            log::info!("quote_completed_frame = {quote_completed_frame:?}");
        }

        {
            // create quote session
            let quote_session_id2 = "qs_000000000002";
            tv_writer.quote_create_session(quote_session_id2).await.expect("failed to create quote session");

            // set quote session fields
            tv_writer.quote_set_fields(quote_session_id2).await.expect("failed to set quote fields");

            // add symbol to quote session
            tv_writer.quote_add_symbols(quote_session_id2, &self.quote_symbol).await.expect("failed to add symbol to quote session");

            // turn on quote fast symbols for quote session
            tv_writer.quote_fast_symbols(quote_session_id2, &self.chart_symbol).await.expect("failed to turn on quote fast symbols");

            // wait for quote completed frame
            let quote_completed_frame = utilities::run_with_timeout(Duration::from_secs(1), Box::pin(utilities::wait_for_message(&frame_rx, &mut buffer, |frame| {
                log::info!("frame = {frame:?}");
                frame.payload.contains("quote_completed")
            })))
                .await
                .expect("timed out")
                .expect("failed to get quote completed frame");
            log::info!("quote_completed_frame = {quote_completed_frame:?}");
        }*/

        // TODO: request more tickmarks from symbol?

        // optionally create study session
        if self.indicators.len() > 0 {
            let study_session_id = "st1";
            tv_writer.create_study(chart_session_id1, study_session_id, "sessions_1", series_id, "Sessions@tv-basicstudies-241", "{}").await.expect("failed to create study session");

            // wait for study completed frame
            let study_completed_frame = utilities::run_with_timeout(Duration::from_secs(1), Box::pin(utilities::wait_for_message(&frame_rx, &mut buffer, |frame| {
                log::info!("frame = {frame:?}");
                frame.payload.contains("study_completed")
            })))
                .await
                .expect("timed out")
                .expect("failed to get study completed frame");
            log::info!("study_completed_frame = {study_completed_frame:?}");

            let mut index = 2;
            for indciator in &self.indicators {
                let study_value = indciator;
                let study_id = format!("st{index}");
                tv_writer.create_study(chart_session_id1, &study_id, study_session_id, series_id, "Script@tv-scripting-101!", study_value).await.expect("failed to add to study session");
                index += 1;
            }
        }

        // read all frames
        loop {
            let frame_result = utilities::wait_for_message(&frame_rx, &mut buffer, |_| true).await;
            match frame_result {
                Some(frame) => {
                    log::info!("[{}]: frame_payload = {}", self.name, frame.payload);
                    let parsed_frame = ParsedTradingViewFrame::from_string(&frame.payload).expect("failed to parse frame");
                    match parsed_frame {
                        ParsedTradingViewFrame::Ping(nonce) => {
                            log::info!("ping nonce = {nonce}");
                            tv_writer.pong(nonce).await.expect("failed to pong");
                        },
                        ParsedTradingViewFrame::ServerHello(server_hello_frame) => {
                            log::info!("server_hello_frame = {server_hello_frame:?}");
                        },
                        ParsedTradingViewFrame::QuoteSeriesData(quote_series_data_frame) => {
                            log::info!("quote_series_data_frame = {quote_series_data_frame:?}");
                        },
                        ParsedTradingViewFrame::DataUpdate(data_update_frame) => {
                            log::info!("data_update_frame = {data_update_frame:?}");
                        },
                        ParsedTradingViewFrame::QuoteCompleted(quote_completed_frame) => {
                            log::info!("quote_completed_frame = {quote_completed_frame:?}");
                        },
                        ParsedTradingViewFrame::TimescaleUpdate(timescale_updated_frame) => {
                            log::info!("timescale_updated_frame = {timescale_updated_frame:?}");
                        },
                        ParsedTradingViewFrame::SeriesLoading(series_loading_frame) => {
                            log::info!("series_loading_frame = {series_loading_frame:?}");
                        },
                        ParsedTradingViewFrame::SymbolResolved(symbol_resolved_frame) => {
                            log::info!("symbol_resolved_frame = {symbol_resolved_frame:?}");
                        },
                        ParsedTradingViewFrame::SeriesCompleted(series_completed_frame) => {
                            log::info!("series_completed_frame = {series_completed_frame:?}");
                        },
                        ParsedTradingViewFrame::StudyLoading(study_loading_frame) => {
                            log::info!("study_loading_frame = {study_loading_frame:?}");
                        },
                        ParsedTradingViewFrame::StudyError(study_error_frame) => {
                            log::info!("study_error_frame = {study_error_frame:?}");
                        },
                        ParsedTradingViewFrame::StudyCompleted(study_completed_frame) => {
                            log::info!("study_completed_frame = {study_completed_frame:?}");
                        },
                    }
                },
                None => panic!("closed")
            }
        }
    }
}

