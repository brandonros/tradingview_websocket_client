# tradingview_websocket_client
Async TradingView WebSocket client for use with async_std

## How to get TradingView API token

1. Go to https://www.tradingview.com/ while logged in + pop developer console

2. `window.user.auth_token`

## How to use

```shell
# put AUTH_TOKEN="..." into .env file
cargo run --example multi_client
```
