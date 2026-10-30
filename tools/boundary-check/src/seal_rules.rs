//! Closed caller sets for exported test controls and certification factories.
use crate::{config::*, inputs::Inputs, lexical::tokens, runtime_courts::scopes};
use std::collections::{BTreeMap, BTreeSet};
mod regions;
const OWNERS: &[&str] = &[
    "crates/worth-relational",
    "crates/worth-execution",
    "crates/worth-signal",
    "crates/worth-signal-wasm",
    "crates/worth-runtime-bridge",
    "crates/worth-runtime-world",
    "workspaces/worth-query",
    "workspaces/worth-query-bank-world",
    "crates/worth-cad-entry",
];
const FACTORIES: &[&str] = &[
    "registered_authoritative_source",
    "with_authoritative_source",
    "from_owner_publication",
    "from_owner_partition_publication",
    "certification_query",
];
fn owner(path: &str) -> Option<String> {
    if under(path, "crates/worth-cad-entry") {
        return Some(
            "crates/worth-cad-entry/src/contribution/installation_omission_for_test.rs".into(),
        );
    }
    OWNERS.iter().find(|o| under(path, o)).and_then(|o| {
        if o.starts_with("workspaces/") {
            path.strip_prefix(&format!("{o}/crates/"))
                .and_then(|s| s.split('/').next())
                .map(|n| format!("{o}/crates/{n}/"))
        } else {
            Some(format!("{o}/"))
        }
    })
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
    let tests = scopes(input);
    let mut out = Vec::new();
    let mut doors: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for (path, text) in input.files.iter().filter(|(p, _)| {
        p.ends_with(".rs")
            && p.contains("/src/")
            && !p.contains("/fixtures/")
            && !p.contains("/examples/")
    }) {
        if !text.contains("_for_test") && !text.contains("ForTest") {
            continue;
        }
        let Some(owner) = owner(path) else {
            continue;
        };
        let t = tokens(text);
        let mut ranges = tests.get(path).cloned().unwrap_or_default();
        ranges.extend(regions::test_items(&t));
        let declared = regions::declared(&t, &ranges);
        if regions::ambiguous_macro(&t, &declared, &ranges) {
            out.push(diagnostic(
                "BC7012",
                path,
                "macro control names require a literal exported declaration",
            ));
        }
        for name in declared {
            doors.entry(name).or_default().insert(owner.clone());
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
        let mut ranges = tests.get(path).cloned().unwrap_or_default();
        ranges.extend(regions::test_items(&t));
        let control_source = owner(path).is_some_and(|owner| {
            regions::declared(&t, &ranges)
                .iter()
                .any(|name| doors.get(name).is_some_and(|o| o.contains(&owner)))
        });
        let excluded = if control_source {
            regions::ordinary_ranges(&t)
        } else {
            Vec::new()
        };
        let control_names: BTreeSet<_> = t
            .iter()
            .filter(|t| !excluded.iter().any(|(a, b)| t.start >= *a && t.start < *b))
            .map(|t| t.text)
            .collect();
        let names: BTreeSet<_> = t.iter().map(|t| t.text).collect();
        for name in &names {
            if let Some(owners) = doors.get(*name) {
                if control_names.contains(name) && !permitted(path, owners, &controls) {
                    out.push(diagnostic(
                        "BC7012",
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
                        "BC7013",
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
    fn composed_controls_and_reexporting_owners_keep_their_existing_reach() {
        let mut input = Inputs::fixture(
            &[
                ("crates/worth-signal/src/a.rs", "pub fn arm_for_test() {}"),
                (
                    "crates/worth-runtime-bridge/src/a.rs",
                    "pub fn bridge_for_test() { arm_for_test(); }",
                ),
            ],
            "",
        );
        assert!(check(&input).is_empty());
        input.files.insert(
            "crates/worth-runtime-bridge/src/a.rs".into(),
            "pub use signal::arm_for_test;".into(),
        );
        assert!(check(&input).is_empty());
        input.files.insert(
            "crates/worth-runtime-bridge/src/a.rs".into(),
            "pub fn bridge_for_test() {} fn ordinary() { arm_for_test(); }".into(),
        );
        assert!(!check(&input).is_empty());
    }
    #[test]
    fn test_only_methods_do_not_seal_unrelated_namesakes() {
        let input = Inputs::fixture(
            &[
                (
                    "crates/worth-signal/src/a.rs",
                    "#[cfg(test)] impl Private { pub fn arm_for_test() {} }",
                ),
                ("other/src/a.rs", "arm_for_test();"),
            ],
            "",
        );
        assert!(check(&input).is_empty());
    }
    #[test]
    fn macro_controls_require_literal_exported_declarations() {
        let good = Inputs::fixture(
            &[(
                "crates/worth-signal/src/a.rs",
                "declare! { pub struct DoorForTest; }",
            )],
            "",
        );
        assert!(check(&good).is_empty());
        let bad = Inputs::fixture(
            &[("crates/worth-signal/src/a.rs", "declare!(DoorForTest);")],
            "",
        );
        assert_eq!(check(&bad).len(), 1);
    }
    #[test]
    fn exported_control_references_require_an_exact_caller() {
        let mut input = Inputs::fixture(
            &[
                (
                    "crates/worth-signal/src/control.rs",
                    "pub fn arm_for_test() {}",
                ),
                ("app/src/lib.rs", "arm_for_test();"),
            ],
            "",
        );
        assert_eq!(
            check(&input).iter().filter(|d| d.code == "BC7012").count(),
            1
        );
        input.files.insert(
            format!("{CONFIG}control-callers.txt"),
            "crates/worth-signal/ app/src/lib.rs".into(),
        );
        assert!(check(&input).is_empty());
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
    #[test]
    fn raw_identifiers_do_not_bypass_the_closed_control_caller_set() {
        let mut input = Inputs::fixture(
            &[
                (
                    "crates/worth-signal/src/control.rs",
                    "pub fn r#arm_for_test() {}",
                ),
                ("app/src/lib.rs", "arm_for_test();"),
            ],
            "",
        );
        assert_eq!(
            check(&input).iter().filter(|d| d.code == "BC7012").count(),
            1
        );
        input.files.insert(
            format!("{CONFIG}control-callers.txt"),
            "crates/worth-signal/ app/src/lib.rs".into(),
        );
        assert!(check(&input).is_empty());
    }
}
