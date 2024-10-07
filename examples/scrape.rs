use std::sync::Arc;

use tradingview_client::{DefaultTradingViewMessageProcessor, TradingViewClient, TradingViewClientConfig, TradingViewMessageProcessor};

fn main() {
    // init logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug,websocket_client=info,rustls=info,http_client=info")).init();

    // init env vars
    dotenvy::from_filename("./.env").expect("failed to load env vars");
    let auth_token = std::env::var("AUTH_TOKEN").expect("failed to get AUTH_TOKEN");

    // build message processor
    let message_processor1: Arc<Box<dyn TradingViewMessageProcessor + Send + Sync>> = Arc::new(Box::new(DefaultTradingViewMessageProcessor {}));

    // get symbol
    let args = std::env::args().collect::<Vec<_>>();
    let symbol = &args[1];
    let config = TradingViewClientConfig {
        name: symbol.to_string(),
        auth_token: auth_token.clone(),
        chart_symbols: vec![],
        quote_symbols: vec![symbol.to_string()],
        indicators: vec![],
        timeframe: "5".to_string(),
        range: 300,
        message_processor: message_processor1.clone()
    };

    // spawn client
    futures_lite::future::block_on(async {
        let client: TradingViewClient = config.to_client();
        match client.run().await {
            Ok(()) => (),
            Err(err) => panic!("{err}"),
        }     
    });
}
