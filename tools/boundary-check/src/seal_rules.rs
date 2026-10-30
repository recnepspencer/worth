//! Closed caller sets for named test controls and certification factories.
use crate::{
    config::*,
    inputs::Inputs,
    lexical::{close, tokens, Token},
};
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
    (name.ends_with("_for_test") || name.ends_with("ForTest"))
        && name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || !c.is_ascii())
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
fn generic_end(t: &[Token<'_>], open: usize) -> usize {
    let (mut at, mut depth) = (open + 1, 1usize);
    while at < t.len() {
        match t[at].text {
            "<" => depth += 1,
            ">" if t[at - 1].text != "-" => {
                depth -= 1;
                if depth == 0 {
                    return at;
                }
            }
            "(" | "[" | "{" => at = close(t, at),
            _ => {}
        }
        at += 1;
    }
    t.len()
}
// Definition positions establish ownership; every reference is still searched.
fn declarations<'a>(t: &[Token<'a>]) -> (Vec<&'a str>, Vec<&'a str>) {
    let (mut names, mut aliases) = (Vec::new(), Vec::new());
    let mut generic_until = 0;
    for (i, w) in t.windows(2).enumerate() {
        match w[0].text {
            "fn" | "struct" | "enum" | "trait" | "type" | "const" | "static" | "mod" | "union"
            | "macro" | "!" | "use" | "extern" | "impl" => {}
            _ => continue,
        }
        if w[0].end - w[0].start != w[0].text.len() || i > 0 && t[i - 1].text == "'" {
            continue; // Raw identifiers and lifetime names are not item keywords.
        }
        let header = i + if w[0].text == "impl" { 1 } else { 2 };
        if ["fn", "struct", "enum", "trait", "type", "union", "impl"].contains(&w[0].text)
            && t.get(header).is_some_and(|t| t.text == "<")
        {
            generic_until = generic_until.max(generic_end(t, header));
        }
        if w[0].text != "use"
            && !w[1]
                .text
                .chars()
                .next()
                .is_some_and(|c| c == '_' || c.is_alphabetic() || !c.is_ascii())
        {
            continue;
        }
        if w[0].text == "const"
            && (i < generic_until
                || i > 0 && t[i - 1].text == "*"
                || t.get(i + 2).is_none_or(|t| t.text != ":"))
        {
            continue;
        }
        if [
            "fn", "struct", "enum", "trait", "type", "const", "static", "mod", "union", "macro",
        ]
        .contains(&w[0].text)
            || w[0].text == "!" && i > 0 && t[i - 1].text == "macro_rules"
        {
            names.push(if w[0].text == "static" && w[1].text == "mut" {
                t.get(i + 2).map_or("", |t| t.text)
            } else {
                w[1].text
            });
        }
        if ["struct", "enum", "union"].contains(&w[0].text) {
            let mut at = i + 2;
            while at < t.len() && t[at].text != ";" {
                match t[at].text {
                    "<" => at = generic_end(t, at),
                    "{" => {
                        members(t, at, w[0].text == "enum", &mut names);
                        break;
                    }
                    "(" | "[" => at = close(t, at),
                    _ => {}
                }
                at += 1;
            }
        }
        if w[0].text == "use" || w[0].text == "extern" && w[1].text == "crate" {
            let end = (i + 1..t.len())
                .find(|j| t[*j].text == ";")
                .unwrap_or(t.len());
            aliases.extend(
                t[i..end]
                    .windows(2)
                    .filter(|w| w[0].text == "as")
                    .map(|w| w[1].text),
            );
        }
    }
    names.extend(
        t.windows(5)
            .filter(|w| {
                w[0].text == "#"
                    && w[1].text == "["
                    && w[2].text == "proc_macro_derive"
                    && w[3].text == "("
            })
            .map(|w| w[4].text),
    );
    names.retain(|n| test_name(n));
    aliases.retain(|n| test_name(n));
    (names, aliases)
}
fn members<'a>(t: &[Token<'a>], open: usize, variants: bool, names: &mut Vec<&'a str>) {
    let (mut at, mut first) = (open + 1, true);
    while at < close(t, open) {
        if t[at].text == "<" {
            at = generic_end(t, at) + 1;
            continue;
        }
        if t[at].text == "#" && t.get(at + 1).is_some_and(|t| t.text == "[") {
            at = close(t, at + 1) + 1;
            continue;
        }
        if variants && first && t[at].text != "," {
            names.push(t[at].text);
            if t.get(at + 1).is_some_and(|t| t.text == "{") {
                members(t, at + 1, false, names);
            }
            first = false;
        } else if !variants
            && t.get(at + 1).is_some_and(|t| t.text == ":")
            && t.get(at + 2).is_none_or(|t| t.text != ":")
        {
            names.push(t[at].text);
        }
        if t[at].text == "," {
            first = true;
        }
        at = close(t, at) + 1;
    }
}
pub fn check(input: &Inputs) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    let sources: Vec<_> = input
        .files
        .iter()
        .filter(|(p, s)| {
            p.ends_with(".rs")
                && (s.contains("_for_test")
                    || s.contains("ForTest")
                    || FACTORIES.iter().any(|n| s.contains(n)))
        })
        .map(|(p, s)| (p, tokens(s)))
        .collect();
    let mut doors: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut renamed = Vec::new();
    for (path, t) in &sources {
        if let Some(owner) = owner(input, path) {
            let (names, aliases) = declarations(t);
            for name in names {
                doors.entry(name.into()).or_default().insert(owner.clone());
            }
            renamed.extend(aliases.into_iter().map(|name| (name, owner.clone())));
        }
    }
    // Re-exporting an existing name cannot adopt its original owner's door.
    for (name, owner) in renamed {
        doors
            .entry(name.into())
            .or_insert_with(|| BTreeSet::from([owner]));
    }
    let controls: BTreeSet<_> = pairs(input.text(&format!("{CONFIG}control-callers.txt")))
        .into_iter()
        .collect();
    let factories: BTreeSet<_> =
        pairs(input.text(&format!("{CONFIG}certification-construction-callers.txt")))
            .into_iter()
            .collect();
    for (path, t) in &sources {
        let names: BTreeSet<_> = t
            .iter()
            .map(|t| t.text)
            .filter(|name| test_name(name) || FACTORIES.contains(name))
            .collect();
        for name in &names {
            if let Some(owners) = doors.get(*name) {
                if !permitted(path, owners, &controls) {
                    out.push(diagnostic(
                        Code::Control,
                        path,
                        format!("control `{name}` requires its owner or an exact caller"),
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
                        format!("factory `{name}` requires its owner or an exact caller"),
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
            "pub const arm_for_test: usize = 0;",
            "pub static arm_for_test: usize = 0;",
            "pub struct Control<T: Fn() -> Option<()>, const N: usize = { 1 }> { pub arm_for_test: (), marker: std::marker::PhantomData<T> }",
            "fn\u{200e}arm_for_test() {}",
            "static mut arm_for_test: () = ();",
            "pub macro arm_for_test() {}",
            "pub struct Control { #[cfg(test)] pub arm_for_test: () }",
            "union Control { arm_for_test: () }",
            "enum Control { arm_for_test }",
            "enum Control { Armed { arm_for_test: () } }",
            "pub use ordinary as arm_for_test;",
            "pub use {crate::ordinary as arm_for_test};",
            "pub use ::ordinary as arm_for_test;",
            "extern crate ordinary as arm_for_test;",
            "#[proc_macro_derive(arm_for_test)] fn derive() {}",
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
                    "// DoorForTest\nlet text = r#\"DoorForTest\"#; noop!(\"ignored\"DoorForTest); /* DoorForTest */",
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
    fn field_initializers_and_renamed_imports_cannot_claim_a_foreign_door() {
        for reference in [
            "fn build() { Control { arm_for_test: () }; }",
            "use owner::Control as r#struct; fn build() { r#struct { arm_for_test: () }; }",
            "use owner::arm_for_test as bridge_for_test;",
            "use owner::arm_for_test as arm_for_test;",
            "use owner::ControlForTest; fn borrow(_: &'static ControlForTest) {}",
            "use owner::ControlForTest; fn borrow(_: *const ControlForTest) {}",
            "#[cfg(any())] struct Binder { field: Box<dyn owner::Trait<ControlForTest: Copy>> } fn borrow(_: owner::ControlForTest) {}",
            "macro_rules! noop { ($lit:literal $name:ident) => {} } noop!(\"ignored\"struct ControlForTest);",
            "fn union(_: owner::Control) {} fn build() { union(owner::Control { arm_for_test: () }); }",
            "fn borrow<T: Fn(<owner::ControlForTest as Trait>::Item), const ControlForTest: usize>(_: owner::ControlForTest) {}",
            "fn borrow<const ControlForTest: usize>(_: owner::ControlForTest) {}",
            "fn borrow<#[allow(non_upper_case_globals)] const ControlForTest: usize>(_: owner::ControlForTest) {}",
        ] {
            let mut input = Inputs::fixture(
                &[
                    (
                        "owner/src/lib.rs",
                        "pub struct Control { pub arm_for_test: () } pub struct ControlForTest;",
                    ),
                    ("app/src/lib.rs", reference),
                ],
                "",
            );
            assert_eq!(check(&input).len(), 1, "{reference}");
            input.files.insert(
                format!("{CONFIG}control-callers.txt"),
                "owner/ app/src/lib.rs".into(),
            );
            assert!(check(&input).is_empty(), "{reference}");
        }
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
