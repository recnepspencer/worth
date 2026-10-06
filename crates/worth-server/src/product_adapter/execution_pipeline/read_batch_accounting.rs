use serde_json::Value;

pub(crate) fn string(value: &String) -> u64 {
    u64::try_from(value.capacity()).unwrap_or(u64::MAX)
}

pub(crate) fn option_string(value: &Option<String>) -> u64 {
    value.as_ref().map_or(0, string)
}

pub(crate) fn strings(values: &Vec<String>) -> u64 {
    let inline = values
        .capacity()
        .saturating_mul(std::mem::size_of::<String>());
    values
        .iter()
        .fold(u64::try_from(inline).unwrap_or(u64::MAX), |total, value| {
            total.saturating_add(string(value))
        })
}

pub(crate) fn json(value: &Value) -> u64 {
    match value {
        Value::Null | Value::Bool(_) | Value::Number(_) => 0,
        Value::String(value) => string(value),
        Value::Array(values) => {
            let inline = values
                .capacity()
                .saturating_mul(std::mem::size_of::<Value>());
            values
                .iter()
                .fold(u64::try_from(inline).unwrap_or(u64::MAX), |total, value| {
                    total.saturating_add(json(value))
                })
        }
        Value::Object(values) => {
            // serde_json::Map uses a BTreeMap by default. Each entry owns two
            // child pointers and one color tag in addition to its key/value.
            let node = std::mem::size_of::<String>()
                .saturating_add(std::mem::size_of::<Value>())
                .saturating_add(4 * std::mem::size_of::<usize>());
            values.iter().fold(0_u64, |total, (key, value)| {
                total
                    .saturating_add(u64::try_from(node).unwrap_or(u64::MAX))
                    .saturating_add(u64::try_from(key.capacity()).unwrap_or(u64::MAX))
                    .saturating_add(json(value))
            })
        }
    }
}
