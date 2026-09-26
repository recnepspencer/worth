use super::super::tests::rule;
use super::super::validate_source_owner_isolations;
use super::TestSources;

fn sources(files: &[(&str, &str)]) -> TestSources {
    let parsed = files
        .iter()
        .map(|(path, source)| ((*path).to_owned(), syn::parse_file(source).unwrap()))
        .collect::<Vec<_>>();
    TestSources::new(&parsed)
}

#[test]
fn a_test_only_path_to_a_production_file_exempts_nothing() {
    let tests = sources(&[
        (
            "owner/lane.rs",
            "mod kernel; #[cfg(test)] #[path = \"lane/kernel.rs\"] mod checks;",
        ),
        ("owner/lane/kernel.rs", "pub struct Kernel;"),
        (
            "owner/other.rs",
            "#[cfg(test)] #[path = \"../owner/lane/kernel.rs\"] mod again;",
        ),
    ]);
    assert!(!tests.contains("owner/lane/kernel.rs"));
    assert!(!tests.contains("owner/lane.rs"));
}

#[test]
fn a_file_whose_parent_is_outside_the_owner_stays_production() {
    // `lane.rs` is declared by a parent outside the owner roots, so a
    // test-only declaration inside them does not make it a test.
    let tests = sources(&[
        ("owner/lane.rs", "pub struct Kernel;"),
        (
            "owner/other.rs",
            "#[cfg(test)] #[path = \"lane.rs\"] mod checks;",
        ),
    ]);
    assert!(!tests.contains("owner/lane.rs"));
}

#[test]
fn files_an_inline_test_module_declares_are_tests() {
    let tests = sources(&[
        (
            "owner/lane.rs",
            "mod live { mod kept; } #[cfg(test)] mod checks { mod fixture; mod deep { mod leaf; } }",
        ),
        ("owner/lane/live/kept.rs", "pub struct Kept;"),
        ("owner/lane/checks/fixture.rs", "pub struct Fixture;"),
        ("owner/lane/checks/deep/leaf.rs", "pub struct Leaf;"),
    ]);
    assert!(!tests.contains("owner/lane/live/kept.rs"));
    assert!(tests.contains("owner/lane/checks/fixture.rs"));
    assert!(tests.contains("owner/lane/checks/deep/leaf.rs"));
}

#[test]
fn a_test_declaration_inside_an_inline_module_is_found() {
    let tests = sources(&[
        (
            "owner/mod.rs",
            "mod live { #[cfg(test)] mod checks; #[cfg(test)] #[path = \"other.rs\"] mod pathed; }",
        ),
        ("owner/live/checks.rs", "pub struct Checks;"),
        ("owner/live/other.rs", "pub struct Other;"),
    ]);
    assert!(tests.contains("owner/live/checks.rs"));
    assert!(tests.contains("owner/live/other.rs"));
    assert!(!tests.contains("owner/mod.rs"));
}

#[test]
fn a_file_below_a_test_file_is_a_test_and_a_crate_root_is_not() {
    let tests = sources(&[
        ("crate/src/lib.rs", "mod lane; #[cfg(test)] mod tests;"),
        ("crate/src/lane.rs", "pub struct Lane;"),
        ("crate/src/tests.rs", "mod helpers;"),
        ("crate/src/tests/helpers.rs", "pub struct Helper;"),
        ("crate/src/tests/orphan.rs", "pub struct Orphan;"),
    ]);
    assert!(!tests.contains("crate/src/lib.rs"));
    assert!(!tests.contains("crate/src/lane.rs"));
    for test in [
        "crate/src/tests.rs",
        "crate/src/tests/helpers.rs",
        "crate/src/tests/orphan.rs",
    ] {
        assert!(tests.contains(test), "{test}");
    }
}

#[test]
fn a_cfg_test_module_declaration_exempts_its_files_and_nothing_else() {
    let workspace =
        std::env::temp_dir().join(format!("boundary-owner-tests-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&workspace);
    for directory in [
        "owner/state/tests",
        "owner/audit",
        "owner/progress",
        "guarded",
    ] {
        std::fs::create_dir_all(workspace.join(directory)).unwrap();
    }
    for (file, source) in [
        (
            "owner/state.rs",
            "#[cfg(test)] mod tests; #[cfg(test)] #[path = \"state/checks.rs\"] mod checks;              mod audit_hook; pub struct Kernel;",
        ),
        ("owner/state/tests.rs", "pub struct Harness;"),
        ("owner/state/tests/nested.rs", "pub struct NestedHarness;"),
        ("owner/state/checks.rs", "pub struct Checks;"),
        ("owner/audit.rs", "pub mod tests;"),
        ("owner/audit/tests.rs", "pub struct AuditRecord;"),
        ("owner/progress.rs", "#[cfg(test)] mod scaling;"),
        ("owner/progress/scaling.rs", "pub struct ScalingFixture;"),
        (
            "guarded/run.rs",
            "fn f(_: Kernel, _: Harness, _: NestedHarness, _: Checks,                  _: AuditRecord, _: ScalingFixture) {}",
        ),
    ] {
        std::fs::write(workspace.join(file), source).unwrap();
    }
    let found = validate_source_owner_isolations(&workspace, &[rule()]);
    std::fs::remove_dir_all(&workspace).unwrap();
    let messages = found.iter().map(|d| d.message()).collect::<Vec<_>>();
    assert_eq!(found.len(), 2, "{messages:?}");
    for reached in ["`Kernel`", "`AuditRecord`"] {
        assert!(
            messages.iter().any(|m| m.contains(reached)),
            "{reached}: {messages:?}"
        );
    }
}
