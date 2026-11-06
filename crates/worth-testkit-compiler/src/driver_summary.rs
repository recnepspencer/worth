//! Recognize a terminal driver summary without weakening real error evidence.
use serde_json::Value;

pub(crate) fn is_abort_summary(message: &Value) -> bool {
    if message["level"] != "error"
        || message.get("code") != Some(&Value::Null)
        || !message["spans"].as_array().is_some_and(Vec::is_empty)
        || !message["children"].as_array().is_some_and(Vec::is_empty)
    {
        return false;
    }
    let Some(text) = message["message"].as_str() else {
        return false;
    };
    let Some(rest) = text.strip_prefix("aborting due to ") else {
        return false;
    };
    let Some(rest) = counted(rest, "previous error", "previous errors") else {
        return false;
    };
    let complete = rest.is_empty()
        || rest
            .strip_prefix("; ")
            .and_then(|rest| counted(rest, "warning emitted", "warnings emitted"))
            == Some("");
    complete
        && message["rendered"].as_str().is_some_and(|rendered| {
            rendered.trim_end_matches(['\r', '\n']) == format!("error: {text}")
        })
}

fn counted<'a>(text: &'a str, singular: &str, plural: &str) -> Option<&'a str> {
    let (count, rest) = text.split_once(' ')?;
    if count.starts_with('0') || !count.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    let count = count.parse::<usize>().ok()?;
    rest.strip_prefix(if count == 1 { singular } else { plural })
}

#[cfg(test)]
mod tests;
