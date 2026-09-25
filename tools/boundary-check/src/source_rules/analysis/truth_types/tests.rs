use super::super::production_scope::ProductionModule;
use super::{disallowed_methods_in, lifecycle_default, lint_levels, sealed_construction};
use crate::config::{
    Road1Config, SealedMintCallerConfig, SealedMintConfig, SealedTruthTypeConfig,
    TruthTypeDenialConfig,
};
use std::collections::BTreeSet;

const OWNER: &str = "
    pub struct Rect { x: f32 }
    impl Rect {
        pub(crate) fn published(x: f32) -> Self { Self { x } }
        pub(crate) fn maybe(x: f32) -> Option<Rect> { Some(Self { x }) }
        pub(crate) const ZERO: Rect = Rect { x: 0.0 };
        pub(crate) fn translated(self, by: f32) -> Self { Self { x: self.x + by } }
        pub(crate) fn describe() -> String { String::new() }
    }
    pub enum Hit { Inside(Rect), Outside }
";

fn rule(mints: Vec<SealedMintConfig>, lifecycle_states: &[&str]) -> TruthTypeDenialConfig {
    TruthTypeDenialConfig {
        crate_root: "crates/covered".to_owned(),
        guidance: "construct through the owner".to_owned(),
        sealed: ["Rect", "Hit"]
            .into_iter()
            .map(|name| SealedTruthTypeConfig {
                name: name.to_owned(),
                requirement: 1,
                owners: vec!["src/geometry".to_owned()],
            })
            .collect(),
        mints,
        lifecycle_states: lifecycle_states
            .iter()
            .map(|name| (*name).to_owned())
            .collect(),
    }
}

fn mint(call: &str, clippy: Option<&str>, path: &str, items: &[&str]) -> SealedMintConfig {
    SealedMintConfig {
        call: call.to_owned(),
        clippy: clippy.map(str::to_owned),
        callers: vec![SealedMintCallerConfig {
            path: path.to_owned(),
            items: items.iter().map(|item| (*item).to_owned()).collect(),
        }],
        reason: "the host mints it".to_owned(),
    }
}

fn parsed(sources: &[(&str, &str)]) -> Vec<(String, syn::File)> {
    sources
        .iter()
        .map(|(path, text)| ((*path).to_owned(), syn::parse_file(text).unwrap()))
        .collect()
}

fn findings(user: &str, rule: &TruthTypeDenialConfig, disallowed: &[&str]) -> Vec<String> {
    let files = parsed(&[("src/geometry/rect.rs", OWNER), ("src/user.rs", user)]);
    let modules: Vec<_> = files
        .iter()
        .map(|(source, file)| ProductionModule {
            relative_source: source,
            attributes: &file.attrs,
            items: &file.items,
        })
        .collect();
    let disallowed: BTreeSet<String> = disallowed.iter().map(|path| (*path).to_owned()).collect();
    sealed_construction::check(&modules, rule, &disallowed)
        .into_iter()
        .chain(lifecycle_default::check(&modules, rule))
        .map(|diagnostic| format!("{}: {}", diagnostic.subject(), diagnostic.message()))
        .collect()
}

#[test]
fn owner_constructions_and_reads_outside_it_are_legal() {
    let user = "
        fn read(rect: &Rect, hit: Hit) -> f32 {
            let _ = Rect::describe();
            match hit { Hit::Inside(_) | Hit::Outside => rect.x }
        }
        fn shift(rect: Rect) -> Rect { rect.translated(1.0) }
        #[cfg(test)]
        fn fixture() -> Rect { Rect::published(0.0) }
        #[cfg(test)]
        #[allow(clippy::all)]
        mod tests;
        #[allow(dead_code, reason = \"no warnings\")]
        fn quiet() {}
    ";
    assert_eq!(
        findings(user, &rule(Vec::new(), &[]), &[]),
        Vec::<String>::new()
    );
}

