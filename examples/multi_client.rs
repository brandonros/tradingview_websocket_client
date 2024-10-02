use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;

use tradingview_client::{ParsedTradingViewMessage, TradingViewClient, TradingViewMessageProcessor};
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

pub struct TradingViewClientConfig {
    name: String,
    auth_token: String,
    chart_symbol: String,
    quote_symbols: Vec<String>,
    indicators: Vec<String>,
    timeframe: String,
    range: usize,
    message_processor: Arc<Box<dyn TradingViewMessageProcessor + Send + Sync>>
}

impl TradingViewClientConfig {
    fn to_client(&self) -> TradingViewClient {
        TradingViewClient::new(
            self.name.to_string(),
            self.auth_token.to_string(),
            self.chart_symbol.to_string(),
            self.quote_symbols.clone(),
            self.indicators.clone(),
            self.timeframe.to_string(),
            self.range,
            self.message_processor.clone()
        )
    }
}
struct Indicators;

impl Indicators {
  pub fn generate_vwap_mvwap_ema_crossover(vwap_length: usize, ema1_source: String, ema1_length: usize, ema2_source: String, ema2_length: usize, rsi_limit: usize, rsi_minimum: usize, mvwap_length: usize) -> String {
    format!(r#"{{
      "text": "bmI9Ks46_14Oy1AFtjg8Ls9wU0S1rlg==_u70xwiBAuvwE8ScMuj3/xelBeUlPpaP443vgI0LOz0anO3Sz0Nml/Cw66rceMmOX/36sFmV/J8A9ocybTXK65SWNk5Mq5ULJ6IYlXtaoFYYsZRWpEMmaP9eq8c+j6BmHYcbh3XLrcNMUimL3emFm7ualhqyIU9Bit+n31nA898zBRSxB1+Jj5sHZ5cCUltgwmiCmbV6WhQoR6fRTVK5DXvgazVghDGv9ZF18/TpaZAnipKAZ1P59oNNL2e72XZQXWzWZlAbu7CHAtjyLv5RmO9bMBdsr2+Icd5cmGy+inNgtM4++cecagL5owwZhZGA/GRPyZ8UtjuvJesqiGPH+yqQEWtyfCnCjpvTV+tpDCn2SKcSQZyA87pNzAIi6/pspgUb01Sf2+wiJY+HuXAMKZQQ9zgD7oIvjjPaQqTBUgjVc0VMlQYX98yW3jzdOkaRXjKHxqSn0MXodjEBr1wQvH8sUv8Pvrttgdb7LVh/NFH4z8sQMRK7U7HB08M277TrUkz5Lak1OArmJ5vGF36Ty+Cw7nF3T2/t+LHecLwbIAzrtxR85m0fHMsZwwfW8z71w6/PuQnSZnlinambAWGDzUOAcc9CcXj9LRHsi9/wjRecaws1CUt1t4DI3oYsdMBcoGdx79k2a5qJT3aAYgpa1GTY3saW3RK5Lf8DasNK3srIlE6NyomS+pGhpBUpEFbd6iZL5o9G3iPUMHApZF3wXAHq78WxT+dnPUc/x3nnTmUK4IzsJnURj7jdi2Ko3LlC6OIO8o9/6knQPipTK7MMPG+sSJoFrfVaQiH6aXUMiTAspzHVmeoxZRFoi3J95HfXh+bOMbIwP62VmHgH0RhZzHWpUxIJof4iK/SIo3JVAQkt43JGyD8A0CzIgH2MVZmMV+rwe6URDCO63Vrs/6Fvz6QzPWbUmiXW5laTpBXJzM5mBrZD+M9Zso42rATUT6w3i23H2VE5kKbHG5p5kkyGM1c134cike1y5gyZDK3SMmnQyNgxUJKG0UpgXF2dnlQJpHXzya8dXco5QhldBd7TG33vKdKN5Ti/LMP6GJsZt6QC4CZWj0tWC8ow9ETVkiw0GGSLNUq818rG0EnWt9ZPVPu2dyT3gP/ZamMmmrKRWne12psNknznrqiH1ffDxdGGkJgVpda377gPVPYK5XrzyXvQKhNf7/xdAqN5DAiW5xpiUJ6GFcl3sgR35OBsFkFA=",
      "pineId": "PUB;N16MOYK6AEJGGAoy40axs0S48GRFYcNn",
      "pineVersion": "1.0",
      "in_0": {{
        "v": {vwap_length},
        "f": true,
        "t": "integer"
      }},
      "in_1": {{
        "v": "{ema1_source}",
        "f": true,
        "t": "source"
      }},
      "in_2": {{
        "v": {ema1_length},
        "f": true,
        "t": "integer"
      }},
      "in_3": {{
        "v": "{ema2_source}",
        "f": true,
        "t": "source"
      }},
      "in_4": {{
        "v": {ema2_length},
        "f": true,
        "t": "integer"
      }},
      "in_5": {{
        "v": {rsi_limit},
        "f": true,
        "t": "integer"
      }},
      "in_6": {{
        "v": {rsi_minimum},
        "f": true,
        "t": "integer"
      }},
      "in_7": {{
        "v": {mvwap_length},
        "f": true,
        "t": "integer"
      }}
    }}"#)
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
              Indicators::generate_vwap_mvwap_ema_crossover(
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
              Indicators::generate_vwap_mvwap_ema_crossover(
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
                let client = config.to_client();
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
