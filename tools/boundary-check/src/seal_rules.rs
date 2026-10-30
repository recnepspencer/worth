//! Closed caller sets for exported test controls and certification factories.
use crate::{config::*, inputs::Inputs, lexical::tokens};
use std::collections::{BTreeMap, BTreeSet};
const FACTORIES: &[&str] = &[
    "registered_authoritative_source",
    "with_authoritative_source",
    "from_owner_publication",
    "from_owner_partition_publication",
    "certification_query",
];
fn owner(input: &Inputs, path: &str) -> Option<String> {
    input
        .manifests
        .iter()
        .filter(|(_, value)| value.get("package").is_some())
        .filter_map(|(p, _)| p.strip_suffix("Cargo.toml"))
        .filter(|p| path.starts_with(p))
        .max_by_key(|p| p.len())
        .map(str::to_owned)
        .or_else(|| path.split_once("/src/").map(|(p, _)| format!("{p}/")))
}
fn test_name(name: &str) -> bool {
    name.ends_with("_for_test") || name.ends_with("ForTest")
}
fn permitted(path: &str, owners: &BTreeSet<String>, callers: &BTreeSet<(&str, &str)>) -> bool {
    owners.iter().any(|o| {
        if o.ends_with('/') {
            path.starts_with(o)
        } else {
            path == o
        }
    }) || owners.iter().any(|o| callers.contains(&(o.as_str(), path)))
}
pub fn check(input: &Inputs) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    let mut doors: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for (path, text) in input.files.iter().filter(|(p, _)| p.ends_with(".rs")) {
        if !text.contains("_for_test") && !text.contains("ForTest") {
            continue;
        }
        let Some(owner) = owner(input, path) else {
            continue;
        };
        let t = tokens(text);
        // Only literal declaration names establish an owner. Visibility and cfg
        // have no bearing on the name contract; bodies are searched in full.
        let declared = t
            .windows(2)
            .filter_map(|w| {
                ([
                    "fn", "struct", "enum", "trait", "type", "const", "static", "mod", "union",
                ]
                .contains(&w[0].text)
                    && test_name(w[1].text))
                .then_some(w[1].text)
            })
            .chain(t.windows(3).filter_map(|w| {
                (w[0].text == "macro_rules" && w[1].text == "!" && test_name(w[2].text))
                    .then_some(w[2].text)
            }));
        for name in declared {
            doors
                .entry(name.to_owned())
                .or_default()
                .insert(owner.clone());
        }
    }
    let controls: BTreeSet<_> = pairs(input.text(&format!("{CONFIG}control-callers.txt")))
        .into_iter()
        .collect();
    let factories: BTreeSet<_> =
        pairs(input.text(&format!("{CONFIG}certification-construction-callers.txt")))
            .into_iter()
            .collect();
    for (path, text) in input.files.iter().filter(|(p, _)| p.ends_with(".rs")) {
        if !text.contains("_for_test")
            && !text.contains("ForTest")
            && !FACTORIES.iter().any(|n| text.contains(n))
        {
            continue;
        }
        let t = tokens(text);
        let names: BTreeSet<_> = t.iter().map(|t| t.text).collect();
        for name in &names {
            if let Some(owners) = doors.get(*name) {
                if !permitted(path, owners, &controls) {
                    out.push(diagnostic(
                        Code::Control,
                        path,
                        format!(
                            "sealed control `{name}` is outside its owner and closed caller set"
                        ),
                    ));
                }
            }
            if FACTORIES.contains(name) {
                let (class, owner) = if *name == "certification_query" {
                    (
                        "installation",
                        "workspaces/worth-query/crates/worth-query-installation/",
                    )
                } else {
                    ("bridge", "crates/worth-runtime-bridge/")
                };
                if !path.starts_with(owner) && !factories.contains(&(class, path.as_str())) {
                    out.push(diagnostic(
                        Code::Construction,
                        path,
                        format!(
                            "sealed factory `{name}` is outside its owner and closed caller set"
                        ),
                    ));
                }
            }
        }
    }
    out
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_control_reference_requires_its_owner_or_an_exact_caller() {
        for declaration in [
            "fn arm_for_test() {}",
            "#[cfg(test)] impl Private { fn arm_for_test() {} }",
            "pub(in crate) fn r#arm_for_test() {}",
            "pub struct Other { #[cfg(test)] pub x: (), } pub fn arm_for_test() {}",
            "declare! { pub fn arm_for_test() {} }",
            "fn hidden() { #[macro_export] macro_rules! arm_for_test { () => {} } }",
        ] {
            let mut input = Inputs::fixture(
                &[
                    ("crates/worth-signal/src/control.rs", declaration),
                    ("app/src/lib.rs", "r#arm_for_test();"),
                ],
                "",
            );
            assert_eq!(check(&input).len(), 1, "{declaration}");
            input.files.insert(
                format!("{CONFIG}control-callers.txt"),
                "crates/worth-signal/ app/src/lib.rs".into(),
            );
            assert!(check(&input).is_empty(), "{declaration}");
        }
    }
    #[test]
    fn comments_and_literals_do_not_create_control_references() {
        let input = Inputs::fixture(
            &[
                ("crates/worth-signal/src/control.rs", "struct DoorForTest;"),
                (
                    "app/src/lib.rs",
                    "// DoorForTest\nlet text = r#\"DoorForTest\"#; /* DoorForTest */",
                ),
            ],
            "",
        );
        assert!(check(&input).is_empty());
    }
    #[test]
    fn cross_owner_composition_does_not_grant_ownership() {
        let input = Inputs::fixture(
            &[
                ("crates/worth-signal/src/control.rs", "fn arm_for_test() {}"),
                (
                    "crates/worth-runtime-bridge/src/lib.rs",
                    "fn bridge_for_test() { arm_for_test(); }",
                ),
            ],
            "",
        );
        assert_eq!(check(&input).len(), 1);
    }
    #[test]
    fn certification_factory_callers_are_class_and_file_exact() {
        let mut input = Inputs::fixture(&[("app/src/lib.rs", "certification_query();")], "");
        assert_eq!(check(&input).len(), 1);
        input.files.insert(
            format!("{CONFIG}certification-construction-callers.txt"),
            "installation app/src/lib.rs".into(),
        );
        assert!(check(&input).is_empty());
    }
}
