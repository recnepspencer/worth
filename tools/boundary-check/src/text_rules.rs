//! Identifier denials, feature predicates and Clippy configuration inheritance.
use crate::{
    config::*,
    inputs::Inputs,
    lexical::{close, tokens},
};
use std::collections::BTreeSet;

pub fn check(input: &Inputs) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    let identifier_rules: Vec<_> = table(&input.config, "source_identifier_denials")
        .into_iter()
        .map(|rule| {
            (
                string(rule, "root"),
                strings(rule, "exclude_paths"),
                strings(rule, "forbidden_identifiers"),
                strings(rule, "forbidden_identifier_fragments"),
                string(rule, "guidance"),
            )
        })
        .collect();
    let root_clippy = toml::from_str::<toml::Value>(input.text("clippy.toml"));
    for (path, text) in &input.files {
        if path.ends_with(".rs") {
            let rules: Vec<_> = identifier_rules
                .iter()
                .filter(|(root, excludes, _, _, _)| {
                    under(path, root) && !excludes.iter().any(|p| under(path, p))
                })
                .collect();
            let relevant = rules.iter().any(|(_, _, names, fragments, _)| {
                names.iter().chain(fragments).any(|n| text.contains(n))
            });
            if !relevant && !(governed(path) && text.contains("cfg") && text.contains("feature")) {
                continue;
            }
            let t = tokens(text);
            for (_, _, names, fragments, guidance) in rules {
                for token in &t {
                    if names.iter().any(|n| token.text == n)
                        || fragments.iter().any(|n| {
                            token.text.contains(n)
                                && token.text.chars().all(|c| c.is_alphanumeric() || c == '_')
                        })
                    {
                        out.push(diagnostic(
                            "BC2001",
                            path,
                            format!("forbidden identifier {}: {}", token.text, guidance),
                        ));
                    }
                }
            }
            if governed(path) {
                for index in 0..t.len().saturating_sub(1) {
                    if !["cfg", "cfg_attr"].contains(&t[index].text) {
                        continue;
                    }
                    let start = index + 1 + usize::from(t[index + 1].text == "!");
                    if t.get(start).is_none_or(|t| t.text != "(") {
                        continue;
                    }
                    for window in t[start..=close(&t, start)].windows(3) {
                        if window[0].text == "feature" && window[1].text == "=" {
                            let name = window[2].text.trim_matches('"');
                            if !feature_allowed(path, name) {
                                out.push(diagnostic(
                                    "BC9001",
                                    path,
                                    format!("one build forbids feature predicate `{name}`"),
                                ));
                            }
                        }
                    }
                }
            }
        }
        if path.ends_with("/clippy.toml") {
            match (&root_clippy, toml::from_str::<toml::Value>(text)) {
                (Ok(root), Ok(local)) if superset(&root, &local) => {}
                _ => out.push(diagnostic(
                    "BC7009",
                    path,
                    "crate Clippy configuration must be a superset of root configuration",
                )),
            }
        }
    }
    out
}
pub fn feature_allowed(path: &str, name: &str) -> bool {
    ((under(path, "crates/worth-signal") || under(path, "crates/worth-signal-wasm"))
        && [
            "default",
            "profile-compact",
            "profile-standard",
            "profile-extended",
        ]
        .contains(&name))
        || under(path, "crates/worth-signal") && name == "test-peak-allocation"
}
pub fn superset(root: &toml::Value, local: &toml::Value) -> bool {
    match (root, local) {
        (toml::Value::Table(a), toml::Value::Table(b)) => a
            .iter()
            .all(|(k, v)| b.get(k).is_some_and(|w| superset(v, w))),
        (toml::Value::Array(a), toml::Value::Array(b)) => a.iter().all(|v| b.contains(v)),
        _ => root == local,
    }
}
pub fn manifest_features(
    path: &str,
    value: &toml::Value,
    owned: &BTreeSet<String>,
) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    if !governed(path) {
        return out;
    }
    if let Some(features) = value.get("features").and_then(toml::Value::as_table) {
        if features.is_empty() || features.keys().any(|n| !feature_allowed(path, n)) {
            out.push(diagnostic(
                "BC9001",
                path,
                "one build forbids this features table",
            ));
        }
    }
    let explicit: BTreeSet<_> = value
        .get("features")
        .and_then(toml::Value::as_table)
        .into_iter()
        .flat_map(|t| t.values())
        .filter_map(toml::Value::as_array)
        .flatten()
        .filter_map(toml::Value::as_str)
        .filter_map(|s| s.strip_prefix("dep:"))
        .collect();
    fn inspect(
        path: &str,
        v: &toml::Value,
        owned: &BTreeSet<String>,
        explicit: &BTreeSet<&str>,
        out: &mut Vec<Diagnostic>,
    ) {
        match v {
            toml::Value::Table(t) => {
                for (key, v) in t {
                    if ["dependencies", "dev-dependencies", "build-dependencies"]
                        .contains(&key.as_str())
                    {
                        if let Some(deps) = v.as_table() {
                            for (name, d) in deps {
                                let package = d
                                    .get("package")
                                    .and_then(toml::Value::as_str)
                                    .unwrap_or(name);
                                if d.get("features").is_some() && owned.contains(package)
                                    || d.get("optional").and_then(toml::Value::as_bool)
                                        == Some(true)
                                        && !feature_allowed(path, name)
                                        && !explicit.contains(name.as_str())
                                {
                                    out.push(diagnostic(
                                        "BC9001",
                                        path,
                                        format!(
                                            "one build forbids dependency features on `{name}`"
                                        ),
                                    ));
                                }
                            }
                        }
                    }
                    if key == "required-features"
                        && v.as_array().is_some_and(|a| {
                            a.iter()
                                .any(|v| !feature_allowed(path, v.as_str().unwrap_or("")))
                        })
                    {
                        out.push(diagnostic(
                            "BC9001",
                            path,
                            "one build forbids required-features",
                        ));
                    }
                    inspect(path, v, owned, explicit, out);
                }
            }
            toml::Value::Array(a) => {
                for v in a {
                    inspect(path, v, owned, explicit, out);
                }
            }
            _ => {}
        }
    }
    inspect(path, value, owned, &explicit, &mut out);
    out
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn identifier_denials_respect_exclusions_and_token_boundaries() {
        let input = Inputs::fixture(&[("crates/a/src/lib.rs", "Forbidden; NotForbidden; // Forbidden"), ("crates/a/src/allowed.rs", "Forbidden;")], "[[source_identifier_denials]]\nroot='crates/a/src'\nforbidden_identifiers=['Forbidden']\nexclude_paths=['crates/a/src/allowed.rs']");
        let out = check(&input);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].subject, "crates/a/src/lib.rs");
        assert!(check(&Inputs::fixture(
            &[("crates/a/src/lib.rs", "NotForbidden;")],
            ""
        ))
        .is_empty());
    }
    #[test]
    fn feature_predicates_are_detected_without_matching_quoted_examples() {
        assert_eq!(
            check(&Inputs::fixture(
                &[(
                    "crates/a/src/lib.rs",
                    "#[cfg(feature = \"extra\")] fn a() {}"
                )],
                ""
            ))
            .len(),
            1
        );
        assert!(check(&Inputs::fixture(
            &[(
                "crates/a/src/lib.rs",
                "let s = r#\"cfg(feature = 'extra')\"#;"
            )],
            ""
        ))
        .is_empty());
    }
    #[test]
    fn local_clippy_configuration_carries_every_root_entry() {
        let a = toml::from_str("x = ['a']").unwrap();
        assert!(superset(&a, &toml::from_str("x = ['a', 'b']").unwrap()));
        assert!(!superset(&a, &toml::from_str("x = ['b']").unwrap()));
    }
    #[test]
    fn features_are_refused_except_for_declared_signal_profiles() {
        let v = toml::from_str("[features]\nprofile-compact = []").unwrap();
        assert!(
            manifest_features("crates/worth-signal/Cargo.toml", &v, &BTreeSet::new()).is_empty()
        );
        assert_eq!(
            manifest_features("crates/other/Cargo.toml", &v, &BTreeSet::new()).len(),
            1
        );
    }
}
