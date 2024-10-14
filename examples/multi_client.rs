use std::sync::Arc;
use std::time::Duration;

use smol_macros::Executor;
use tradingview_websocket_client::{DefaultTradingViewMessageProcessor, TradingViewClientConfig, TradingViewClientMode, TradingViewIndicators, TradingViewMessageProcessor, SPY5_EXT_SYMBOL, SPY5_REG_SYMBOL};

#[macro_rules_attribute::apply(smol_macros::main!)]
async fn main(executor: Arc<Executor<'static>>) -> anyhow::Result<()> {
    // init logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug,websocket_client=info,rustls=info,http_client=info")).init();

    // init env vars
    dotenvy::from_filename("./.env").expect("failed to load env vars");
    let auth_token = std::env::var("AUTH_TOKEN").expect("failed to get AUTH_TOKEN");

    // build message processor
    let message_processor1: Arc<Box<dyn TradingViewMessageProcessor + Send + Sync>> = Arc::new(Box::new(DefaultTradingViewMessageProcessor {}));
    let message_processor2: Arc<Box<dyn TradingViewMessageProcessor + Send + Sync>> = Arc::new(Box::new(DefaultTradingViewMessageProcessor {}));
        
    // build clients
    let vwap_mvwap_ema_crossover = TradingViewIndicators::generate_vwap_mvwap_ema_crossover(
      1,
      "close".to_string(),
      7,
      "close".to_string(),
      25,
      65,
      51,
      21
    );
    let clients = vec![
        TradingViewClientConfig {
            name: "SPY5REG".to_string(),
            auth_token: auth_token.clone(),
            chart_symbols: vec![SPY5_REG_SYMBOL.to_string()],
            quote_symbols: vec![SPY5_REG_SYMBOL.to_string()],
            indicators: vec![
              vwap_mvwap_ema_crossover.clone()
            ],
            timeframe: "5".to_string(),
            range: 300,
            mode: TradingViewClientMode::Streaming
        }.to_client(message_processor1),

        TradingViewClientConfig {
            name: "SPY5EXT".to_string(),
            auth_token: auth_token.clone(),
            chart_symbols: vec![SPY5_EXT_SYMBOL.to_string()],
            quote_symbols: vec![SPY5_EXT_SYMBOL.to_string()],
            indicators: vec![
              vwap_mvwap_ema_crossover.clone()
            ],
            timeframe: "5".to_string(),
            range: 300,
            mode: TradingViewClientMode::Streaming
        }.to_client(message_processor2),
    ];

    // spawn clients on threads
    let mut handles = vec![];
    for client in clients {
        let executor_clone = executor.clone();
        let handle = executor.spawn(async move {
            match client.run(executor_clone).await {
                Ok(_) => (),
                Err(err) => panic!("{err}"),
            }
        });
        handles.push(handle);
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
