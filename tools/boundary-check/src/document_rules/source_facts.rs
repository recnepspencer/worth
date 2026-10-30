//! Agreement between documented outcome tables and their owning enum declarations.
use crate::{
    config::*,
    inputs::Inputs,
    lexical::{close, tokens},
};
pub(super) fn check(input: &Inputs, config: &toml::Value) -> Vec<Diagnostic> {
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
                Code::DocumentRegion,
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
