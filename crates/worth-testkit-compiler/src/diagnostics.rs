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

pub(crate) fn normalize(text: &str, source: &Path) -> String {
    let key = source_key(source);
    let moved_root = moved_source_root(&key);
    text.replace("\r\n", "\n")
        .lines()
        .map(|line| {
            let leading = line.len() - line.trim_start().len();
            let marker = line[leading..]
                .strip_prefix("--> ")
                .or_else(|| line[leading..].strip_prefix("::: "))
                .map(|_| leading + 4);
            let Some(start) = marker else {
                return line.to_owned();
            };
            let location = &line[start..];
            let mut parts = location.rsplitn(3, ':');
            let (Some(column), Some(number), Some(path)) =
                (parts.next(), parts.next(), parts.next())
            else {
                return line.to_owned();
            };
            if column.is_empty()
                || number.is_empty()
                || !column.bytes().all(|byte| byte.is_ascii_digit())
                || !number.bytes().all(|byte| byte.is_ascii_digit())
            {
                return line.to_owned();
            }
            let observed_path = source_key(Path::new(path));
            match moved_root
                .as_deref()
                .and_then(|root| observed_path.strip_prefix(root))
            {
                Some(relative) => format!("{}tests/{relative}:{number}:{column}", &line[..start]),
                None => line.to_owned(),
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
        .trim_end_matches('\n')
        .to_owned()
}

fn moved_source_root(key: &str) -> Option<String> {
    for (offset, _) in key.match_indices("/tests/") {
        let tail = &key[offset + "/tests/".len()..];
        let Some((_, after_binary)) = tail.split_once('/') else {
            continue;
        };
        let Some(after_sources) = after_binary.strip_prefix("sources/") else {
            continue;
        };
        let Some((owner, _)) = after_sources.split_once('/') else {
            continue;
        };
        let prefix_len = key.len() - after_sources.len() + owner.len() + 1;
        return Some(key[..prefix_len].to_owned());
    }
    None
}
