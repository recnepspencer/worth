//! Consumer Markdown code regions and repository-relative destinations.
use crate::{config::*, inputs::Inputs, runtime_courts::normalize};
use std::collections::{BTreeMap, BTreeSet};
mod source_facts;
pub fn check(input: &Inputs) -> Vec<Diagnostic> {
    let config =
        match toml::from_str::<toml::Value>(input.text(&format!("{CONFIG}consumer-docs.toml"))) {
            Ok(c) => c,
            Err(e) => return vec![diagnostic(Code::DocumentRegion, CONFIG, e.to_string())],
        };
    let mut out = Vec::new();
    for document in strings(&config, "documents") {
        let Some(text) = input.files.get(&document) else {
            out.push(diagnostic(
                Code::DocumentRegion,
                &document,
                "configured document missing",
            ));
            continue;
        };
        let mut lines = text.lines().enumerate().peekable();
        let mut marker = None;
        let mut link_lines = vec![""; text.lines().count()];
        let definitions: BTreeMap<_, _> = text
            .lines()
            .filter_map(reference_definition)
            .map(|(label, dest)| (label.to_lowercase(), dest.trim()))
            .collect();
        while let Some((index, line)) = lines.next() {
            let subject = format!("{document}:{}", index + 1);
            if line.trim().starts_with("<!-- compiled:") {
                if marker.take().is_some() {
                    out.push(diagnostic(
                        Code::DocumentRegion,
                        &subject,
                        "compiled marker replaced before its fenced region",
                    ));
                }
                marker = line
                    .trim()
                    .strip_prefix("<!-- compiled: ")
                    .and_then(|s| s.strip_suffix(" -->"))
                    .filter(|s| !s.is_empty());
                if marker.is_none() {
                    out.push(diagnostic(
                        Code::DocumentRegion,
                        &subject,
                        "invalid compiled marker",
                    ));
                }
                continue;
            }
            if line.trim_start().starts_with("```") || line.trim_start().starts_with("~~~") {
                let opening = line.trim_start();
                let width = opening
                    .bytes()
                    .take_while(|b| *b == opening.as_bytes()[0])
                    .count();
                let fence = &opening[..width];
                let mut code = String::new();
                for (_, line) in lines.by_ref() {
                    if line.trim_start().strip_prefix(fence).is_some_and(|tail| {
                        tail.trim_start_matches(opening.as_bytes()[0] as char)
                            .trim()
                            .is_empty()
                    }) {
                        break;
                    }
                    code.push_str(line);
                    code.push('\n');
                }
                if let Some(name) = marker.take() {
                    let binding = table(&config, "compiled_sources")
                        .into_iter()
                        .find(|b| string(b, "name") == name);
                    let source = binding.map_or_else(|| format!("workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/topology_entry/src/{name}"), |b| format!("{}/{}", string(b, "crate_root"), string(b, "source")));
                    if !region_matches(input.text(&source), &code) {
                        out.push(diagnostic(
                            Code::DocumentRegion,
                            &subject,
                            format!("compiled code region differs from {source}"),
                        ));
                    }
                }
                continue;
            }
            if marker.take().is_some() {
                out.push(diagnostic(
                    Code::DocumentRegion,
                    &subject,
                    "compiled marker must immediately precede a fenced region",
                ));
            }
            if reference_definition(line).is_none() {
                link_lines[index] = line;
            }
        }
        let link_text = link_lines.join("\n");
        for (offset, destination) in destinations(&link_text, &definitions) {
            if let Err(reason) = link(input, &document, &destination) {
                let subject = format!(
                    "{document}:{}",
                    link_text[..offset].bytes().filter(|b| *b == b'\n').count() + 1
                );
                out.push(diagnostic(
                    Code::DocumentLink,
                    &subject,
                    format!("{destination}: {reason}"),
                ));
            }
        }
        if marker.is_some() {
            out.push(diagnostic(
                Code::DocumentRegion,
                &document,
                "compiled marker has no fenced region",
            ));
        }
    }
    out.extend(source_facts::check(input, &config));
    out
}
fn region_matches(source: &str, code: &str) -> bool {
    let begin = "// compiled-example: begin\n";
    let end = "// compiled-example: end";
    source.matches(begin).count() == 1
        && source.matches(end).count() == 1
        && source
            .split_once(begin)
            .and_then(|(_, s)| s.split_once(end))
            .is_some_and(|(region, _)| region.trim() == code.trim())
}
fn reference_definition(line: &str) -> Option<(&str, &str)> {
    let (label, tail) = line.trim().strip_prefix('[')?.split_once(']')?;
    Some((label, tail.strip_prefix(':')?.trim()))
}
fn destinations(line: &str, definitions: &BTreeMap<String, &str>) -> Vec<(usize, String)> {
    let mut result = Vec::new();
    let mut rest = line;
    let mut literal = false;
    while let Some(i) = rest.find(['[', '`']) {
        let offset = line.len() - rest.len() + i;
        let ch = rest.as_bytes()[i];
        rest = &rest[i + 1..];
        if ch == b'`' {
            literal = !literal;
            continue;
        }
        if literal {
            continue;
        }
        let Some((label, after)) = rest.split_once(']') else {
            break;
        };
        if let Some(tail) = after.strip_prefix('(') {
            let mut depth = 1;
            let mut end = None;
            for (i, c) in tail.char_indices() {
                if c == '(' {
                    depth += 1;
                }
                if c == ')' {
                    depth -= 1;
                    if depth == 0 {
                        end = Some(i);
                        break;
                    }
                }
            }
            if let Some(end) = end {
                result.push((
                    offset,
                    tail[..end]
                        .trim()
                        .split_whitespace()
                        .next()
                        .unwrap_or("")
                        .trim_matches(['<', '>'])
                        .into(),
                ));
                rest = &tail[end + 1..];
            } else {
                break;
            }
        } else if let Some(tail) = after.strip_prefix('[') {
            if let Some((key, tail)) = tail.split_once(']') {
                if let Some(dest) =
                    definitions.get(&if key.is_empty() { label } else { key }.to_lowercase())
                {
                    result.push((offset, dest.split_whitespace().next().unwrap_or("").into()));
                }
                rest = tail;
            } else {
                break;
            }
        } else {
            if let Some(dest) = definitions.get(&label.to_lowercase()) {
                result.push((
                    offset,
                    dest.split_whitespace()
                        .next()
                        .unwrap_or("")
                        .trim_matches(['<', '>'])
                        .into(),
                ));
            }
            rest = after;
        }
    }
    result
}
fn decode(text: &str) -> Result<String, String> {
    let mut bytes = Vec::new();
    let mut it = text.bytes();
    while let Some(b) = it.next() {
        if b == b'%' {
            let a = it.next().and_then(|c| (c as char).to_digit(16));
            let b = it.next().and_then(|c| (c as char).to_digit(16));
            match (a, b) {
                (Some(a), Some(b)) => bytes.push((a * 16 + b) as u8),
                _ => return Err("invalid percent encoding".into()),
            }
        } else {
            bytes.push(b);
        }
    }
    String::from_utf8(bytes).map_err(|e| e.to_string())
}
fn link(input: &Inputs, document: &str, destination: &str) -> Result<(), String> {
    if destination.starts_with("//")
        || destination.split_once(':').is_some_and(|(scheme, _)| {
            scheme
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || "+-.".contains(c))
        })
    {
        return Ok(());
    }
    let (path, anchor) = destination.split_once('#').unwrap_or((destination, ""));
    let path = decode(path.split('?').next().unwrap())?;
    let target = if path.is_empty() {
        document.into()
    } else if path.starts_with('/') {
        normalize(&path)
    } else {
        normalize(&format!(
            "{}/{path}",
            document.rsplit_once('/').map_or("", |(p, _)| p)
        ))
    };
    if target == "<outside-repository>"
        || !target.is_empty()
            && !input.paths.contains(&target)
            && !input.paths.iter().any(|p| under(p, &target))
    {
        return Err("relative target missing or outside repository".into());
    }
    if !anchor.is_empty() && !anchors(input.text(&target)).contains(&decode(anchor)?) {
        return Err("heading anchor missing".into());
    }
    Ok(())
}
fn anchors(text: &str) -> BTreeSet<String> {
    let mut anchors = BTreeSet::new();
    let mut fenced = false;
    for line in text.lines() {
        if line.trim_start().starts_with("```") || line.trim_start().starts_with("~~~") {
            fenced = !fenced;
            continue;
        }
        if fenced {
            continue;
        }
        let heading = line.trim_start_matches('#');
        if heading.len() == line.len() || !heading.starts_with(' ') {
            continue;
        }
        let base: String = heading
            .trim()
            .trim_end_matches('#')
            .trim()
            .to_lowercase()
            .chars()
            .filter(|c| c.is_alphanumeric() || ['_', '-', ' '].contains(c))
            .map(|c| if c == ' ' { '-' } else { c })
            .collect();
        let mut slug = base.clone();
        let mut suffix = 0;
        while anchors.contains(&slug) {
            suffix += 1;
            slug = format!("{base}-{suffix}");
        }
        anchors.insert(slug);
    }
    anchors
}
#[cfg(test)]
mod tests;
