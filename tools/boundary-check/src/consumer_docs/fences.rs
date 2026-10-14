//! Rust fence observations and immediately adjacent classification markers.
use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag, TagEnd};

#[derive(Debug)]
pub(super) enum Marker {
    Compiled(String),
    Doctest(String),
    Illustrative,
    Invalid(String),
}
pub(super) struct Fence {
    pub line: usize,
    pub info: String,
    pub code: String,
    pub marker: Option<Marker>,
}
pub(super) struct Observation {
    pub fences: Vec<Fence>,
    pub invalid_markers: Vec<(usize, String)>,
}

pub(super) fn observe(text: &str) -> Observation {
    let mut fences = Vec::new();
    let mut pending = None;
    let mut invalid_markers = Vec::new();
    let mut active: Option<Fence> = None;
    for (event, range) in Parser::new_ext(text, Options::ENABLE_TABLES).into_offset_iter() {
        match event {
            Event::Html(html) | Event::InlineHtml(html) => {
                if let Some(marker) = parse_marker(html.trim()) {
                    if let Some((_, line, _)) = pending.take() {
                        invalid_markers.push((
                            line,
                            "marker has no immediately following Rust fence".into(),
                        ));
                    }
                    pending = Some((marker, line_number(text, range.start), range.end));
                }
            }
            Event::Start(Tag::CodeBlock(kind)) => {
                let info = match kind {
                    CodeBlockKind::Fenced(info) => info.to_string(),
                    CodeBlockKind::Indented => String::new(),
                };
                let marker = pending.take().and_then(|(marker, line, end)| {
                    if line_number(text, range.start) == line + 1
                        && text[end..range.start].chars().all(char::is_whitespace)
                    {
                        Some(marker)
                    } else {
                        invalid_markers.push((
                            line,
                            "marker must be on the line immediately before its fence".into(),
                        ));
                        None
                    }
                });
                active = Some(Fence {
                    line: line_number(text, range.start),
                    info,
                    code: String::new(),
                    marker,
                });
            }
            Event::Text(value) if active.is_some() => {
                active.as_mut().unwrap().code.push_str(&value)
            }
            Event::End(TagEnd::CodeBlock) => {
                if let Some(fence) = active.take() {
                    if fence.info.split([',', ' ']).next() == Some("rust") || fence.marker.is_some()
                    {
                        fences.push(fence);
                    }
                }
            }
            _ => {}
        }
    }
    if let Some((_, line, _)) = pending {
        invalid_markers.push((line, "marker has no following Rust fence".into()));
    }
    Observation {
        fences,
        invalid_markers,
    }
}

fn parse_marker(html: &str) -> Option<Marker> {
    if html == "<!-- illustrative -->" {
        return Some(Marker::Illustrative);
    }
    for (prefix, compiled) in [("<!-- compiled: ", true), ("<!-- doctest: ", false)] {
        if let Some(value) = html
            .strip_prefix(prefix)
            .and_then(|rest| rest.strip_suffix(" -->"))
        {
            if !value.is_empty() && !value.chars().any(char::is_whitespace) {
                return Some(if compiled {
                    Marker::Compiled(value.into())
                } else {
                    Marker::Doctest(value.into())
                });
            }
        }
    }
    ["compiled", "doctest", "illustrative"]
        .iter()
        .any(|name| html.starts_with(&format!("<!-- {name}")))
        .then(|| Marker::Invalid("invalid consumer marker grammar".into()))
}

fn line_number(text: &str, offset: usize) -> usize {
    text[..offset].bytes().filter(|&byte| byte == b'\n').count() + 1
}
