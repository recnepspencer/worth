use std::path::Path;

use crate::paths::source_key;

pub(crate) fn attributed(message: &serde_json::Value, source: &Path) -> bool {
    if message["spans"]
        .as_array()
        .into_iter()
        .flatten()
        .any(|span| attributed_span(span, source))
    {
        return true;
    }
    message["children"]
        .as_array()
        .into_iter()
        .flatten()
        .any(|child| attributed(child, source))
}

fn attributed_span(span: &serde_json::Value, source: &Path) -> bool {
    if span["file_name"]
        .as_str()
        .is_some_and(|path| source_key(Path::new(path)) == source_key(source))
    {
        return true;
    }
    span.get("expansion")
        .filter(|value| !value.is_null())
        .is_some_and(|expansion| attributed_span(&expansion["span"], source))
}

#[cfg(test)]
pub(crate) use crate::source_locations::normalize;
