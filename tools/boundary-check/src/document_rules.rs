//! Consumer Markdown code regions and repository-relative destinations.
use crate::{
    config::*,
    inputs::Inputs,
    lexical::{close, tokens},
    runtime_courts::normalize,
};
use std::collections::{BTreeMap, BTreeSet};
pub fn check(input: &Inputs) -> Vec<Diagnostic> {
    let config =
        match toml::from_str::<toml::Value>(input.text(&format!("{CONFIG}consumer-docs.toml"))) {
            Ok(c) => c,
            Err(e) => return vec![diagnostic("BC8008", CONFIG, e.to_string())],
        };
    let mut out = Vec::new();
    for document in strings(&config, "documents") {
        let Some(text) = input.files.get(&document) else {
            out.push(diagnostic(
                "BC8008",
                &document,
                "configured document missing",
            ));
            continue;
        };
        let mut lines = text.lines().enumerate().peekable();
        let mut marker = None;
        let definitions: BTreeMap<_, _> = text
            .lines()
            .filter_map(|line| line.trim().strip_prefix('[')?.split_once("]:"))
            .map(|(label, dest)| (label.to_lowercase(), dest.trim()))
            .collect();
        while let Some((index, line)) = lines.next() {
            let subject = format!("{document}:{}", index + 1);
            if let Some(name) = line
                .strip_prefix("<!-- compiled: ")
                .and_then(|s| s.strip_suffix(" -->"))
            {
                marker = Some(name);
                continue;
            }
            if line.trim_start().starts_with("```") || line.trim_start().starts_with("~~~") {
                let fence = &line.trim_start()[..3];
                let mut code = String::new();
                for (_, line) in lines.by_ref() {
                    if line.trim_start().starts_with(fence) {
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
                            "BC8008",
                            &subject,
                            format!("compiled code region differs from {source}"),
                        ));
                    }
                }
                continue;
            }
            if marker.take().is_some() {
                out.push(diagnostic(
                    "BC8008",
                    &subject,
                    "compiled marker must immediately precede a fenced region",
                ));
            }
            for destination in destinations(line, &definitions) {
                if let Err(reason) = link(input, &document, &destination) {
                    out.push(diagnostic(
                        "BC8009",
                        &subject,
                        format!("{destination}: {reason}"),
                    ));
                }
            }
        }
    }
    out.extend(facts(input, &config));
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
fn destinations(line: &str, definitions: &BTreeMap<String, &str>) -> Vec<String> {
    if line.trim().starts_with('[') && line.trim().contains("]:") {
        return Vec::new();
    }
    let mut result = Vec::new();
    let mut rest = line;
    let mut literal = false;
    while let Some(i) = rest.find(['[', '`']) {
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
                result.push(
                    tail[..end]
                        .trim()
                        .split_whitespace()
                        .next()
                        .unwrap_or("")
                        .trim_matches(['<', '>'])
                        .into(),
                );
                rest = &tail[end + 1..];
            } else {
                break;
            }
        } else if let Some(tail) = after.strip_prefix('[') {
            if let Some((key, tail)) = tail.split_once(']') {
                if let Some(dest) =
                    definitions.get(&if key.is_empty() { label } else { key }.to_lowercase())
                {
                    result.push(dest.split_whitespace().next().unwrap_or("").into());
                }
                rest = tail;
            } else {
                break;
            }
        } else {
            if let Some(dest) = definitions.get(&label.to_lowercase()) {
                result.push(
                    dest.split_whitespace()
                        .next()
                        .unwrap_or("")
                        .trim_matches(['<', '>'])
                        .into(),
                );
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
fn facts(input: &Inputs, config: &toml::Value) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    for fact in table(config, "facts") {
        let name = string(fact, "item").rsplit("::").next().unwrap();
        let mut definitions = Vec::new();
        for (_, source) in input
            .files
            .iter()
            .filter(|(p, s)| p.ends_with(".rs") && s.contains(&format!("enum {name}")))
        {
            let t = tokens(source);
            for i in 0..t.len().saturating_sub(2) {
                if t[i].text != "enum" || t[i + 1].text != name {
                    continue;
                }
                let Some(open) = (i + 2..t.len()).find(|i| t[*i].text == "{") else {
                    continue;
                };
                let end = close(&t, open);
                let mut at = open + 1;
                let mut variants = Vec::new();
                while at < end {
                    if t[at].text == "#" && t.get(at + 1).is_some_and(|t| t.text == "[") {
                        at = close(&t, at + 1) + 1;
                        continue;
                    }
                    if t[at].text == "," {
                        at += 1;
                        continue;
                    }
                    let name = t[at].text.to_owned();
                    at += 1;
                    let shape = if ["(", "{"].contains(&t[at].text) {
                        let prefix = if t[at].text == "(" { "tuple" } else { "named" };
                        canonical(&format!(
                            "{prefix}{}",
                            t[at..=close(&t, at)]
                                .iter()
                                .map(|t| t.text)
                                .collect::<String>()
                        ))
                    } else {
                        "unit".into()
                    };
                    variants.push((name, shape));
                    while at < end && t[at].text != "," {
                        at = close(&t, at) + 1;
                    }
                }
                definitions.push(variants);
            }
        }
        let document = string(fact, "document");
        let text = input.text(document);
        let start = format!("<!-- source-facts: {} begin -->", string(fact, "id"));
        let end = format!("<!-- source-facts: {} end -->", string(fact, "id"));
        let rows: Vec<_> = text
            .split_once(&start)
            .and_then(|(_, t)| t.split_once(&end))
            .map(|(t, _)| {
                t.lines()
                    .filter_map(|l| {
                        l.strip_prefix("| `")
                            .and_then(|l| l.split_once('`'))
                            .map(|(v, shape)| {
                                (
                                    v.to_owned(),
                                    canonical(shape.trim().trim_matches('|').trim()),
                                )
                            })
                    })
                    .collect()
            })
            .unwrap_or_default();
        if definitions.len() != 1 || definitions[0] != rows {
            out.push(diagnostic(
                "BC8008",
                document,
                format!("source-facts variant or payload drift for {name}"),
            ));
        }
    }
    out
}
fn canonical(shape: &str) -> String {
    let t = tokens(shape);
    t.iter()
        .enumerate()
        .filter(|(i, token)| {
            token.text != ","
                && !(token.text == ":"
                    && (i.checked_sub(1).is_some_and(|p| t[p].text == ":")
                        || t.get(i + 1).is_some_and(|t| t.text == ":")))
                && !(t.get(i + 1).is_some_and(|t| t.text == ":")
                    && t.get(i + 2).is_some_and(|t| t.text == ":"))
        })
        .map(|(_, t)| t.text)
        .collect()
}
#[cfg(test)]
mod tests;
