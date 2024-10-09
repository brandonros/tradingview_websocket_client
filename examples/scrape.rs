use std::sync::Arc;

use tradingview_client::{DefaultTradingViewMessageProcessor, TradingViewClient, TradingViewClientConfig, TradingViewClientMode, TradingViewMessageProcessor};

fn main() {
    // init logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug,websocket_client=info,rustls=info,http_client=info")).init();

    // init env vars
    dotenvy::from_filename("./.env").expect("failed to load env vars");
    let auth_token = std::env::var("AUTH_TOKEN").expect("failed to get AUTH_TOKEN");

    // build message processor
    let message_processor: Arc<Box<dyn TradingViewMessageProcessor + Send + Sync>> = Arc::new(Box::new(DefaultTradingViewMessageProcessor {}));

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
        mode: TradingViewClientMode::Standard
    };
    let client: TradingViewClient = config.to_client(message_processor);

    // spawn client
    let scrape_result = futures_lite::future::block_on(async {
        match client.run().await {
            Ok(scrape_result) => scrape_result,
            Err(err) => panic!("{err}"),
        }     
    });

    log::info!("scrape_result = {scrape_result:?}");
}
