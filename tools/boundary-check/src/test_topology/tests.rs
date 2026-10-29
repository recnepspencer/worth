use super::*;
mod fixture;
mod source_ratchet;
mod workspace_membership;
use fixture::{Fixture, COURT, TARGET};

#[test]
fn law18_v2_refuses_tests_directory_in_a_product_crate() {
    let f = Fixture::new();
    f.write("governed/Cargo.toml", "[package]\nname='product'\n");
    f.refuses("product crate must not contain a tests/");
}
#[test]
fn law18_v2_refuses_product_integration_targets_outside_tests_directory() {
    let f = Fixture::new();
    f.write(
        "governed/Cargo.toml",
        "[package]\nname='product'\n[[test]]\nname='outside'\npath='outside.rs'\n",
    );
    f.refuses("product crate cannot declare integration targets");
}
#[test]
fn court_requires_autotests_false() {
    let f = Fixture::new();
    f.manifest("autotests=true", "", "host");
    f.refuses("autotests = false");
}
#[test]
fn law18_refuses_second_explicit_target_even_with_autotests_disabled() {
    let f = Fixture::new();
    f.manifest(
        "autotests=false",
        "[[test]]\nname='extra'\npath='tests/extra/main.rs'\n",
        "host",
    );
    f.write("governed/tests/extra/main.rs", "#[test] fn extra() {}");
    f.refuses("exactly one integration-test target");
}
#[test]
fn court_refuses_wrong_canonical_target_path() {
    let f = Fixture::new();
    let path = f.0.join("governed/Cargo.toml");
    let text = std::fs::read_to_string(&path)
        .unwrap()
        .replace(&format!("tests/{TARGET}.rs"), "tests/other.rs");
    f.write("governed/Cargo.toml", &text);
    f.write("governed/tests/other.rs", "#[test] fn ordinary() {}");
    f.refuses("canonical explicit integration target");
}
#[test]
fn law18_v2_refuses_a_misnamed_court() {
    let f = Fixture::new();
    f.manifest("autotests=false", "", "other");
    f.refuses("misnamed court");
}
#[test]
fn manifest_classes_are_retired() {
    let f = Fixture::new();
    f.append(
        "governed/Cargo.toml",
        "[package.metadata.worth.court]\nsurface='host'\nclass='fast'\n",
    );
    // Replace, rather than duplicate, the metadata table.
    let text = std::fs::read_to_string(f.0.join("governed/Cargo.toml")).unwrap();
    let start = text.find("[package.metadata.worth.court]").unwrap();
    f.write(
        "governed/Cargo.toml",
        &format!(
            "{}[package.metadata.worth.court]\nsurface='host'\nclass=['fast','process']\n",
            &text[..start]
        ),
    );
    f.refuses("categories belong in module paths");
}
#[test]
fn law18_v2_refuses_product_code_in_a_court() {
    let f = Fixture::new();
    f.write("governed/src/lib.rs", "pub fn product() {} ");
    f.refuses("court contains product code");
}
#[test]
fn law18_v2_refuses_bench_targets_hidden_inside_tests() {
    let f = Fixture::new();
    f.manifest(
        "autotests=false",
        "[[bench]]\nname='hidden'\npath='tests/bench.rs'\nharness=false\n",
        "host",
    );
    f.write("governed/tests/bench.rs", "fn main() {}");
    f.refuses("court contains product code or a production target");
}
#[test]
fn court_cannot_disable_harness_or_test_execution() {
    for field in ["harness=false", "test=false"] {
        let f = Fixture::new();
        let text = std::fs::read_to_string(f.0.join("governed/Cargo.toml")).unwrap();
        f.write(
            "governed/Cargo.toml",
            &text.replace(
                &format!("name='{TARGET}'"),
                &format!("name='{TARGET}'\n{field}"),
            ),
        );
        f.refuses("standard harness");
    }
}
#[test]
fn executable_exception_registry_cannot_bypass_one_target() {
    let f = Fixture::new();
    f.write(
        EXCEPTIONS,
        "governed/Cargo.toml | child | former executable\n",
    );
    f.refuses("no executable exceptions");
}
#[test]
fn ordinary_and_each_category_can_share_one_binary() {
    let f = Fixture::new();
    f.entry("#[test] fn ordinary() {} mod process { mod serial { #[test] fn isolated() {} } } mod oracle { #[test] fn model() {} } mod cost { #[test] fn growth() {} } mod determinism { #[test] fn replay() {} } mod compile_fail { #[test] fn sealed() {} }");
    assert!(f.messages().is_empty(), "{:?}", f.messages());
}
#[test]
fn retired_and_nested_categories_are_refused() {
    for source in [
        "mod fast {}",
        "mod scale {}",
        "mod area { mod process {} }",
        "mod serial { #[test] fn isolated() {} }",
    ] {
        let f = Fixture::new();
        f.entry(source);
        f.refuses("invalid category path");
    }
}
#[test]
fn real_module_paths_and_wrapped_test_functions_are_checked() {
    let f = Fixture::new();
    f.entry("mod r#process { #[path=\"../leaf.rs\"] mod family; }");
    f.write("governed/tests/process/.keep", "");
    f.write(
        "governed/tests/leaf.rs",
        "wrapper! { #[test] fn phase_nine_old_id() {} }",
    );
    f.refuses("without an identifier");
}
#[test]
fn milestone_decision_ticket_and_phase_identifiers_are_refused() {
    for name in [
        "m3_old_id",
        "d17_old_id",
        "r8_old_id",
        "phase_nine_old_id",
        "ticket_123_old_id",
        "test_old_prefix",
        "it_old_prefix",
    ] {
        let f = Fixture::new();
        f.entry(&format!("#[test] fn {name}() {{}}"));
        f.refuses("without an identifier");
    }
}
#[test]
fn semantic_program_revision_is_not_a_milestone_identifier() {
    let f = Fixture::new();
    f.entry("#[test] fn p2_adoption_preserves_the_program_contract() {}");
    assert!(f.messages().is_empty(), "{:?}", f.messages());
}
#[test]
fn court_and_testkit_source_ceilings_are_deterministic() {
    for testkit in [false, true] {
        let f = Fixture::new();
        if testkit {
            f.write("governed/Cargo.toml", "[package]\nname='sample-testkit-host'\n[package.metadata.worth.testkit]\nworkspace='sample'\n");
            std::fs::remove_dir_all(f.0.join("governed/tests")).unwrap();
        }
        f.write("governed/large.rs", &"// source line\n".repeat(15_001));
        f.refuses("ceiling is 15000");
    }
}
#[test]
fn testkit_name_and_metadata_are_required() {
    let f = Fixture::new();
    f.write(
        "governed/Cargo.toml",
        "[package]\nname='wrong-name'\n[package.metadata.worth.testkit]\nworkspace='sample'\n",
    );
    f.refuses("misnamed testkit");
}
#[test]
fn normal_and_build_dependencies_cannot_import_testkits() {
    for kind in ["dependencies", "build-dependencies"] {
        let f = Fixture::new();
        f.write("governed/Cargo.toml", &format!("[package]\nname='product'\n[{kind}]\nfixture={{package='sample-testkit-host',path='../fixture'}}\n"));
        f.refuses("must be in dev-dependencies");
    }
}
#[test]
fn no_package_can_depend_on_a_court_even_in_dev_dependencies() {
    let f = Fixture::new();
    f.workspace();
    f.write("app/Cargo.toml", &format!("[package]\nname='app'\nversion='0.1.0'\n[dev-dependencies]\nproof={{package='{COURT}',path='../governed'}}\n"));
    f.write("app/src/lib.rs", "");
    f.refuses("must be a leaf");
}
#[test]
fn law18_v2_workspace_alias_cannot_hide_a_production_fixture_dependency() {
    let f = Fixture::new();
    f.workspace();
    f.append(
        "Cargo.toml",
        "[workspace.dependencies]\nfixture={package='sample-testkit-host',path='fixture'}\n",
    );
    f.write(
        "app/Cargo.toml",
        "[package]\nname='app'\nversion='0.1.0'\n[dependencies]\nfixture.workspace=true\n",
    );
    f.write("app/src/lib.rs", "");
    f.write("fixture/Cargo.toml", "[package]\nname='sample-testkit-host'\nversion='0.1.0'\n[package.metadata.worth.testkit]\nworkspace='sample'\n");
    f.write("fixture/src/lib.rs", "");
    f.refuses("product dependency `sample-testkit-host`");
}
#[test]
fn workspace_membership_does_not_annex_external_consumer_fixtures() {
    let f = Fixture::new();
    f.write("other/Cargo.toml", "[package]\nname='other'\n[workspace]\n");
    f.write("other/tests/untouched.rs", "#[test] fn outside_scope() {}");
    assert!(f.messages().is_empty(), "{:?}", f.messages());
}
#[test]
fn law18_checkout_cannot_disable_the_rule_by_removing_both_config_files() {
    let f = Fixture::new();
    f.write(".git", "gitdir: fixture\n");
    std::fs::remove_file(f.0.join(CONFIG)).unwrap();
    std::fs::remove_file(f.0.join(EXCEPTIONS)).unwrap();
    f.refuses("missing its test-topology configuration");
}
#[test]
fn query_gate_and_governed_nextest_filters_refuse_name_regexes() {
    for path in [
        "scripts/ci/query_gate.py",
        "workspaces/worth-query/.config/nextest.toml",
    ] {
        let f = Fixture::new();
        f.write(path, "filter = 'test(/behaviour_name/)' ");
        f.refuses("name regex");
        f.write(path, "filter = 'test(/^process::/)' ");
        assert!(f.messages().is_empty(), "{:?}", f.messages());
    }
}