#[test]
fn every_construction_outside_the_owner_is_rejected() {
    for (user, expected) in [
        (
            "fn f() -> Rect { Rect::published(1.0) }",
            "calls constructor `Rect::published`",
        ),
        (
            "fn f() -> Option<Rect> { Rect::maybe(1.0) }",
            "calls constructor `Rect::maybe`",
        ),
        (
            "fn f() -> Rect { Rect::ZERO }",
            "calls constructor `Rect::ZERO`",
        ),
        (
            "fn f() -> Hit { Hit::Outside }",
            "calls constructor `Hit::Outside`",
        ),
        (
            "fn f() -> Rect { <Rect>::published(1.0) }",
            "calls constructor `Rect::published`",
        ),
        (
            "fn f() { let _ = vec![Rect::published(1.0)]; }",
            "calls constructor `Rect::published`",
        ),
        (
            "fn f() -> Rect { Rect { x: 1.0 } }",
            "builds sealed `Rect` by its literal",
        ),
        (
            "impl Rect { fn forged() -> Self { todo!() } }",
            "implements for sealed `Rect`",
        ),
        (
            "impl Default for Hit { fn default() -> Self { todo!() } }",
            "implements for sealed `Hit`",
        ),
        (
            "use crate::geometry::Hit::*;",
            "imports through sealed `Hit`",
        ),
        (
            "use crate::geometry::Rect as Plain;",
            "renames sealed `Rect` to `Plain`",
        ),
        (
            "type Plain = crate::geometry::Rect;",
            "aliases sealed `Rect`",
        ),
        (
            "#[allow(clippy::disallowed_methods)] fn f(r: Rect) -> Rect { r.translated(1.0) }",
            "suppresses Clippy's disallowed methods outside every declared mint caller",
        ),
        (
            "#![allow(clippy::disallowed_methods)] fn f() {}",
            "the module suppresses Clippy's disallowed methods",
        ),
        (
            "#[allow(clippy::all)] mod inner;",
            "suppresses Clippy's disallowed methods outside every declared mint caller",
        ),
        (
            "#[cfg_attr(unix, allow(clippy::style))] fn f() {}",
            "suppresses Clippy's disallowed methods outside every declared mint caller",
        ),
        (
            "#[allow(warnings)] fn f() {}",
            "suppresses Clippy's disallowed methods outside every declared mint caller",
        ),
    ] {
        let found = findings(user, &rule(Vec::new(), &[]), &[]);
        assert_eq!(found.len(), 1, "{user}: {found:?}");
        assert!(
            found[0].starts_with("crates/covered/src/user.rs:"),
            "{found:?}"
        );
        assert!(found[0].contains(expected), "{user}: {found:?}");
    }
}

#[test]
fn a_declared_caller_admits_exactly_its_mint() {
    let user = "
        fn host() -> Rect { Rect::published(1.0) }
        fn other() -> Rect { Rect::published(2.0) }
    ";
    let rule = rule(
        vec![mint("Rect::published", None, "src/user.rs", &["host"])],
        &[],
    );
    let found = findings(user, &rule, &[]);
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(
        found[0].contains("`other` calls constructor `Rect::published`"),
        "{found:?}"
    );
}

#[test]
fn a_clippy_mint_is_admitted_only_by_expect_at_its_caller() {
    let clippy = Some("crate::geometry::Rect::translated");
    let user = "
        #[expect(clippy::disallowed_methods, reason = \"host\")]
        fn host(r: Rect) -> Rect { r.translated(1.0) }
    ";
    let declared = rule(
        vec![mint("Rect::translated", clippy, "src/user.rs", &["host"])],
        &[],
    );
    let disallowed = ["crate::geometry::Rect::translated"];
    assert_eq!(findings(user, &declared, &disallowed), Vec::<String>::new());

    let allowed = user.replace("expect", "allow");
    let found = findings(&allowed, &declared, &disallowed);
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(found[0].contains("with `allow`; use `expect`"), "{found:?}");

    let found = findings(user, &declared, &[]);
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(
        found[0].contains("clippy.toml does not disallow"),
        "{found:?}"
    );
}

#[test]
fn stale_declarations_are_reported() {
    let rule = rule(
        vec![
            mint("Rect::published", None, "src/user.rs", &["gone"]),
            mint("Rect::translated", None, "src/user.rs", &[]),
        ],
        &["Phase"],
    );
    let found = findings("fn f() {}", &rule, &[]);
    let expected = [
        "declared caller [\"gone\"] no longer mints `Rect::published`",
        "declared mint `Rect::translated` names no constructor",
        "declared caller [] no longer mints `Rect::translated`",
        "declared lifecycle state `Phase` is not defined in production",
    ];
    assert_eq!(found.len(), expected.len(), "{found:?}");
    for message in expected {
        assert!(
            found.iter().any(|finding| finding.contains(message)),
            "{message}: {found:?}"
        );
    }
}

#[test]
fn a_sealed_type_missing_from_its_owner_is_reported() {
    let mut rule = rule(Vec::new(), &[]);
    rule.sealed[0].owners = vec!["src/elsewhere".to_owned()];
    let found = findings("fn f() {}", &rule, &[]);
    assert!(
        found
            .iter()
            .any(|finding| finding.contains("`Rect` is not defined in its declared owners")),
        "{found:?}"
    );
}

