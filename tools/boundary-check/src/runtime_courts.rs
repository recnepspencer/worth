//! Law 2: registered runtime names inside cfg(test) module text, not constructors.
use crate::{
    config::*,
    inputs::Inputs,
    lexical::{close, test_modules, tokens, Token},
};
use serde::{
    ser::{SerializeStruct, Serializer},
    Serialize,
};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Finding {
    pub source: String,
    pub module: String,
    pub roots: Vec<String>,
}
pub type Scopes = BTreeMap<String, Vec<(String, usize, usize)>>;
pub fn scopes(input: &Inputs) -> Scopes {
    let mut scopes: Scopes = BTreeMap::new();
    let mut mounted = BTreeSet::new();
    let mut pending = Vec::new();
    for (path, text) in input.files.iter().filter(|(p, _)| {
        p.ends_with(".rs") && p.contains("/src/") && governed(p) && !p.starts_with("tools/")
    }) {
        if !text.contains("cfg") || !text.contains("test") || !text.contains("mod") {
            continue;
        }
        let t = tokens(text);
        for (name, start, end) in test_modules(&t) {
            if t[start].text == "{" {
                let module = (0..start)
                    .filter(|i| {
                        t[*i].text == "mod"
                            && t.get(i + 2).is_some_and(|t| t.text == "{")
                            && close(&t, i + 2) >= end
                    })
                    .map(|i| t[i + 1].text)
                    .collect::<Vec<_>>()
                    .join("::");
                scopes
                    .entry(path.clone())
                    .or_default()
                    .push((module, t[start].end, t[end].start));
            } else {
                pending.extend(mounts(input, path, &t, name, start));
            }
        }
    }
    while let Some(path) = pending.pop() {
        if !mounted.insert(path.clone()) {
            continue;
        }
        if let Some(text) = input.files.get(&path) {
            scopes
                .entry(path.clone())
                .or_default()
                .push(("file".into(), 0, text.len()));
            let t = tokens(text);
            for i in 0..t.len().saturating_sub(2) {
                if t[i].text == "mod" && t[i + 2].text == ";" {
                    pending.extend(mounts(input, &path, &t, t[i + 1].text, i + 2));
                }
            }
        }
    }
    scopes
}
fn mounts(input: &Inputs, source: &str, t: &[Token<'_>], name: &str, end: usize) -> Vec<String> {
    let parent = source.rsplit_once('/').unwrap().0;
    let stem = source.trim_end_matches(".rs");
    let directory = if source.ends_with("/lib.rs")
        || source.ends_with("/main.rs")
        || source.ends_with("/mod.rs")
    {
        parent
    } else {
        stem
    };
    let boundary = t[..end.saturating_sub(2)]
        .iter()
        .rposition(|t| [";", "{", "}"].contains(&t.text))
        .map_or(0, |i| i + 1);
    let explicit = t[boundary..end]
        .windows(3)
        .find(|w| w[0].text == "path" && w[1].text == "=")
        .map(|w| w[2].text.trim_matches('"'));
    let candidates = explicit.map_or_else(
        || {
            vec![
                format!("{directory}/{name}.rs"),
                format!("{directory}/{name}/mod.rs"),
            ]
        },
        |p| vec![normalize(&format!("{parent}/{p}"))],
    );
    candidates
        .into_iter()
        .filter(|p| input.files.contains_key(p))
        .collect()
}
pub fn normalize(path: &str) -> String {
    let mut parts = Vec::new();
    for part in path.split('/') {
        match part {
            "." | "" => {}
            ".." => {
                if parts.pop().is_none() {
                    return "<outside-repository>".into();
                }
            }
            _ => parts.push(part),
        }
    }
    parts.join("/")
}
pub fn observations(input: &Inputs) -> Vec<Finding> {
    let registry: toml::Value = toml::from_str(input.text(&format!("{CONFIG}runtime-roots.toml")))
        .expect("runtime root configuration validated by input");
    let names: BTreeSet<_> = table(&registry, "roots")
        .iter()
        .map(|r| string(r, "type").to_owned())
        .collect();
    let mut found = Vec::new();
    for (source, scopes) in scopes(input) {
        let Some((manifest, _)) = input
            .manifests
            .iter()
            .filter(|(p, _)| under(&source, p.trim_end_matches("/Cargo.toml")))
            .max_by_key(|(p, _)| p.len())
        else {
            continue;
        };
        let directory = manifest.trim_end_matches("/Cargo.toml");
        if directory.starts_with("tools/")
            || directory.contains("-court-")
            || directory.contains("-testkit")
            || !governed(directory)
        {
            continue;
        }
        let text = input.text(&source);
        for (module, start, end) in scopes {
            let roots: BTreeSet<_> = tokens(&text[start..end])
                .iter()
                .filter(|t| names.contains(t.text))
                .map(|t| t.text.to_owned())
                .collect();
            if !roots.is_empty() {
                found.push(Finding {
                    source: source.clone(),
                    module,
                    roots: roots.into_iter().collect(),
                });
            }
        }
    }
    found.sort();
    found.dedup();
    found
}
pub fn check(input: &Inputs) -> Vec<Diagnostic> {
    let baseline = match serde_json::from_str::<Vec<serde_json::Value>>(
        input.text(&format!("{CONFIG}src-court-baseline.json")),
    ) {
        Ok(b) => b,
        Err(e) => return vec![diagnostic("BC9002", CONFIG, e.to_string())],
    };
    if baseline.iter().any(|b| {
        b["source"].as_str().is_none()
            || b["module"].as_str().is_none()
            || b["roots"]
                .as_array()
                .is_none_or(|a| a.iter().any(|r| r.as_str().is_none()))
    }) {
        return vec![diagnostic(
            "BC9002",
            CONFIG,
            "invalid runtime-root baseline schema",
        )];
    }
    observations(input)
        .into_iter()
        .filter(|f| {
            !baseline.iter().any(|b| {
                b["source"].as_str() == Some(f.source.as_str())
                    && b["module"].as_str() == Some(f.module.as_str())
                    && f.roots.iter().all(|r| {
                        b["roots"].as_array().is_some_and(|roots| {
                            roots.iter().any(|v| v.as_str() == Some(r.as_str()))
                        })
                    })
            })
        })
        .map(|f| {
            diagnostic(
                "BC9002",
                &format!("{}::{}", f.source, f.module),
                serde_json::to_string(&f).unwrap(),
            )
        })
        .collect()
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn runtime_mentions_are_scoped_to_inline_and_mounted_test_modules() {
        let mut input = Inputs::fixture(
            &[
                ("crates/a/Cargo.toml", "[package]\nname='a'"),
                (
                    "crates/a/src/lib.rs",
                    "fn product() { Root; } #[cfg(test)] mod tests;",
                ),
                ("crates/a/src/tests.rs", "fn unit() { Root; }"),
            ],
            "",
        );
        input.files.insert(
            format!("{CONFIG}runtime-roots.toml"),
            "[[roots]]\ntype='Root'".into(),
        );
        assert_eq!(observations(&input).len(), 1);
        input
            .files
            .insert(format!("{CONFIG}src-court-baseline.json"), "[]".into());
        assert_eq!(check(&input).len(), 1);
        input.files.insert(
            format!("{CONFIG}src-court-baseline.json"),
            r#"[{"source":"crates/a/src/tests.rs","module":"file","roots":["Root"]}]"#.into(),
        );
        assert!(check(&input).is_empty());
        input.files.insert(
            "crates/a/src/lib.rs".into(),
            "fn product() { Root; }".into(),
        );
        assert!(observations(&input).is_empty());
    }
}

impl Serialize for Finding {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("Finding", 3)?;
        state.serialize_field("source", &self.source)?;
        state.serialize_field("module", &self.module)?;
        state.serialize_field("roots", &self.roots)?;
        state.end()
    }
}
