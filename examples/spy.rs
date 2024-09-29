#[cfg(not(any(feature = "futures", feature = "futures-lite")))]
compile_error!(
    "You must enable either the `futures` or `futures-lite` feature to build this crate."
);

#[cfg(feature = "futures")]
use futures as futures_provider;

#[cfg(feature = "futures-lite")]
use futures_lite as futures_provider;

use futures_provider::io::{BufReader, BufWriter};
use http::{Request, Uri, Version};
use http_client::HttpClient;
use websocket_client::{WebSocketReader, WebSocketWriter, WebSocketHelpers};
use tradingview_client::{TradingViewFrame, TradingViewPingFrame, TradingViewReader, TradingViewWriter};

use async_io::Timer;
use std::time::Duration;

async fn run_with_timeout<F, T>(timeout: Duration, future: F) -> Option<T>
where
    F: futures_lite::future::Future<Output = T> + Unpin,
{
    futures_lite::future::or(async { Some(future.await) }, async {
        Timer::after(timeout).await;
        None
    })
    .await
}

async fn wait_for_message<F, T>(
    frame_rx: &async_channel::Receiver<T>,
    buffer: &mut Vec<T>,
    condition: F,
) -> Option<T>
where
    F: Fn(&T) -> bool,
{
    loop {
        match frame_rx.recv().await {
            Ok(frame) => {
                if condition(&frame) {
                    return Some(frame);
                } else {
                    buffer.push(frame);
                }
            }
            Err(_) => return None, // Channel closed
        }
    }
}

