use crate::resp::Value;

pub fn extract_array(value: &Value) -> Result<&[Value], anyhow::Error> {
    match value {
        Value::Array(array) => Ok(array),
        _ => Err(anyhow::anyhow!("expected array")),
    }
}

pub fn extract_bulk_string(array: &[Value], index: usize) -> Result<&str, anyhow::Error> {
    match array.get(index) {
        Some(Value::BulkString(s)) => Ok(s),
        Some(_) => Err(anyhow::anyhow!("expected bulk string at index {}", index)),
        None => Err(anyhow::anyhow!("missing element at index {}", index)),
    }
}

pub fn validate_main_command(array: &[Value], expected: &str) -> Result<(), anyhow::Error> {
    let cmd = extract_bulk_string(array, 0)?;
    if cmd.to_uppercase() != expected.to_uppercase() {
        return Err(anyhow::anyhow!("expected {} main command", expected));
    }
    Ok(())
}

pub fn validate_sub_command(array: &[Value], expected: &str) -> Result<(), anyhow::Error> {
    let cmd = extract_bulk_string(array, 1)?;
    if cmd.to_uppercase() != expected.to_uppercase() {
        return Err(anyhow::anyhow!("expected {} sub command", expected));
    }
    Ok(())
}

pub fn validate_array_length(array: &[Value], expected: usize) -> Result<(), anyhow::Error> {
    if array.len() != expected {
        return Err(anyhow::anyhow!(
            "expected {} arguments, got {}",
            expected,
            array.len()
        ));
    }
    Ok(())
}

pub fn validate_min_array_length(array: &[Value], min: usize) -> Result<(), anyhow::Error> {
    if array.len() < min {
        return Err(anyhow::anyhow!(
            "expected at least {} arguments, got {}",
            min,
            array.len()
        ));
    }
    Ok(())
}
