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
    auth_token: String,
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
            self.auth_token.to_string(),
            self.chart_symbol.to_string(),
            self.quote_symbol.to_string(),
            self.indicators.clone(),
            self.timeframe.to_string(),
            self.range,
        )
    }
}

static VWAP_MVWAP_EMA_CROSSOVER: &str = r#"{
  "text": "bmI9Ks46_14Oy1AFtjg8Ls9wU0S1rlg==_u70xwiBAuvwE8ScMuj3/xelBeUlPpaP443vgI0LOz0anO3Sz0Nml/Cw66rceMmOX/36sFmV/J8A9ocybTXK65SWNk5Mq5ULJ6IYlXtaoFYYsZRWpEMmaP9eq8c+j6BmHYcbh3XLrcNMUimL3emFm7ualhqyIU9Bit+n31nA898zBRSxB1+Jj5sHZ5cCUltgwmiCmbV6WhQoR6fRTVK5DXvgazVghDGv9ZF18/TpaZAnipKAZ1P59oNNL2e72XZQXWzWZlAbu7CHAtjyLv5RmO9bMBdsr2+Icd5cmGy+inNgtM4++cecagL5owwZhZGA/GRPyZ8UtjuvJesqiGPH+yqQEWtyfCnCjpvTV+tpDCn2SKcSQZyA87pNzAIi6/pspgUb01Sf2+wiJY+HuXAMKZQQ9zgD7oIvjjPaQqTBUgjVc0VMlQYX98yW3jzdOkaRXjKHxqSn0MXodjEBr1wQvH8sUv8Pvrttgdb7LVh/NFH4z8sQMRK7U7HB08M277TrUkz5Lak1OArmJ5vGF36Ty+Cw7nF3T2/t+LHecLwbIAzrtxR85m0fHMsZwwfW8z71w6/PuQnSZnlinambAWGDzUOAcc9CcXj9LRHsi9/wjRecaws1CUt1t4DI3oYsdMBcoGdx79k2a5qJT3aAYgpa1GTY3saW3RK5Lf8DasNK3srIlE6NyomS+pGhpBUpEFbd6iZL5o9G3iPUMHApZF3wXAHq78WxT+dnPUc/x3nnTmUK4IzsJnURj7jdi2Ko3LlC6OIO8o9/6knQPipTK7MMPG+sSJoFrfVaQiH6aXUMiTAspzHVmeoxZRFoi3J95HfXh+bOMbIwP62VmHgH0RhZzHWpUxIJof4iK/SIo3JVAQkt43JGyD8A0CzIgH2MVZmMV+rwe6URDCO63Vrs/6Fvz6QzPWbUmiXW5laTpBXJzM5mBrZD+M9Zso42rATUT6w3i23H2VE5kKbHG5p5kkyGM1c134cike1y5gyZDK3SMmnQyNgxUJKG0UpgXF2dnlQJpHXzya8dXco5QhldBd7TG33vKdKN5Ti/LMP6GJsZt6QC4CZWj0tWC8ow9ETVkiw0GGSLNUq818rG0EnWt9ZPVPu2dyT3gP/ZamMmmrKRWne12psNknznrqiH1ffDxdGGkJgVpda377gPVPYK5XrzyXvQKhNf7/xdAqN5DAiW5xpiUJ6GFcl3sgR35OBsFkFA=",
  "pineId": "PUB;N16MOYK6AEJGGAoy40axs0S48GRFYcNn",
  "pineVersion": "1.0",
  "in_0": {
    "v": 1,
    "f": true,
    "t": "integer"
  },
  "in_1": {
    "v": "close",
    "f": true,
    "t": "source"
  },
  "in_2": {
    "v": 7,
    "f": true,
    "t": "integer"
  },
  "in_3": {
    "v": "close",
    "f": true,
    "t": "source"
  },
  "in_4": {
    "v": 25,
    "f": true,
    "t": "integer"
  },
  "in_5": {
    "v": 65,
    "f": true,
    "t": "integer"
  },
  "in_6": {
    "v": 51,
    "f": true,
    "t": "integer"
  },
  "in_7": {
    "v": 21,
    "f": true,
    "t": "integer"
  }
}"#;

fn main() {
    // init logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug,websocket_client=info,rustls=info,http_client=info")).init();

    // init env vars
    dotenvy::from_filename("./.env").expect("failed to load env vars");
        
    // build clients
    let clients = vec![
        TradingViewClientConfig {
            name: "BTC5".to_string(),
            auth_token: std::env::var("AUTH_TOKEN").expect("failed to get AUTH_TOKEN"),
            chart_symbol: r#"={\"adjustment\":\"splits\",\"symbol\":\"BINANCE:BTCUSDT\"}"#.to_string(),
            quote_symbol: "BINANCE:BTCUSDT".to_string(),
            indicators: vec![VWAP_MVWAP_EMA_CROSSOVER.to_string()],
            timeframe: "5".to_string(),
            range: 300,
        }.to_client(),
        
        /*TradingViewClientConfig {
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

        TradingViewClientConfig {
            name: "ES1".to_string(),
            chart_symbol: r#"={\"adjustment\":\"splits\",\"session\":\"regular\",\"symbol\":\"CME_MINI:ES1!\"}"#.to_string(),
            quote_symbol: "CME_MINI:ES1!".to_string(),
            indicators: vec![VOLUME_DELTA_14_INDICATOR.to_string()],
            timeframe: "5".to_string(),
            range: 300,
        }.to_client(),*/
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
        for handle in &handles {
            if handle.is_finished() {
                panic!("a handle finished");
            }
            std::thread::sleep(Duration::from_millis(1000));
        }
    }
}
