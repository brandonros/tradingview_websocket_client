use websocket_client::WebSocketWriter;
use futures_lite::io::AsyncWrite;

use crate::message_wrapper::TradingViewMessageWrapper;
use crate::types::Result;

/// TradingViewWriter handles writing TradingView messages.
pub struct TradingViewWriter<W>
where
    W: AsyncWrite + Unpin,
{
    ws_writer: WebSocketWriter<W>,
}

impl<W> TradingViewWriter<W>
where
    W: AsyncWrite + Unpin,
{
    /// Creates a new `TradingViewWriter` with the given `WebSocketWriter`.
    pub fn new(ws_writer: WebSocketWriter<W>) -> Self {
        Self { ws_writer }
    }

    /// Writes a message to the TradingView server.
    pub async fn write_message(&mut self, message: &str) -> Result<()> {
        let tv_message = TradingViewMessageWrapper::serialize(message);
        log::debug!("write_message: tv_message = {tv_message}");
        self.ws_writer.write_text_message(&tv_message).await
    }

    pub async fn set_auth_token(&mut self, auth_token: &str) -> Result<()> {
        let message = format!(r#"{{"m":"set_auth_token","p":["{auth_token}"]}}"#);
        self
            .write_message(&message)
            .await
    }

    pub async fn set_locale(&mut self, language_code: &str, region_code: &str) -> Result<()> {
        let message = format!(r#"{{"m":"set_locale","p":["{language_code}", "{region_code}"]}}"#);
        self
            .write_message(&message)
            .await
    }

    pub async fn chart_create_session(&mut self, chart_session_id: &str) -> Result<()> {
        let message = format!(r#"{{"m":"chart_create_session","p":["{chart_session_id}",""]}}"#);
        self
            .write_message(&message)
            .await
    }

    pub async fn switch_timezone(&mut self, chart_session_id: &str, timezone: &str) -> Result<()> {
        let message = format!(r#"{{"m":"switch_timezone","p":["{chart_session_id}","{timezone}"]}}"#);
        self
            .write_message(&message)
            .await
    }

    pub async fn quote_create_session(&mut self, quote_session_id: &str) -> Result<()> {
        let message = format!(r#"{{"m":"quote_create_session","p":["{quote_session_id}",""]}}"#);
        self
            .write_message(&message)
            .await
    }

    pub async fn quote_add_symbols(&mut self, quote_session_id: &str, symbol: &str) -> Result<()> {
        let message = format!(r#"{{"m":"quote_add_symbols","p":["{quote_session_id}","{symbol}"]}}"#);
        self
            .write_message(&message)
            .await
    }

    pub async fn resolve_symbol(&mut self, chart_session_id: &str, symbol_id: &str, symbol: &str) -> Result<()> {
        let message = format!(r#"{{"m":"resolve_symbol","p":["{chart_session_id}","{symbol_id}", "{symbol}"]}}"#);
        self
            .write_message(&message)
            .await
    }

    pub async fn create_series(&mut self, chart_session_id: &str, series_id: &str, unk1: &str, symbol_id: &str, timeframe: &str, range: usize) -> Result<()> {
        // ~m~81~m~{"m":"create_series","p":["cs_L2mu7VPJpvcr","sds_1","s1","sds_sym_1","5",300,""]}
        // ~m~81~m~{"m":"create_series","p":["cs_000000000001","sds_1","s1","sds_sym_1","5",300,""]}
        let message = format!(r#"{{"m":"create_series","p":["{chart_session_id}","{series_id}","{unk1}","{symbol_id}","{timeframe}",{range},""]}}"#);
        self
            .write_message(&message)
            .await
    }

    pub async fn request_more_tickmarks(&mut self, chart_session_id: &str, series_id: &str, range: usize) -> Result<()> {
        let message = format!(r#"{{"m":"request_more_tickmarks","p":["{chart_session_id}","{series_id}",{range}]}}"#);
        self
            .write_message(&message)
            .await
    }

    pub async fn request_more_data(&mut self, chart_session_id: &str, series_id: &str, amount: usize) -> Result<()> {
        let message = format!(r#"{{"m":"request_more_data","p":["{chart_session_id}","{series_id}",{amount}]}}"#);
        self
            .write_message(&message)
            .await
    }       

    pub async fn quote_fast_symbols(&mut self, quote_session_id: &str, symbol: &str) -> Result<()> {
        let message = format!(r#"{{"m":"quote_fast_symbols","p":["{quote_session_id}","{symbol}"]}}"#);
        self
            .write_message(&message)
            .await
    }   

    pub async fn quote_set_fields(&mut self, quote_session_id: &str) -> Result<()> {
        // TODO: make fields configurable
        let message = format!(r#"{{"m":"quote_set_fields","p":["{quote_session_id}","base-currency-logoid","ch","chp","currency-logoid","currency_code","currency_id","base_currency_id","current_session","description","exchange","format","fractional","is_tradable","language","local_description","listed_exchange","logoid","lp","lp_time","minmov","minmove2","original_name","pricescale","pro_name","short_name","type","typespecs","update_mode","volume","variable_tick_size","value_unit_id","unit_id","measure"]}}"#);
        self
            .write_message(&message)
            .await
    }  

    pub async fn create_study(&mut self, chart_session_id: &str, study_id: &str, session_id: &str, series_id: &str, name: &str, value: &str) -> Result<()> {
        // ~m~105~m~{"m":"create_study","p":["cs_L2mu7VPJpvcr","st1","sessions_1","sds_1","Sessions@tv-basicstudies-241",{}]}
        // ~m~1739~m~{"m":"create_study","p":["cs_L2mu7VPJpvcr","st2","st1","sds_1","Script@tv-scripting-101!",{"text":"bmI9Ks46_14Oy1AFtjg8Ls9wU0S1rlg==_u70xwiBAuvwE8ScMuj3/xelBeUlPpaP443vgI0LOz0anO3Sz0Nml/Cw66rceMmOX/36sFmV/J8A9ocybTXK65SWNk5Mq5ULJ6IYlXtaoFYYsZRWpEMmaP9eq8c+j6BmHYcbh3XLrcNMUimL3emFm7ualhqyIU9Bit+n31nA898zBRSxB1+Jj5sHZ5cCUltgwmiCmbV6WhQoR6fRTVK5DXvgazVghDGv9ZF18/TpaZAnipKAZ1P59oNNL2e72XZQXWzWZlAbu7CHAtjyLv5RmO9bMBdsr2+Icd5cmGy+inNgtM4++cecagL5owwZhZGA/GRPyZ8UtjuvJesqiGPH+yqQEWtyfCnCjpvTV+tpDCn2SKcSQZyA87pNzAIi6/pspgUb01Sf2+wiJY+HuXAMKZQQ9zgD7oIvjjPaQqTBUgjVc0VMlQYX98yW3jzdOkaRXjKHxqSn0MXodjEBr1wQvH8sUv8Pvrttgdb7LVh/NFH4z8sQMRK7U7HB08M277TrUkz5Lak1OArmJ5vGF36Ty+Cw7nF3T2/t+LHecLwbIAzrtxR85m0fHMsZwwfW8z71w6/PuQnSZnlinambAWGDzUOAcc9CcXj9LRHsi9/wjRecaws1CUt1t4DI3oYsdMBcoGdx79k2a5qJT3aAYgpa1GTY3saW3RK5Lf8DasNK3srIlE6NyomS+pGhpBUpEFbd6iZL5o9G3iPUMHApZF3wXAHq78WxT+dnPUc/x3nnTmUK4IzsJnURj7jdi2Ko3LlC6OIO8o9/6knQPipTK7MMPG+sSJoFrfVaQiH6aXUMiTAspzHVmeoxZRFoi3J95HfXh+bOMbIwP62VmHgH0RhZzHWpUxIJof4iK/SIo3JVAQkt43JGyD8A0CzIgH2MVZmMV+rwe6URDCO63Vrs/6Fvz6QzPWbUmiXW5laTpBXJzM5mBrZD+M9Zso42rATUT6w3i23H2VE5kKbHG5p5kkyGM1c134cike1y5gyZDK3SMmnQyNgxUJKG0UpgXF2dnlQJpHXzya8dXco5QhldBd7TG33vKdKN5Ti/LMP6GJsZt6QC4CZWj0tWC8ow9ETVkiw0GGSLNUq818rG0EnWt9ZPVPu2dyT3gP/ZamMmmrKRWne12psNknznrqiH1ffDxdGGkJgVpda377gPVPYK5XrzyXvQKhNf7/xdAqN5DAiW5xpiUJ6GFcl3sgR35OBsFkFA=","pineId":"PUB;N16MOYK6AEJGGAoy40axs0S48GRFYcNn","pineVersion":"1.0","in_0":{"v":1,"f":true,"t":"integer"},"in_1":{"v":"close","f":true,"t":"source"},"in_2":{"v":7,"f":true,"t":"integer"},"in_3":{"v":"close","f":true,"t":"source"},"in_4":{"v":25,"f":true,"t":"integer"},"in_5":{"v":65,"f":true,"t":"integer"},"in_6":{"v":51,"f":true,"t":"integer"},"in_7":{"v":21,"f":true,"t":"integer"}}]}
        let message = format!(r#"{{
            "m":"create_study",
            "p":[
                "{chart_session_id}",
                "{study_id}",
                "{session_id}",
                "{series_id}",
                "{name}",
                {value}
            ]
        }}"#);
        self
            .write_message(&message)
            .await
    }   

    pub async fn pong(&mut self, nonce: usize) -> Result<()> {
        let message = format!("~h~{nonce}");
        self
            .write_message(&message)
            .await
    }
}
