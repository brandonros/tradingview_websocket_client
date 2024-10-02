use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;

use tradingview_client::{ParsedTradingViewMessage, TradingViewClient, TradingViewClientConfig, TradingViewIndicators, TradingViewMessageProcessor};
struct TradingViewMessageProcessorImpl;

#[async_trait]
impl TradingViewMessageProcessor for TradingViewMessageProcessorImpl {
  async fn process_message(&self, name: String, message: ParsedTradingViewMessage) ->() {
    match message {
      ParsedTradingViewMessage::Ping(nonce) => {
        log::info!("[{name}] ping nonce = {nonce:?}");
      },
      ParsedTradingViewMessage::ServerHello(server_hello_message) => {
        log::info!("[{name}] server_hello_message = {server_hello_message:?}");
      },
      ParsedTradingViewMessage::QuoteSeriesData(quote_series_data_message) => {
        //log::info!("[{name}] quote_series_data_message = {quote_series_data_message:?}");
        let quote_session_id = quote_series_data_message.quote_session_id;
        let quote_update = quote_series_data_message.quote_update;
        log::info!("[{name}:{quote_session_id}] quote_update = {quote_update:?}");
      },
      ParsedTradingViewMessage::DataUpdate(data_update_message) => {
        //log::info!("[{name}] data_update_message = {data_update_message:?}");
        let chart_session_id = data_update_message.chart_session_id;
        let update_key = data_update_message.update_key;
        if let Some(series_updates) = data_update_message.series_updates {
          for series_update in series_updates {
            log::info!("[{name}:{chart_session_id}:{update_key}] series_update = {series_update:?}");
          }
        }
        if let Some(study_updates) = data_update_message.study_updates {
          for study_update in study_updates {
            log::info!("[{name}:{chart_session_id}:{update_key}] study_update = {study_update:?}");
          }
        }
      },
      ParsedTradingViewMessage::QuoteCompleted(quote_completed_message) => {
        log::info!("[{name}] quote_completed_message = {quote_completed_message:?}");
      },
      ParsedTradingViewMessage::TimescaleUpdate(timescale_updated_message) => {
        log::info!("[{name}] timescale_updated_message = {timescale_updated_message:?}");
      },
      ParsedTradingViewMessage::SeriesLoading(series_loading_message) => {
        log::info!("[{name}] series_loading_message = {series_loading_message:?}");
      },
      ParsedTradingViewMessage::SymbolResolved(symbol_resolved_message) => {
        log::info!("[{name}] symbol_resolved_message = {symbol_resolved_message:?}");
      },
      ParsedTradingViewMessage::SeriesCompleted(series_completed_message) => {
        log::info!("[{name}] series_completed_message = {series_completed_message:?}");
      },
      ParsedTradingViewMessage::StudyLoading(study_loading_message) => {
        log::info!("[{name}] study_loading_message = {study_loading_message:?}");
      },
      ParsedTradingViewMessage::StudyError(study_error_message) => {
        log::info!("[{name}] study_error_message = {study_error_message:?}");
      },
      ParsedTradingViewMessage::StudyCompleted(study_completed_message) => {
        log::info!("[{name}] study_completed_message = {study_completed_message:?}");
      },
      ParsedTradingViewMessage::TickmarkUpdate(tickmark_update_message) => {
        log::info!("[{name}] tickmark_update_message = {tickmark_update_message:?}");
      },
      ParsedTradingViewMessage::CriticalError(critical_error_message) => {
        log::info!("[{name}] critical_error_message = {critical_error_message:?}");
      },
      ParsedTradingViewMessage::ProtocolError(protocol_error_message) => {
        log::info!("[{name}] protocol_error_message = {protocol_error_message:?}");
      },
      ParsedTradingViewMessage::NotifyUser(notify_user_message) => {
        log::info!("[{name}] notify_user_message = {notify_user_message:?}");
      },
    }
  }
}

fn main() {
    // init logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug,websocket_client=info,rustls=info,http_client=info")).init();

    // init env vars
    dotenvy::from_filename("./.env").expect("failed to load env vars");
    let auth_token = std::env::var("AUTH_TOKEN").expect("failed to get AUTH_TOKEN");

    // build message processor
    let message_processor1: Arc<Box<dyn TradingViewMessageProcessor + Send + Sync>> = Arc::new(Box::new(TradingViewMessageProcessorImpl {}));
    let message_processor2: Arc<Box<dyn TradingViewMessageProcessor + Send + Sync>> = Arc::new(Box::new(TradingViewMessageProcessorImpl {}));
        
    // build clients
    let configs = vec![
        TradingViewClientConfig {
            name: "SPY5REG".to_string(),
            auth_token: auth_token.clone(),
            chart_symbol: r#"={\"adjustment\":\"splits\",\"currency-id\":\"USD\",\"session\":\"regular\",\"symbol\":\"AMEX:SPY\"}"#.to_string(),
            quote_symbols: vec![r#"={\"adjustment\":\"splits\",\"currency-id\":\"USD\",\"session\":\"regular\",\"symbol\":\"AMEX:SPY\"}"#.to_string()],
            indicators: vec![
              TradingViewIndicators::generate_vwap_mvwap_ema_crossover(
                1,
                "close".to_string(),
                7,
                "close".to_string(),
                25,
                65,
                51,
                21
              )
            ],
            timeframe: "5".to_string(),
            range: 300,
            message_processor: message_processor1.clone()
        },

        TradingViewClientConfig {
            name: "SPY5EXT".to_string(),
            auth_token: auth_token.clone(),
            chart_symbol: r#"={\"adjustment\":\"splits\",\"currency-id\":\"USD\",\"session\":\"extended\",\"symbol\":\"AMEX:SPY\"}"#.to_string(),
            quote_symbols: vec![r#"={\"adjustment\":\"splits\",\"currency-id\":\"USD\",\"session\":\"extended\",\"symbol\":\"AMEX:SPY\"}"#.to_string()],
            indicators: vec![
              TradingViewIndicators::generate_vwap_mvwap_ema_crossover(
                1,
                "close".to_string(),
                7,
                "close".to_string(),
                25,
                65,
                51,
                21
              )
            ],
            timeframe: "5".to_string(),
            range: 300,
            message_processor: message_processor2.clone()
        },
    ];

    // spawn clients on threads
    let mut handles = vec![];
    for config in configs {
        handles.push(std::thread::spawn(move || {
            futures_lite::future::block_on(async {
                let client: TradingViewClient = config.to_client();
                match client.run().await {
                    Ok(()) => (),
                    Err(err) => panic!("{err}"),
                }
            })
        }));
    }

    // watch handles
    loop {
        for handle in &handles {
            if handle.is_finished() {
                panic!("a handle finished");
            }
            std::thread::sleep(Duration::from_millis(1000));
        }
    }
}
