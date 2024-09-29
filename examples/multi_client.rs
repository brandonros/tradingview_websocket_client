#[cfg(not(any(feature = "futures", feature = "futures-lite")))]
compile_error!(
    "You must enable either the `futures` or `futures-lite` feature to build this crate."
);

use std::time::Duration;

#[cfg(feature = "futures")]
use futures as futures_provider;

#[cfg(feature = "futures-lite")]
use futures_lite as futures_provider;

use tradingview_client::TradingViewClient;

struct TradingViewClientConfig {
    name: String,
    chart_symbol: String,
    quote_symbol: String,
    indicators: Vec<String>,
    timeframe: String,
    range: usize,
}

impl TradingViewClientConfig {
    fn to_client(&self) -> TradingViewClient {
        TradingViewClient::new(
            self.name.to_string(),
            self.chart_symbol.to_string(),
            self.quote_symbol.to_string(),
            self.indicators.clone(),
            self.timeframe.to_string(),
            self.range,
        )
    }
}

static VOLUME_DELTA_14_INDICATOR: &str = r#"{
    "text":"bmI9Ks46_4gc7PrwVT7kbVtIYR/9uGg==_z7csuIzdePr6JPnbZAwqsKwEUkTI7TrTZSgEmde35UKVbdqohGhUD1yCWiHpt+B1Q0vIrmanbmURoNk6YsXJIkATAAugCZIFBchkXEXEtHVYTL93KHjI70Y6xwlV7ajXNuA2vCc+i7Ir3NLpZEhOidaTK/p+FivGCrxfI3A6ooM4GZLFC0oEYkpcGCLSdP9IpSP9SKquOsBQmgpMoZ384QSD8qMK924eJbLjN1wSpjSp8LVHF+IqTWbYpx9ZlmXUU20bs6EY3EwAOG3qf40qdHoyPAL4UG6TrP5+V3h2I5CootDH13gZAtI75hdNhpJbUJDNAgvkKcVRDx6O8BIbmjzSeJ6C2+btsFxFmIFfcZHPie5dPPyAsd7ewSjmFVToSbvXw6KF+2y0+H3uk09hqj0a2F1F8WRJ15zmyCpuRNNHxOJtl3OatH2/MbJcKWn61/3bD+lY9HODKmnhLsZ8sZNF0uV2+QShPIlBARfnh3Nl8eUDQ+g4Z/2KZihDzb7hJZvQbkPAd/BDyXK7h4jvuZ0PBm07SxVaQapPfDrgeLJiimT9unatDTdgZNthoW7WoqwdtuENC76CEGiq3/llQYn7i/VAwaBMM/QnQBTlXdB7l+k=",
    "pineId":"PUB;K07lTKE3tPla7glvOhmesGYj7Veuq4eX",
    "pineVersion":"1.0",
    "in_0":{"v":14,"f":true,"t":"integer"}
}"#;

fn main() {
    // init logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug,websocket_client=info,rustls=info,http_client=info")).init();
        
    // build clients
    let clients = vec![
        TradingViewClientConfig {
            name: "BTC5".to_string(),
            chart_symbol: r#"={\"adjustment\":\"splits\",\"symbol\":\"BINANCE:BTCUSDT\"}"#.to_string(),
            quote_symbol: "BINANCE:BTCUSDT".to_string(),
            indicators: vec![VOLUME_DELTA_14_INDICATOR.to_string()],
            timeframe: "5".to_string(),
            range: 300,
        }.to_client(),
        
        TradingViewClientConfig {
            name: "BONK5".to_string(),
            chart_symbol: r#"={\"adjustment\":\"splits\",\"symbol\":\"BINANCE:BONKUSDT\"}"#.to_string(),
            quote_symbol: "BINANCE:BONKUSDT".to_string(),
            indicators: vec![VOLUME_DELTA_14_INDICATOR.to_string()],
            timeframe: "5".to_string(),
            range: 300,
        }.to_client(),

        TradingViewClientConfig {
            name: "SPY5".to_string(),
            chart_symbol: r#"={\"adjustment\":\"splits\",\"currency-id\":\"USD\",\"session\":\"regular\",\"symbol\":\"AMEX:SPY\"}"#.to_string(),
            quote_symbol: "AMEX:SPY".to_string(),
            indicators: vec![VOLUME_DELTA_14_INDICATOR.to_string()],
            timeframe: "5".to_string(),
            range: 300,
        }.to_client(),
    ];

    // spawn clients on threads
    let mut handles = vec![];
    for client in clients {
        handles.push(std::thread::spawn(move || {
            futures_provider::future::block_on(async {
                client.run().await
            })
        }));
    }

    // watch handles
    loop {
        for handle in handles {
            if handle.is_finished() {
                panic!("a handle finished");
            }
            std::thread::sleep(Duration::from_millis(1000));
        }
    }
}
