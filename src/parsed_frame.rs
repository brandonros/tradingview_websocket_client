#![allow(dead_code)]

use miniserde::json::{Array, Number, Object, Value};

use crate::types::Result;

fn value_to_string(input: &Value) -> Result<String> {
    match input {
        Value::String(value) => Ok(value.clone()),
        _ => Err("parsing failed".into())
    }
}

fn value_to_array(input: &Value) -> Result<Array> {
    match input {
        Value::Array(value) => Ok(value.clone()),
        _ => Err("parsing failed".into())
    }
}

fn value_to_object(input: &Value) -> Result<Object> {
    match input {
        Value::Object(value) => Ok(value.clone()),
        _ => Err("parsing failed".into())
    }
}

fn value_to_number(input: &Value) -> Result<Number> {
    match input {
        Value::Number(value) => Ok(value.clone()),
        _ => Err("parsing failed".into())
    }
}

fn value_to_bool(input: &Value) -> Result<bool> {
    match input {
        Value::Bool(value) => Ok(value.clone()),
        _ => Err("parsing failed".into())
    }
}

fn is_null(input: &Object, key: &str) -> Result<bool> {
    let value = input.get(key).ok_or("failed to get key")?;
    match value {
        Value::Null => Ok(true),
        _ => Ok(false),
    }
}

