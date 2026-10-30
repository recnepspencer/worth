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

/// rustc emits a codegen instantiation callsite as a separate diagnostic note.
/// It attests only the preceding E0080, never an unrelated error or note.
pub(crate) fn codegen_attributed(messages: &[serde_json::Value], source: &Path) -> bool {
    let mut pending = None;
    let mut instantiation_note_allowed = false;
    for message in messages {
        if message["level"] == "error" {
            if pending == Some(false) {
                return false;
            }
            pending = Some(attributed(message, source));
            instantiation_note_allowed = message["code"]["code"] == "E0080";
        } else if pending == Some(false)
            && instantiation_note_allowed
            && message["level"] == "note"
            && message["message"].as_str().is_some_and(|text| {
                text.starts_with("the above error was encountered while instantiating `")
            })
            && attributed(message, source)
        {
            pending = Some(true);
        }
    }
    pending == Some(true)
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
