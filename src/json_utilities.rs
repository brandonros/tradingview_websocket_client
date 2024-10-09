use miniserde::json::{Array, Number, Object, Value};

use crate::types::Result;

pub fn value_to_string(input: &Value) -> Result<String> {
    match input {
        Value::String(value) => Ok(value.clone()),
        _ => Err("parsing failed".into())
    }
}

pub fn value_to_array(input: &Value) -> Result<Array> {
    match input {
        Value::Array(value) => Ok(value.clone()),
        _ => Err("parsing failed".into())
    }
}

pub fn value_to_object(input: &Value) -> Result<Object> {
    match input {
        Value::Object(value) => Ok(value.clone()),
        _ => Err("parsing failed".into())
    }
}

pub fn value_to_number(input: &Value) -> Result<Number> {
    match input {
        Value::Number(value) => Ok(value.clone()),
        _ => Err("parsing failed".into())
    }
}

pub fn value_to_bool(input: &Value) -> Result<bool> {
    match input {
        Value::Bool(value) => Ok(value.clone()),
        _ => Err("parsing failed".into())
    }
}

pub fn is_null(input: &Object, key: &str) -> Result<bool> {
    let value = input.get(key).ok_or("failed to get key")?;
    match value {
        Value::Null => Ok(true),
        _ => Ok(false),
    }
}