#[derive(Debug, Clone)]
pub struct QuoteSeriesDataUpdate {
    pub symbol: String,
    pub volume: Option<Number>,
    pub ch: Option<Number>,
    pub chp: Option<Number>,
    pub rch: Option<Number>,
    pub rchp: Option<Number>,
    pub rtc: Option<Number>,
    pub rtc_time: Option<Number>,
    pub lp: Option<Number>,
    pub lp_time: Option<Number>,
    pub ask: Option<Number>,
    pub ask_size: Option<Number>,
    pub bid: Option<Number>,
    pub bid_size: Option<Number>,
    pub trade_loaded: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct QuoteSeriesDataFrame {
    pub quote_session_id: String,
    pub update: QuoteSeriesDataUpdate,
}

#[derive(Debug, Clone)]
pub struct DataUpdateFrame {
    pub chart_session_id: String,
    pub update_key: String,
    pub series_updates: Option<Array>,
    pub study_updates: Option<Array>,
}

#[derive(Debug, Clone)]
pub struct QuoteCompletedFrame {
    pub quote_session_id: String,
    pub symbol: String,
}

#[derive(Debug, Clone)]
pub struct TimescaleUpdate {
    pub index: Number,
    pub timestamp: Number,
    pub open: Number,
    pub high: Number,
    pub low: Number,
    pub close: Number,
}

#[derive(Debug, Clone)]
pub struct TimescaleUpdatedFrame {
    pub chart_session_id: String,
    pub update_key: Option<String>,
    pub updates: Option<Vec<TimescaleUpdate>>
}

#[derive(Debug, Clone)]
pub struct ServerHelloFrame {

}

#[derive(Debug, Clone)]
pub struct SeriesLoadingFrame {

}

#[derive(Debug, Clone)]
pub struct SymbolResolvedFrame {

}

#[derive(Debug, Clone)]
pub struct SeriesCompletedFrame {

}

#[derive(Debug, Clone)]
pub struct StudyLoadingFrame {

}

#[derive(Debug, Clone)]
pub struct StudyErrorFrame {

}

#[derive(Debug, Clone)]
pub struct StudyCompletedFrame {

}

#[derive(Debug, Clone)]
pub struct TickmarkUpdateFrame {

}

#[derive(Debug, Clone)]
pub enum ParsedTradingViewFrame {
    ServerHello(ServerHelloFrame),
    Ping(usize),
    QuoteSeriesData(QuoteSeriesDataFrame),
    DataUpdate(DataUpdateFrame),
    QuoteCompleted(QuoteCompletedFrame),
    TimescaleUpdate(TimescaleUpdatedFrame),
    SeriesLoading(SeriesLoadingFrame),
    SymbolResolved(SymbolResolvedFrame),
    SeriesCompleted(SeriesCompletedFrame),
    StudyLoading(StudyLoadingFrame),
    StudyError(StudyErrorFrame),
    StudyCompleted(StudyCompletedFrame),
    TickmarkUpdate(TickmarkUpdateFrame),
}

impl ParsedTradingViewFrame {
    pub fn from_string(value: &str) -> Result<Self> {
        // ping frames are not json
        if value.starts_with("~h~") {
            let nonce_str = &value[3..];
            let nonce = nonce_str.parse::<usize>().map_err(|_| "failed to parse nonce")?;
            return Ok(ParsedTradingViewFrame::Ping(nonce));
        }

        // all other frames are json
        let parsed_frame: miniserde::json::Object = miniserde::json::from_str(&value)?;

        // check for server hello frame
        if parsed_frame.contains_key("javastudies") {
            return Ok(ParsedTradingViewFrame::ServerHello(ServerHelloFrame {  

            }));
        }
        
        // all other frames have m property
        let frame_type = parsed_frame.get("m").ok_or("failed to get frame_type")?;
        let frame_type = value_to_string(frame_type)?;
        if frame_type == "qsd" {
            //log::info!("qsd = {parsed_frame:?}");
            let p = parsed_frame.get("p").ok_or("failed to get p")?;
            let p = value_to_array(p)?;
            let quote_session_id = &p[0];
            let quote_session_id = value_to_string(&quote_session_id)?;
            let update = &p[1];
            let update = value_to_object(&update)?;
            let symbol = value_to_string(update.get("n").ok_or("failed to get n")?)?;
            let v = value_to_object(update.get("v").ok_or("failed to get v")?)?;
            //let v_keys = v.keys().collect::<Vec<&String>>();
            // TODO: check more combinations
            let quote_series_data_update = QuoteSeriesDataUpdate {
                symbol,

                volume: if v.contains_key("volume") { Some(value_to_number(v.get("volume").ok_or("failed to get v")?)?) } else { None },

                ch: if v.contains_key("ch") { Some(value_to_number(v.get("ch").ok_or("failed to get ch")?)?) } else { None },
                chp: if v.contains_key("chp") { Some(value_to_number(v.get("chp").ok_or("failed to get chp")?)?) } else { None },

                rch: if v.contains_key("rch") && !is_null(&v, "rch")? { Some(value_to_number(v.get("rch").ok_or("failed to get rch")?)?) } else { None },
                rchp: if v.contains_key("rchp") && !is_null(&v, "rchp")? { Some(value_to_number(v.get("rchp").ok_or("failed to get rchp")?)?) } else { None },

                lp: if v.contains_key("lp") { Some(value_to_number(v.get("lp").ok_or("failed to get lp")?)?) } else { None },
                lp_time: if v.contains_key("lp_time") { Some(value_to_number(v.get("lp_time").ok_or("failed to get lp_time")?)?) } else { None },

                rtc: if v.contains_key("rtc") && !is_null(&v, "rtc")? { Some(value_to_number(v.get("rtc").ok_or("failed to get rtc")?)?) } else { None },
                rtc_time: if v.contains_key("rtc_time") && !is_null(&v, "rtc_time")? { Some(value_to_number(v.get("rtc_time").ok_or("failed to get rtc_time")?)?) } else { None },

                ask: if v.contains_key("ask") { Some(value_to_number(v.get("ask").ok_or("failed to get ask")?)?) } else { None },
                ask_size: if v.contains_key("ask_size") { Some(value_to_number(v.get("ask_size").ok_or("failed to get ask_size")?)?) } else { None },

                bid: if v.contains_key("bid") { Some(value_to_number(v.get("bid").ok_or("failed to get bid")?)?) } else { None },
                bid_size: if v.contains_key("bid_size") { Some(value_to_number(v.get("bid_size").ok_or("failed to get bid_size")?)?) } else { None },

                trade_loaded: if v.contains_key("trade_loaded") { Some(value_to_bool(v.get("trade_loaded").ok_or("failed to get trade_loaded")?)?) } else { None },

                // TODO: more fields?
            };
            Ok(ParsedTradingViewFrame::QuoteSeriesData(QuoteSeriesDataFrame {
                quote_session_id,
                update: quote_series_data_update
            }))
        } else if frame_type == "du" {
            //log::info!("du = {parsed_frame:?}");   
            let p = parsed_frame.get("p").ok_or("failed to get p")?;
            let p = value_to_array(p)?;
            let chart_session_id = &p[0];
            let chart_session_id = value_to_string(&chart_session_id)?;
            let update = &p[1];
            let update = value_to_object(&update)?;
            let update_keys = update.keys().collect::<Vec<&String>>();
            assert!(update_keys.len() == 1);
            let update_key = update_keys[0];
            if update_key == "sds_1" { // series
                let update_value = value_to_object(update.get(update_key).ok_or("failed to get update_key")?)?;
                if update_value.contains_key("s") {
                    let s = update_value.get("s").ok_or("failed to get s")?;
                    let s = value_to_array(s)?;
                    Ok(ParsedTradingViewFrame::DataUpdate(DataUpdateFrame {
                        chart_session_id,
                        update_key: update_key.to_string(),
                        series_updates: Some(s),
                        study_updates: None
                    }))
                } else {
                    // watch out for weird du frame with no updates on it? ns property
                    Ok(ParsedTradingViewFrame::DataUpdate(DataUpdateFrame {
                        chart_session_id,
                        update_key: update_key.to_string(),
                        series_updates: None,
                        study_updates: None
                    }))
                }
            } else if update_key == "st1" || update_key == "st2" { // study
                let update_value = value_to_object(update.get(update_key).ok_or("failed to get update_key")?)?;
                let st = update_value.get("st").ok_or("failed to get st")?;
                let st = value_to_array(st)?;
                Ok(ParsedTradingViewFrame::DataUpdate(DataUpdateFrame {
                    chart_session_id,
                    update_key: update_key.to_string(),
                    series_updates: None,
                    study_updates: Some(st)
                }))
            } else {
                todo!("update_key = {update_key}");
            }
        } else if frame_type == "quote_completed" {
            //log::info!("quote_completed = {parsed_frame:?}"); 
            let p = parsed_frame.get("p").ok_or("failed to get p")?;
            let p = value_to_array(p)?;
            let quote_session_id = &p[0];
            let quote_session_id = value_to_string(&quote_session_id)?;
            let symbol = &p[1];
            let symbol = value_to_string(&symbol)?;
            Ok(ParsedTradingViewFrame::QuoteCompleted(QuoteCompletedFrame {
                quote_session_id,
                symbol
            }))
        } else if frame_type == "timescale_update" {
            //log::info!("timescale_update parsed_frame = {parsed_frame:?}");    
            let p = parsed_frame.get("p").ok_or("failed to get p")?;
            let p = value_to_array(p)?;
            let chart_session_id = &p[0];
            let chart_session_id = value_to_string(&chart_session_id)?;
            let update = &p[1];
            let update = value_to_object(&update)?;
            let update_keys = update.keys().collect::<Vec<&String>>();
            if update_keys.len() == 0 {
                // weird timescale_update with index/zoffset/changes/marks but nothing of any interest/importance
                Ok(ParsedTradingViewFrame::TimescaleUpdate(TimescaleUpdatedFrame {
                    chart_session_id,
                    update_key: None,
                    updates: None
                }))
            } else if update_keys.len() == 1 {
                let update_key = update_keys[0];
                let update_value = value_to_object(update.get(update_key).ok_or("failed to get update_key")?)?;
                let s = update_value.get("s").ok_or("failed to get s")?;
                let s = value_to_array(s)?;
                let timescale_updates = s.iter().map(|element| {
                    // value -> object
                    let element = value_to_object(&element).expect("failed to cast");
                    
                    // pluck i (index)
                    let i = element.get("i").expect("failed to get i");
                    let i = value_to_number(i).expect("failed to cast");

                    // pluck v (values)
                    let v = element.get("v").expect("failed to get v");
                    let v = value_to_array(v).expect("failed to cast");

                    // pluck out of values
                    let timestamp = value_to_number(&v[0]).expect("failed to cast");
                    let open = value_to_number(&v[1]).expect("failed to cast");
                    let high = value_to_number(&v[2]).expect("failed to cast");
                    let low = value_to_number(&v[3]).expect("failed to cast");
                    let close = value_to_number(&v[4]).expect("failed to cast");

                    // return
                    TimescaleUpdate {
                        index: i,
                        timestamp,
                        open,
                        high,
                        low,
                        close,
                    }
                }).collect::<Vec<_>>();
                Ok(ParsedTradingViewFrame::TimescaleUpdate(TimescaleUpdatedFrame {
                    chart_session_id,
                    update_key: Some(update_key.to_string()),
                    updates: Some(timescale_updates)
                }))
            } else {
                unimplemented!()
            }
        } else if frame_type == "series_loading" {
            log::info!("series_loading = {parsed_frame:?}");                        
            Ok(ParsedTradingViewFrame::SeriesLoading(SeriesLoadingFrame {
                
            }))
        } else if frame_type == "symbol_resolved" {
            log::info!("symbol_resolved = {parsed_frame:?}");                        
            Ok(ParsedTradingViewFrame::SymbolResolved(SymbolResolvedFrame {
                
            }))
        } else if frame_type == "series_completed" {
            log::info!("series_completed = {parsed_frame:?}");                        
            Ok(ParsedTradingViewFrame::SeriesCompleted(SeriesCompletedFrame {
                
            }))
        } else if frame_type == "study_loading" {
            log::info!("study_loading = {parsed_frame:?}");                        
            Ok(ParsedTradingViewFrame::StudyLoading(StudyLoadingFrame {
                
            }))
        } else if frame_type == "study_error" {
            log::info!("study_error = {parsed_frame:?}");                        
            Ok(ParsedTradingViewFrame::StudyError(StudyErrorFrame {
                
            }))
        } else if frame_type == "study_completed" {
            log::info!("study_completed = {parsed_frame:?}");                        
            Ok(ParsedTradingViewFrame::StudyCompleted(StudyCompletedFrame {
                
            }))
        } else if frame_type == "tickmark_update" {
            log::info!("tickmark_update = {parsed_frame:?}");                        
            Ok(ParsedTradingViewFrame::TickmarkUpdate(TickmarkUpdateFrame {
                
            }))
        } else {
            unimplemented!("frame_type = {frame_type}")
        }
    }
}
