use std::sync::Arc;
use std::time::Duration;

use async_lock::RwLock;
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
use crate::frame_processor::TradingViewFrameProcessor;

pub struct TradingViewClient
{
    name: String,
    auth_token: String,
    chart_symbol: String,
    quote_symbol: String,
    indicators: Vec<String>,
    timeframe: String,
    range: usize,
    frame_processor: Arc<Box<dyn TradingViewFrameProcessor + Send + Sync>>
}

impl TradingViewClient
{
    pub fn new(name: String, auth_token: String, chart_symbol: String, quote_symbol: String, indicators: Vec<String>, timeframe: String, range: usize, frame_processor: Arc<Box<dyn TradingViewFrameProcessor + Send + Sync>>) -> Self {
        Self {
            name,
            auth_token,
            chart_symbol,
            quote_symbol,
            indicators,
            timeframe,
            range,
            frame_processor
        }
    }

    pub async fn run(&self) {
        // Build the URI for the request
        let uri: Uri = "wss://data.tradingview.com/socket.io/websocket?type=chart".parse().expect("Failed to parse URI");

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

        // prepare buffer + references
        let buffer: Vec<TradingViewFrameWrapper> = Vec::new();
        let buffer = RwLock::new(buffer);
        let buffer_arc = Arc::new(buffer);
        let reader_handle_buffer_ref = buffer_arc.clone();

        // Spawn the reader task
        let _reader_handle = std::thread::spawn(move || {
            futures_provider::future::block_on(async {
                loop {
                    match tv_reader.read_frame().await {
                        Ok(result) => {
                            match result {
                                Some(frame) => {
                                    // add frame to buffer
                                    let mut write_lock = reader_handle_buffer_ref.write().await;
                                    write_lock.push(frame);
                                    drop(write_lock);
                                },
                                None => panic!("received none"),
                            }
                        },
                        Err(err) => panic!("{err:?}"),
                    }
                }
            })
        });
        
        // Wait for server hello frame with timeout
        let server_hello_frame = utilities::run_with_timeout(Duration::from_secs(1), Box::pin(utilities::wait_for_message(buffer_arc.clone(), |frame| frame.payload.contains("javastudies"))))
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

        // quote_create_session
        // quote_add_symbols symbol with session in it

        // resolve symbol
        let symbol_id = "sds_sym_1";
        tv_writer.resolve_symbol(chart_session_id1, symbol_id, &self.chart_symbol).await.expect("failed to add symbol to resolve symbol");

        // wait for symbol resolved frame
        let symbol_resolved_frame = utilities::run_with_timeout(Duration::from_secs(1), Box::pin(utilities::wait_for_message(buffer_arc.clone(), |frame| frame.payload.contains("symbol_resolved"))))
            .await
            .expect("timed out")
            .expect("failed to get symbol resolved frame");
        log::info!("symbol_resolved_frame = {symbol_resolved_frame:?}");

        // add symbol to chart session as series
        let series_id = "sds_1";
        tv_writer.create_series(chart_session_id1, series_id, "s1",  symbol_id, &self.timeframe, self.range).await.expect("failed to create series");

        // switch chart timezone
        tv_writer.switch_timezone(chart_session_id1, "exchange").await.expect("failed to switch chart timezone");

        // wait for series loading frame
        let series_loading_frame = utilities::run_with_timeout(Duration::from_secs(1), Box::pin(utilities::wait_for_message(buffer_arc.clone(), |frame| frame.payload.contains("series_loading"))))
            .await
            .expect("timed out")
            .expect("failed to get series loading frame");
        log::info!("series_loading_frame = {series_loading_frame:?}");

        // wait for timescale update frame
        let timescale_update_frame = utilities::run_with_timeout(Duration::from_secs(2), Box::pin(utilities::wait_for_message(buffer_arc.clone(), |frame| frame.payload.contains("timescale_update"))))
            .await
            .expect("timed out")
            .expect("failed to get timesale update frame");
        log::info!("timescale_update_frame = {timescale_update_frame:?}");

        // wait for series completed frame
        let series_completed_frame = utilities::run_with_timeout(Duration::from_secs(1), Box::pin(utilities::wait_for_message(buffer_arc.clone(), |frame| frame.payload.contains("series_completed"))))
            .await
            .expect("timed out")
            .expect("failed to get series completed frame");
        log::info!("series_completed_frame = {series_completed_frame:?}");

        // chart_symbol quote session
        {
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
            let quote_completed_frame = utilities::run_with_timeout(Duration::from_secs(1), Box::pin(utilities::wait_for_message(buffer_arc.clone(), |frame| frame.payload.contains("quote_completed"))))
                .await
                .expect("timed out")
                .expect("failed to get quote completed frame");
            log::info!("quote_completed_frame = {quote_completed_frame:?}");
        }

        // quote_symbol quote session
        {
            // create quote session
            let quote_session_id2 = "qs_000000000002";
            tv_writer.quote_create_session(quote_session_id2).await.expect("failed to create quote session");

            // set quote session fields
            tv_writer.quote_set_fields(quote_session_id2).await.expect("failed to set quote fields");

            // add symbol to quote session
            tv_writer.quote_add_symbols(quote_session_id2, &self.quote_symbol).await.expect("failed to add symbol to quote session");

            // turn on quote fast symbols for quote session
            tv_writer.quote_fast_symbols(quote_session_id2, &self.quote_symbol).await.expect("failed to turn on quote fast symbols");

            // wait for quote completed frame
            let quote_completed_frame = utilities::run_with_timeout(Duration::from_secs(1), Box::pin(utilities::wait_for_message(buffer_arc.clone(), |frame| frame.payload.contains("quote_completed"))))
                .await
                .expect("timed out")
                .expect("failed to get quote completed frame");
            log::info!("quote_completed_frame = {quote_completed_frame:?}");
        }

        // TODO: request more tickmarks from symbol?

        // optionally create study session
        if self.indicators.len() > 0 {
            let study_session_id = "st1";
            tv_writer.create_study(chart_session_id1, study_session_id, "sessions_1", series_id, "Sessions@tv-basicstudies-241", "{}").await.expect("failed to create study session");

            // wait for study loading frame
            let study_loading_frame = utilities::run_with_timeout(Duration::from_secs(1), Box::pin(utilities::wait_for_message(buffer_arc.clone(), |frame| frame.payload.contains("study_loading"))))
                .await
                .expect("timed out")
                .expect("failed to get study loading frame");
            log::info!("study_loading_frame = {study_loading_frame:?}");

            // wait for study completed frame
            let study_completed_frame = utilities::run_with_timeout(Duration::from_secs(1), Box::pin(utilities::wait_for_message(buffer_arc.clone(), |frame| frame.payload.contains("study_completed"))))
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

                // wait for study loading frame
                let study_loading_frame = utilities::run_with_timeout(Duration::from_secs(1), Box::pin(utilities::wait_for_message(buffer_arc.clone(), |frame| frame.payload.contains("study_loading"))))
                    .await
                    .expect("timed out")
                    .expect("failed to get study loading frame");
                log::info!("study_loading_frame = {study_loading_frame:?}");

                // wait for study completed frame
                let study_completed_frame = utilities::run_with_timeout(Duration::from_secs(1), Box::pin(utilities::wait_for_message(buffer_arc.clone(), |frame| frame.payload.contains("study_completed"))))
                    .await
                    .expect("timed out")
                    .expect("failed to get study completed frame");
                log::info!("study_completed_frame = {study_completed_frame:?}");
            }
        }

        // read all frames
        loop {
            let frame_result = utilities::wait_for_message(buffer_arc.clone(), |_| true).await;
            match frame_result {
                Some(frame) => {
                    let parsed_frame = ParsedTradingViewFrame::from_string(&frame.payload).expect("failed to parse frame");
                    match &parsed_frame {
                        ParsedTradingViewFrame::Ping(nonce) => {
                            log::info!("ping nonce = {nonce}");
                            tv_writer.pong(*nonce).await.expect("failed to pong");
                        },
                        _ => {
                            // send to frame processor
                            self.frame_processor.process_frame(self.name.clone(), parsed_frame).await;
                        }
                    }
                },
                None => panic!("closed")
            }
        }
    }
}

