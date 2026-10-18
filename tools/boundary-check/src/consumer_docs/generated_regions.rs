//! Unique bounded generated regions; no handwritten prose is rewritten.
use super::{configuration::Configuration, source_facts};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub(super) fn planned(
    root: &Path,
    config: &Configuration,
) -> Result<BTreeMap<PathBuf, String>, String> {
    let mut changes = BTreeMap::new();
    for document in &config.documents {
        let text = crate::source_documents::read_to_string(&root.join(document))
            .map_err(|e| e.to_string())?;
        for event in pulldown_cmark::Parser::new(&text) {
            let pulldown_cmark::Event::Html(html) = event else {
                continue;
            };
            for line in html
                .lines()
                .filter(|line| line.trim_start().starts_with("<!-- source-facts:"))
            {
                let known = config.facts.iter().any(|fact| {
                    fact.document == *document
                        && ["begin", "end"].iter().any(|side| {
                            line == format!("<!-- source-facts: {} {side} -->", fact.id)
                        })
                });
                if !known {
                    return Err(format!(
                        "{document}: unknown or malformed source-facts region"
                    ));
                }
            }
        }
    }
    let sources = crate::facade_docs::resolution::CrateIndex::new(root);
    for fact in &config.facts {
        let path = super::configuration::confined_existing(root, &fact.document)?;
        let text = changes
            .entry(path.clone())
            .or_insert(crate::source_documents::read_to_string(&path).map_err(|e| e.to_string())?);
        let rendered = source_facts::render(root, fact, &sources)?;
        *text = replace(text, &fact.id, &rendered)?;
    }
    Ok(changes)
}

/// Offsets come only from standalone active HTML comments observed by Markdown.
pub(super) fn markers(
    text: &str,
    prefix: &str,
) -> Result<Vec<(String, std::ops::Range<usize>)>, String> {
    let mut result = Vec::new();
    for (event, range) in pulldown_cmark::Parser::new(text).into_offset_iter() {
        let pulldown_cmark::Event::Html(_) = event else {
            continue;
        };
        let mut start = range.start;
        // Markdown may split HTML at CRLF and omit the physical line ending
        // from an event range. Keep the parser's start, then use source offsets.
        while start < range.end {
            let end = text[start..]
                .find('\n')
                .map_or(text.len(), |n| start + n + 1);
            let marker = text[start..end].trim_end_matches(['\r', '\n']);
            if marker.trim_start().starts_with(prefix) {
                if marker != marker.trim() || !marker.ends_with(" -->") {
                    return Err("marker must be a standalone unindented comment".into());
                }
                result.push((marker.to_owned(), start..end));
            }
            start = end;
        }
    }
    Ok(result)
}

pub(super) fn replace(text: &str, id: &str, body: &str) -> Result<String, String> {
    let newline = if text.contains("\r\n") { "\r\n" } else { "\n" };
    let begin = format!("<!-- source-facts: {id} begin -->");
    let end = format!("<!-- source-facts: {id} end -->");
    let observed = markers(text, "<!-- source-facts:")?;
    let starts = observed
        .iter()
        .filter(|(s, _)| s == &begin)
        .collect::<Vec<_>>();
    let ends = observed
        .iter()
        .filter(|(s, _)| s == &end)
        .collect::<Vec<_>>();
    if starts.len() != 1 || ends.len() != 1 {
        return Err(format!(
            "{id}: expected exactly one active generated begin/end pair"
        ));
    }
    let start = starts[0].1.end;
    let finish = ends[0].1.start;
    if finish < start {
        return Err(format!("{id}: reversed region"));
    }
    // A region may neither contain another active marker nor be inside one.
    let mut active = None;
    for (marker, range) in &observed {
        if marker.ends_with(" begin -->") {
            if active.is_some() {
                return Err("nested generated regions".into());
            }
            active = Some(marker);
        } else if marker.ends_with(" end -->") {
            let expected = marker.replace(" end -->", " begin -->");
            if active.map(String::as_str) != Some(expected.as_str()) {
                return Err("unpaired generated region".into());
            }
            active = None;
        } else {
            return Err(format!("invalid generated marker at {}", range.start));
        }
    }
    if active.is_some() {
        return Err("unclosed generated region".into());
    }
    Ok(format!(
        "{}{}{}",
        &text[..start],
        body.replace('\n', newline),
        &text[finish..]
    ))
}
