use std::sync::Arc;
use std::time::Duration;

use async_lock::RwLock;
use http::{Request, Uri, Version};
use http_client::HttpClient;

use websocket_client::{WebSocketHelpers, WebSocketReader, WebSocketWriter};
use futures_lite::io::{BufReader, BufWriter};

use crate::parsed_message::ParsedTradingViewMessage;
use crate::utilities;
use crate::client_config::TradingViewClientConfig;
use crate::reader::TradingViewReader;
use crate::writer::TradingViewWriter;
use crate::message_wrapper::TradingViewMessageWrapper;
use crate::types::Result;

pub struct TradingViewClient {
    config: TradingViewClientConfig
}

impl TradingViewClient {
    pub fn new(config: TradingViewClientConfig) -> Self {
        Self {
            config
        }
    }

    pub async fn run(&self) -> Result<()> {
        // Build the URI for the request
        let uri: Uri = "wss://data.tradingview.com/socket.io/websocket?type=chart".parse()?;

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
            .body(())?;

        // Get the response
        let mut stream = HttpClient::connect(&request).await?;
        let response = HttpClient::send::<(), String>(&mut stream, &request).await?;
        log::info!("response = {response:?}");

        // split
        let (reader, writer) = futures_lite::io::split(stream);
        let reader = BufReader::new(reader);
        let writer = BufWriter::new(writer);

        // create websocket client
        let ws_reader = WebSocketReader::new(reader);
        let ws_writer = WebSocketWriter::new(writer);        

        // Create the TradingViewClient
        let mut tv_reader = TradingViewReader::new(ws_reader);
        let mut tv_writer = TradingViewWriter::new(ws_writer);

        // prepare buffer + references
        let buffer: Vec<TradingViewMessageWrapper> = Vec::new();
        let buffer = RwLock::new(buffer);
        let buffer_arc = Arc::new(buffer);
        let reader_handle_buffer_ref = buffer_arc.clone();

        // Spawn the reader task
        let _reader_handle = std::thread::spawn(move || {
            futures_lite::future::block_on(async {
                loop {
                    match tv_reader.read_message().await {
                        Ok(result) => {
                            match result {
                                Some(message) => {
                                    // add message to buffer
                                    let mut write_lock = reader_handle_buffer_ref.write().await;
                                    write_lock.push(message);
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
        
        // Wait for server hello message with timeout
        let server_hello_message = utilities::run_with_timeout(Duration::from_secs(1), Box::pin(utilities::wait_for_message(buffer_arc.clone(), |message| message.payload.contains("javastudies"))))
            .await
            .expect("timed out")
            .expect("failed to get server hello message");
        log::info!("server_hello_message = {server_hello_message:?}");

        // set auth token
        tv_writer.set_auth_token(&self.config.auth_token).await?;
        
        // set locale
        tv_writer.set_locale("en", "US").await?;

        // handle chart sessions
        let mut index = 1;
        for chart_symbol in &self.config.chart_symbols {
            // create chart session
            let chart_session_id = format!("cs_{index:012}");

            // create chart session
            tv_writer.chart_create_session(&chart_session_id).await?;

            // resolve symbol
            let symbol_id = "sds_sym_1";
            tv_writer.resolve_symbol(&chart_session_id, symbol_id, &chart_symbol).await?;

            // wait for symbol resolved message
            let symbol_resolved_message = utilities::run_with_timeout(Duration::from_secs(1), Box::pin(utilities::wait_for_message(buffer_arc.clone(), |message| message.payload.contains("symbol_resolved"))))
                .await
                .expect("timed out")
                .expect("failed to get symbol resolved message");
            log::info!("symbol_resolved_message = {symbol_resolved_message:?}");

            // add symbol to chart session as series
            let series_id = "sds_1";
            tv_writer.create_series(&chart_session_id, series_id, "s1",  symbol_id, &self.config.timeframe, self.config.range).await?;

            // switch chart timezone
            tv_writer.switch_timezone(&chart_session_id, "exchange").await?;

            // wait for series loading message
            let series_loading_message = utilities::run_with_timeout(Duration::from_secs(1), Box::pin(utilities::wait_for_message(buffer_arc.clone(), |message| message.payload.contains("series_loading"))))
                .await
                .expect("timed out")
                .expect("failed to get series loading message");
            log::info!("series_loading_message = {series_loading_message:?}");

            // wait for timescale update message
            let timescale_update_message = utilities::run_with_timeout(Duration::from_secs(2), Box::pin(utilities::wait_for_message(buffer_arc.clone(), |message| message.payload.contains("timescale_update"))))
                .await
                .expect("timed out")
                .expect("failed to get timesale update message");
            log::info!("timescale_update_message = {timescale_update_message:?}");

            // wait for series completed message
            let series_completed_message = utilities::run_with_timeout(Duration::from_secs(1), Box::pin(utilities::wait_for_message(buffer_arc.clone(), |message| message.payload.contains("series_completed"))))
                .await
                .expect("timed out")
                .expect("failed to get series completed message");
            log::info!("series_completed_message = {series_completed_message:?}");

            // optionally create study session
            if self.config.indicators.len() > 0 {
                let study_session_id = "st1";
                tv_writer.create_study(&chart_session_id, study_session_id, "sessions_1", series_id, "Sessions@tv-basicstudies-241", "{}").await?;

                // wait for study loading message
                let study_loading_message = utilities::run_with_timeout(Duration::from_secs(1), Box::pin(utilities::wait_for_message(buffer_arc.clone(), |message| message.payload.contains("study_loading"))))
                    .await
                    .expect("timed out")
                    .expect("failed to get study loading message");
                log::info!("study_loading_message = {study_loading_message:?}");

                // wait for study completed message
                let study_completed_message = utilities::run_with_timeout(Duration::from_secs(1), Box::pin(utilities::wait_for_message(buffer_arc.clone(), |message| message.payload.contains("study_completed"))))
                    .await
                    .expect("timed out")
                    .expect("failed to get study completed message");
                log::info!("study_completed_message = {study_completed_message:?}");

                let mut index = 2;
                for indciator in &self.config.indicators {
                    let study_value = indciator;
                    let study_id = format!("st{index}");
                    tv_writer.create_study(&chart_session_id, &study_id, study_session_id, series_id, "Script@tv-scripting-101!", study_value).await?;
                    index += 1;

                    // wait for study loading message
                    let study_loading_message = utilities::run_with_timeout(Duration::from_secs(1), Box::pin(utilities::wait_for_message(buffer_arc.clone(), |message| message.payload.contains("study_loading"))))
                        .await
                        .expect("timed out")
                        .expect("failed to get study loading message");
                    log::info!("study_loading_message = {study_loading_message:?}");

                    // wait for study completed message
                    let study_completed_message = utilities::run_with_timeout(Duration::from_secs(1), Box::pin(utilities::wait_for_message(buffer_arc.clone(), |message| message.payload.contains("study_completed"))))
                        .await
                        .expect("timed out")
                        .expect("failed to get study completed message");
                    log::info!("study_completed_message = {study_completed_message:?}");
                }
            }

            // increment index
            index += 1;
        }

        // quote_symbol quote session
        let mut index = 1;
        for quote_symbol in &self.config.quote_symbols {
            // create quote session
            let quote_session_id = format!("qs_{index:012}");
            tv_writer.quote_create_session(&quote_session_id).await?;

            // set quote session fields
            tv_writer.quote_set_fields(&quote_session_id).await?;

            // add symbol to quote session
            tv_writer.quote_add_symbols(&quote_session_id, &quote_symbol).await?;

            // turn on quote fast symbols for quote session
            tv_writer.quote_fast_symbols(&quote_session_id, &quote_symbol).await?;

            // wait for quote completed message
            let quote_completed_message = utilities::run_with_timeout(Duration::from_secs(1), Box::pin(utilities::wait_for_message(buffer_arc.clone(), |message| message.payload.contains("quote_completed"))))
                .await
                .expect("timed out")
                .expect("failed to get quote completed message");
            log::info!("quote_completed_message = {quote_completed_message:?}");

            // increment index
            index += 1;
        }

        // request more data from series?
        /*for _ in 0..20 {
            tv_writer.request_more_data(chart_session_id1, series_id, 1000).await?;

            // TODO: wait for individual sries_loading / study_loading / study_completed messages

            async_io::Timer::after(Duration::from_millis(1000)).await;
        }*/

        // read all messages
        loop {
            let result = utilities::wait_for_message(buffer_arc.clone(), |_| true).await;
            match result {
                Some(message) => {
                    let parsed_message = ParsedTradingViewMessage::from_string(&message.payload)?;
                    match &parsed_message {
                        ParsedTradingViewMessage::Ping(nonce) => {
                            log::info!("ping nonce = {nonce}");
                            tv_writer.pong(*nonce).await?;
                        },
                        _ => {
                            // send to message processor
                            self.config.message_processor.process_message(self.config.name.clone(), parsed_message).await;
                        }
                    }
                },
                None => panic!("closed")
            }
        }
    }
}
