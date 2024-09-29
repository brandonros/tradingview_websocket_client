use miniserde::json::{Object, Array, Value};

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

#[derive(Debug)]
pub struct QsdFrame {
    pub quote_session_id: String,
    pub update: Object
}

#[derive(Debug)]
pub struct DuFrame {
    pub chart_session_id: String,
    pub update: Object
}

#[derive(Debug)]
pub struct QuoteCompletedFrame {
    
}

#[derive(Debug)]
pub struct TimescaleUpdatedFrame {

}

#[derive(Debug)]
pub enum ParsedTradingViewFrame {
    Qsd(QsdFrame),
    Du(DuFrame),
    QuoteCompleted(QuoteCompletedFrame),
    TimescaleUpdate(TimescaleUpdatedFrame),
}

impl ParsedTradingViewFrame {
    pub fn from_string(value: &str) -> Result<Self> {
        let parsed_frame: miniserde::json::Object = miniserde::json::from_str(&value)?;
        let frame_type = parsed_frame.get("m").ok_or("failed to get frame_type")?;
        let frame_type = value_to_string(frame_type)?;

        if frame_type == "qsd" {
            log::info!("qsd = {parsed_frame:?}");
            let p = parsed_frame.get("p").ok_or("failed to get p")?;
            let p = value_to_array(p)?;
            let quote_session_id = &p[0];
            let quote_session_id = value_to_string(&quote_session_id)?;
            let update = &p[1];
            let update = value_to_object(&update)?;
            Ok(ParsedTradingViewFrame::Qsd(QsdFrame {
                quote_session_id,
                update,
            }))
        } else if frame_type == "du" {
            log::info!("du = {parsed_frame:?}");   
            let p = parsed_frame.get("p").ok_or("failed to get p")?;
            let p = value_to_array(p)?;
            let chart_session_id = &p[0];
            let chart_session_id = value_to_string(&chart_session_id)?;
            let update = &p[1];
            let update = value_to_object(&update)?;
            Ok(ParsedTradingViewFrame::Du(DuFrame {
                chart_session_id,
                update,
            }))
        } else if frame_type == "quote_completed" {
            log::info!("quote_completed = {parsed_frame:?}");            
            Ok(ParsedTradingViewFrame::QuoteCompleted(QuoteCompletedFrame {
                
            }))
        } else if frame_type == "timescale_update" {
            log::info!("timescale_update = {parsed_frame:?}");                        
            Ok(ParsedTradingViewFrame::TimescaleUpdate(TimescaleUpdatedFrame {
                
            }))
        } else {
            unimplemented!("frame_type = {frame_type}")
        }
    }
}
