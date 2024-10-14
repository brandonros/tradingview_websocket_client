use miniserde::json::{Array, Number, Object, Value};

pub fn value_to_string(input: &Value) -> anyhow::Result<String> {
    match input {
        Value::String(value) => Ok(value.clone()),
        _ => Err(anyhow::anyhow!("parsing failed"))
    }
}

pub fn value_to_array(input: &Value) -> anyhow::Result<Array> {
    match input {
        Value::Array(value) => Ok(value.clone()),
        _ => Err(anyhow::anyhow!("parsing failed"))
    }
}

pub fn value_to_object(input: &Value) -> anyhow::Result<Object> {
    match input {
        Value::Object(value) => Ok(value.clone()),
        _ => Err(anyhow::anyhow!("parsing failed"))
    }
}

pub fn value_to_number(input: &Value) -> anyhow::Result<Number> {
    match input {
        Value::Number(value) => Ok(value.clone()),
        _ => Err(anyhow::anyhow!("parsing failed"))
    }
}

pub fn value_to_bool(input: &Value) -> anyhow::Result<bool> {
    match input {
        Value::Bool(value) => Ok(value.clone()),
        _ => Err(anyhow::anyhow!("parsing failed"))
    }
}

pub fn is_null(input: &Object, key: &str) -> anyhow::Result<bool> {
    let value = input.get(key).ok_or(anyhow::anyhow!("failed to get key"))?;
    match value {
        Value::Null => Ok(true),
        _ => Ok(false),
    }
}