fn main() {
    futures_provider::future::block_on(async {
        // init logging
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug,websocket_client=info,rustls=info,http_client=info")).init();
        
        // Build the URI for the request
        let uri: Uri = "wss://data.tradingview.com/socket.io/websocket?from=chart%2F&date=2024_09_25-14_09&type=chart".parse().expect("Failed to parse URI");

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
            //.header("Sec-WebSocket-Extensions", "permessage-deflate; client_max_window_bits")
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
        let (frame_tx, frame_rx) = async_channel::unbounded::<TradingViewFrame>();

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
        let server_hello_frame = run_with_timeout(Duration::from_secs(1), Box::pin(wait_for_message(&frame_rx, &mut buffer, |frame| frame.payload.contains("javastudies"))))
            .await
            .expect("timed out")
            .expect("failed to get server hello frame");
        log::info!("server_hello_frame = {server_hello_frame:?}");

        // set auth token
        tv_writer.set_auth_token("unauthorized_user_token").await.expect("failed to set auth token");
        
        // set locale
        tv_writer.set_locale("en", "US").await.expect("failed to set locale");

        // create chart session
        let chart_session_id = "cs_2AzYWhCHOwik";
        tv_writer.chart_create_session(chart_session_id).await.expect("failed to create chart session");

        // resolve symbol
        let symbol = r#"={\"adjustment\":\"splits\",\"symbol\":\"BINANCE:BTCUSDT\"}"#;
        //let symbol = r#"={\"adjustment\":\"splits\",\"currency-id\":\"USD\",\"session\":\"regular\",\"symbol\":\"AMEX:SPY\"}"#;
        let symbol_id = "sds_sym_1";
        tv_writer.resolve_symbol(chart_session_id, symbol_id, symbol).await.expect("failed to add symbol to resolve symbol");

        // switch chart timezone
        tv_writer.switch_timezone(chart_session_id, "exchange").await.expect("failed to switch chart timezone");

        // add symbol to chart session as series
        let series_id = "sds_1";
        let timeframe = "5";
        let range = 300;
        tv_writer.create_series(chart_session_id, series_id, "s1",  symbol_id, timeframe, range).await.expect("failed to create series");

        // TODO: wait for series loading frame
        let series_loading_frame = run_with_timeout(Duration::from_secs(1), Box::pin(wait_for_message(&frame_rx, &mut buffer, |frame| {
            log::info!("frame = {frame:?}");
            frame.payload.contains("series_loading")
        })))
            .await
            .expect("timed out")
            .expect("failed to get series loading frame");
        log::info!("series_loading_frame = {series_loading_frame:?}");

        // create quote session
        let quote_session_id1 = "qs_EaDCc5CHTQaa";
        tv_writer.quote_create_session(quote_session_id1).await.expect("failed to create quote session");

        // add symbol to quote session
        tv_writer.quote_add_symbols(quote_session_id1, symbol).await.expect("failed to add symbol to quote session");

        // create quote session
        let quote_session_id2 = "qs_CbGU6IdgHeyC";
        tv_writer.quote_create_session(quote_session_id2).await.expect("failed to create quote session");

        // set quote session fields
        tv_writer.quote_set_fields(quote_session_id2).await.expect("failed to set quote fields");

        // add symbol to quote session
        //let quote_symbol = "AEMX:SPY";
        let quote_symbol = "BINANCE:BTCUSDT";
        tv_writer.quote_add_symbols(quote_session_id2, quote_symbol).await.expect("failed to add symbol to quote session");

        // wait for quote session data frame
        let quote_session_data_frame = run_with_timeout(Duration::from_secs(1), Box::pin(wait_for_message(&frame_rx, &mut buffer, |frame| {
            log::info!("frame = {frame:?}");
            frame.payload.contains("qsd")
        })))
            .await
            .expect("timed out")
            .expect("failed to get quote session data frame");
        log::info!("quote_session_data_frame = {quote_session_data_frame:?}");

        // request more tickmarks from symbol
        /*let range = 10;
        tv_writer.request_more_tickmarks(chart_session_id, series_id, range).await.expect("failed to request more tickmarks");*/

        // turn on quote fast symbols for quote session
        tv_writer.quote_fast_symbols(quote_session_id1, symbol).await.expect("failed to turn on quote fast symbols");

        // create study session
        let study_session_id = "st1";
        tv_writer.create_study(chart_session_id, study_session_id, "sessions_1", series_id, "Sessions@tv-basicstudies-241", "{}").await.expect("failed to create study session");

        // add studies to study session
        let study_value = r#"{
            "text":"bmI9Ks46_4gc7PrwVT7kbVtIYR/9uGg==_z7csuIzdePr6JPnbZAwqsKwEUkTI7TrTZSgEmde35UKVbdqohGhUD1yCWiHpt+B1Q0vIrmanbmURoNk6YsXJIkATAAugCZIFBchkXEXEtHVYTL93KHjI70Y6xwlV7ajXNuA2vCc+i7Ir3NLpZEhOidaTK/p+FivGCrxfI3A6ooM4GZLFC0oEYkpcGCLSdP9IpSP9SKquOsBQmgpMoZ384QSD8qMK924eJbLjN1wSpjSp8LVHF+IqTWbYpx9ZlmXUU20bs6EY3EwAOG3qf40qdHoyPAL4UG6TrP5+V3h2I5CootDH13gZAtI75hdNhpJbUJDNAgvkKcVRDx6O8BIbmjzSeJ6C2+btsFxFmIFfcZHPie5dPPyAsd7ewSjmFVToSbvXw6KF+2y0+H3uk09hqj0a2F1F8WRJ15zmyCpuRNNHxOJtl3OatH2/MbJcKWn61/3bD+lY9HODKmnhLsZ8sZNF0uV2+QShPIlBARfnh3Nl8eUDQ+g4Z/2KZihDzb7hJZvQbkPAd/BDyXK7h4jvuZ0PBm07SxVaQapPfDrgeLJiimT9unatDTdgZNthoW7WoqwdtuENC76CEGiq3/llQYn7i/VAwaBMM/QnQBTlXdB7l+k=","pineId":"PUB;K07lTKE3tPla7glvOhmesGYj7Veuq4eX",
            "pineVersion":"1.0",
            "in_0":{"v":14,"f":true,"t":"integer"}
        }"#;
        tv_writer.create_study(chart_session_id, "st2", study_session_id, series_id, "Script@tv-scripting-101!", study_value).await.expect("failed to add to study session");

        // read all frames
        loop {
            let timeout_result = run_with_timeout(Duration::from_secs(1), Box::pin(wait_for_message(&frame_rx, &mut buffer, |_| true)))
                .await;
            match timeout_result {
                Some(frame_result) => {
                    match frame_result {
                        Some(frame) => {
                            log::info!("frame = {frame:?}");

                            // check if frame is ping frame
                            if let Ok((_, ping_frame)) = TradingViewPingFrame::parse(&frame.payload.as_bytes()) {
                                log::info!("ping_frame = {ping_frame:02x?}");
                                // respond to ping
                                tv_writer.pong(ping_frame.nonce).await.expect("failed to add to pong");
                            }
                        },
                        None => panic!("no match")
                    }
                },
                None => log::warn!("timed out"),
            }
        }
    })
}