#[test]
fn lifecycle_state_never_takes_a_default() {
    for (user, expected) in [
        ("#[derive(Clone, Default)] enum Phase { Idle }", "`Phase` derives Default"),
        (
            "#[cfg_attr(feature = \"x\", derive(Default))] struct Phase;",
            "`Phase` derives Default",
        ),
        (
            "enum Phase { Idle } impl std::default::Default for Phase { fn default() -> Self { Self::Idle } }",
            "`Phase` implements Default",
        ),
    ] {
        let found = findings(user, &rule(Vec::new(), &["Phase"]), &[]);
        assert_eq!(found.len(), 1, "{user}: {found:?}");
        assert!(found[0].contains(expected), "{user}: {found:?}");
    }
    let named = "#[derive(Clone)] enum Phase { Idle } #[derive(Default)] struct Cache;";
    assert_eq!(
        findings(named, &rule(Vec::new(), &["Phase"]), &[]),
        Vec::<String>::new()
    );
}

#[test]
fn manifest_lints_may_not_allow_what_silences_disallowed_methods() {
    let lints: toml::Value = toml::from_str(
        "[clippy]\n\
         all = { level = \"deny\", priority = -1 }\n\
         style = { level = \"allow\", priority = -1 }\n\
         disallowed_methods = \"allow\"\n\
         [rust]\n\
         warnings = \"allow\"\n\
         unsafe_code = \"allow\"\n",
    )
    .unwrap();
    assert_eq!(
        lint_levels::relaxed_in(&lints),
        [
            "clippy::disallowed_methods",
            "clippy::style",
            "rust::warnings"
        ]
    );
    let strict: toml::Value = toml::from_str("[clippy]\nall = \"deny\"\n").unwrap();
    assert!(lint_levels::relaxed_in(&strict).is_empty());
}

#[test]
fn inherited_lints_are_read_from_the_workspace_manifest() {
    let root = std::env::temp_dir().join(format!("bc-lint-levels-{}", std::process::id()));
    let member = root.join("workspace/crates/member");
    std::fs::create_dir_all(&member).unwrap();
    std::fs::write(
        root.join("workspace/Cargo.toml"),
        "[workspace]\n[workspace.lints.clippy]\nstyle = \"allow\"\n",
    )
    .unwrap();
    std::fs::write(member.join("Cargo.toml"), "[lints]\nworkspace = true\n").unwrap();
    let inherited = lint_levels::relaxed(&root, "workspace/crates/member");
    std::fs::write(member.join("Cargo.toml"), "[package]\nname = \"member\"\n").unwrap();
    let own = lint_levels::relaxed(&root, "workspace/crates/member");
    std::fs::remove_file(root.join("workspace/Cargo.toml")).unwrap();
    std::fs::write(member.join("Cargo.toml"), "[lints]\nworkspace = true\n").unwrap();
    let orphaned = lint_levels::relaxed(&root, "workspace/crates/member");
    std::fs::remove_dir_all(&root).unwrap();

    assert_eq!(inherited.unwrap(), ["clippy::style"]);
    assert!(own.unwrap().is_empty());
    assert!(orphaned.is_err());
}

#[test]
fn disallowed_methods_read_both_entry_forms() {
    let text = "disallowed-methods = [\"a::b\", { path = \"c::d\", reason = \"why\" }]";
    let paths = disallowed_methods_in(text).unwrap();
    assert_eq!(paths.into_iter().collect::<Vec<_>>(), ["a::b", "c::d"]);
    assert!(disallowed_methods_in("disallowed-methods = [{ reason = \"x\" }]").is_err());
    assert!(disallowed_methods_in("too-many-arguments-threshold = 32")
        .unwrap()
        .is_empty());
}

#[test]
fn every_configured_clippy_mint_is_disallowed_in_clippy_toml() {
    let config: Road1Config =
        toml::from_str(include_str!("../../../../config/road1.toml")).unwrap();
    let disallowed = disallowed_methods_in(include_str!("../../../../../../clippy.toml")).unwrap();
    assert!(!config.truth_type_denials.is_empty());
    for rule in &config.truth_type_denials {
        for path in rule.mints.iter().filter_map(|mint| mint.clippy.as_ref()) {
            assert!(disallowed.contains(path), "{}: {path}", rule.crate_root);
        }
    }
}
