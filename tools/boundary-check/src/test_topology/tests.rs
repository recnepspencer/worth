use super::*;
use std::path::PathBuf;

const COURT: &str = "sample-court-host-fast";
const TARGET: &str = "sample_court_host_fast";
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let fixture = Self(worth_harness::tooling::fixture_path::unique_path(
            "test-topology",
        ));
        fixture.write(CONFIG, "governed | sample\n");
        fixture.write(EXCEPTIONS, "# no executable exceptions\n");
        fixture.manifest("", "");
        fixture.entry("mod fast { #[test] fn lawful() {} }");
        fixture
    }
    fn write(&self, relative: &str, text: &str) {
        let path = self.0.join(relative);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    }
    fn manifest(&self, package_fields: &str, targets: &str) {
        self.write("governed/Cargo.toml", &format!("[package]\nname = '{COURT}'\nversion = '0.1.0'\n{package_fields}\n[package.metadata.worth.court]\nboundary = 'host'\nclass = 'fast'\n{targets}"));
    }
    fn entry(&self, source: &str) {
        self.write(&format!("governed/tests/{TARGET}.rs"), source);
    }
    fn messages(&self) -> Vec<String> {
        validate(&self.0)
            .into_iter()
            .map(|d| d.message().to_owned())
            .collect()
    }
    fn refuses(&self, claim: &str) {
        assert!(
            self.messages().iter().any(|m| m.contains(claim)),
            "{:?}",
            self.messages()
        );
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let path = self.0.canonicalize().unwrap();
        let parent = self.0.parent().unwrap().canonicalize().unwrap();
        assert!(path.starts_with(&parent) && path != parent);
        std::fs::remove_dir_all(path).unwrap();
    }
}

#[test]
fn law18_refuses_a_second_implicit_target_and_directory_target() {
    let fixture = Fixture::new();
    fixture.write(
        "governed/tests/extra/main.rs",
        "mod fast { #[test] fn extra() {} }",
    );
    fixture.refuses("second integration-test target");
}
#[test]
fn law18_refuses_second_explicit_target_even_with_autotests_disabled() {
    let fixture = Fixture::new();
    fixture.manifest(
        "autotests = false",
        &format!(
            "[[test]]\nname = '{TARGET}'\n[[test]]\nname = 'hidden'\npath = 'tests/hidden.rs'\n"
        ),
    );
    fixture.write(
        "governed/tests/hidden.rs",
        "mod fast { #[test] fn hidden() {} }",
    );
    fixture.refuses("second integration-test target");
}
#[test]
fn law18_refuses_unclassed_root_inline_external_and_async_tests() {
    let fixture = Fixture::new();
    fixture.manifest(
        "autotests = false",
        &format!("[[test]]\nname = '{TARGET}'\n"),
    );
    fixture.entry("#[test] fn root() {}\nmod wrong { #[tokio::test] async fn async_case() {} }\n#[path = \"leaf.rs\"] mod leaf;");
    fixture.write("governed/tests/leaf.rs", "#[test] fn external() {}");
    for name in ["root", "async_case", "external"] {
        fixture.refuses(&format!("`{name}`"));
    }
}
#[test]
fn law18_uses_module_paths_including_path_attributes_and_raw_identifiers() {
    let fixture = Fixture::new();
    fixture.manifest(
        "autotests = false",
        &format!("[[test]]\nname = '{TARGET}'\n"),
    );
    fixture.entry("mod r#fast { #[path = \"../leaf.rs\"] mod scenario; mod serial { #[test] fn isolated() {} } }");
    fixture.write("governed/tests/fast/.keep", "");
    fixture.write("governed/tests/leaf.rs", "#[test] fn scenario() {}");
    assert!(fixture.messages().is_empty(), "{:?}", fixture.messages());
}
#[test]
fn law18_exception_requires_explicit_target_and_reason_and_must_exist() {
    let fixture = Fixture::new();
    fixture.write("governed/tests/child.rs", "fn main() {}");
    fixture.write(
        EXCEPTIONS,
        "governed/Cargo.toml | child | spawned executable\n",
    );
    fixture.refuses("explicit [[test]]");
    fixture.manifest("", "[[test]]\nname = 'child'\nharness = false\n");
    assert!(fixture.messages().is_empty(), "{:?}", fixture.messages());
    fixture.write(EXCEPTIONS, "governed/Cargo.toml | absent | own main\n");
    fixture.refuses("stale or ungoverned");
    fixture.write(EXCEPTIONS, "governed/Cargo.toml | child | \n");
    fixture.refuses("reason");
}
#[test]
fn law18_nested_fixture_workspace_is_governed_but_other_workspaces_are_not() {
    let fixture = Fixture::new();
    fixture.write(
        CONFIG,
        "governed | sample\ngoverned/fixtures/court | nested\n",
    );
    fixture.write("governed/fixtures/court/Cargo.toml", "[package]\nname = 'nested-court-host-fast'\n[workspace]\n[package.metadata.worth.court]\nboundary = 'host'\nclass = 'fast'\n");
    fixture.write(
        "governed/fixtures/court/tests/nested_court_host_fast.rs",
        "#[test] fn unclassed() {}",
    );
    fixture.write("other/Cargo.toml", "[package]\nname = 'other'\n");
    fixture.write("other/tests/other.rs", "#[test] fn excluded() {}");
    let messages = fixture.messages();
    assert_eq!(messages.len(), 1, "{messages:?}");
    assert!(messages[0].contains("unclassed"));
}
#[test]
fn law18_workspace_membership_does_not_annex_other_workspaces_or_fixtures() {
    let fixture = Fixture::new();
    fixture.write(CONFIG, ". | sample\n");
    fixture.write(
        "Cargo.toml",
        "[workspace]\nmembers = ['governed/member*']\nexclude = ['governed/member-excluded']\n",
    );
    for owner in ["member", "member-excluded"] {
        fixture.write(
            &format!("governed/{owner}/Cargo.toml"),
            "[package]\nname = 'product'\nversion = '0.1.0'\n",
        );
        fixture.write(&format!("governed/{owner}/src/lib.rs"), "");
        fixture.write(
            &format!("governed/{owner}/tests/forbidden.rs"),
            "#[test] fn case() {}",
        );
    }
    fixture.write(
        "other/Cargo.toml",
        "[package]\nname = 'other'\n[workspace]\n",
    );
    fixture.write("other/tests/other.rs", "#[test] fn untouched() {}");
    // The valid default court lies outside the workspace member glob.
    let messages = fixture.messages();
    assert_eq!(messages.len(), 1, "{messages:?}");
    assert!(messages[0].contains("product crate"));
}
#[test]
fn law18_implicit_path_dependency_members_cannot_hide_a_second_target() {
    let fixture = Fixture::new();
    fixture.write(CONFIG, ". | sample\n");
    fixture.write("Cargo.toml", "[workspace]\nmembers = ['owner']\n");
    fixture.write("owner/Cargo.toml", "[package]\nname = 'owner'\nversion = '0.1.0'\n[dependencies]\nimplicit = { path = '../implicit' }\n");
    fixture.write("owner/src/lib.rs", "");
    fixture.write("implicit/Cargo.toml", "[package]\nname = 'sample-court-host-fast'\nversion = '0.1.0'\n[lib]\npath = 'outside.rs'\n[package.metadata.worth.court]\nboundary = 'host'\nclass = 'fast'\n");
    fixture.write("implicit/outside.rs", "");
    for target in [TARGET, "extra"] {
        fixture.write(
            &format!("implicit/tests/{target}.rs"),
            "mod fast { #[test] fn case() {} }",
        );
    }
    fixture.refuses("second integration-test target");
}
#[test]
fn law18_harness_false_is_not_an_automatic_exemption() {
    let fixture = Fixture::new();
    fixture.manifest(
        "",
        &format!("[[test]]\nname = '{TARGET}'\nharness = false\n"),
    );
    fixture.entry("fn main() {}");
    fixture.refuses("harness=false");
}
#[test]
fn law18_conditional_attributes_and_macro_tests_cannot_hide_unclassed_cases() {
    let fixture = Fixture::new();
    fixture.entry("#[cfg_attr(test, test)] fn conditional() {}\nmacro_rules! generated { () => { #[test] fn hidden() {} }; }\ninclude!(\"uninspected.rs\");");
    let messages = fixture.messages();
    assert_eq!(messages.len(), 3, "{messages:?}");
    assert!(messages.iter().any(|m| m.contains("conditional")));
    assert!(messages.iter().any(|m| m.contains("generated")));
}
#[test]
fn law18_checkout_cannot_disable_the_rule_by_removing_both_config_files() {
    let fixture = Fixture::new();
    fixture.write(".git", "gitdir: fixture checkout marker\n");
    std::fs::remove_file(fixture.0.join(CONFIG)).unwrap();
    std::fs::remove_file(fixture.0.join(EXCEPTIONS)).unwrap();
    fixture.refuses("missing its test-topology configuration");
}
#[test]
fn law18_v2_refuses_tests_directory_in_a_product_crate() {
    let fixture = Fixture::new();
    fixture.write("governed/Cargo.toml", "[package]\nname = 'product'\n");
    fixture.refuses("product crate must not contain a tests/");
}
#[test]
fn law18_v2_refuses_a_misnamed_court() {
    let fixture = Fixture::new();
    let path = fixture.0.join("governed/Cargo.toml");
    let source = std::fs::read_to_string(&path)
        .unwrap()
        .replace(COURT, "sample-court-host-process");
    std::fs::write(path, source).unwrap();
    fixture.refuses("misnamed court");
}
#[test]
fn law18_v2_refuses_a_court_declaring_two_classes() {
    let fixture = Fixture::new();
    let path = fixture.0.join("governed/Cargo.toml");
    let source = std::fs::read_to_string(&path)
        .unwrap()
        .replace("class = 'fast'", "class = ['fast', 'process']");
    std::fs::write(path, source).unwrap();
    fixture.refuses("exactly one class");
}
#[test]
fn law18_v2_refuses_product_code_in_a_court() {
    let fixture = Fixture::new();
    fixture.write("governed/src/lib.rs", "pub fn product() -> u32 { 3 }");
    fixture.refuses("court contains product code");
}
#[test]
fn law18_v2_refuses_a_second_class_module_even_without_tests() {
    let fixture = Fixture::new();
    fixture.entry("mod fast { #[test] fn lawful() {} } mod process {}");
    fixture.refuses("second class module");
}

#[test]
fn law18_v2_products_cannot_consume_fixture_authority_as_a_normal_dependency() {
    let fixture = Fixture::new();
    fixture.write("governed/Cargo.toml", "[package]\nname = 'product'\n[dependencies]\nworld = { package = 'sample-worlds', path = '../world' }\n");
    fixture.refuses("must be in dev-dependencies");
    fixture.write("governed/Cargo.toml", "[package]\nname = 'product'\n[dev-dependencies]\nworld = { package = 'sample-worlds', path = '../world' }\n");
    assert!(!fixture
        .messages()
        .iter()
        .any(|m| m.contains("must be in dev-dependencies")));
}

#[test]
fn law18_v2_workspace_alias_cannot_hide_a_production_fixture_dependency() {
    let fixture = Fixture::new();
    fixture.write(CONFIG, ". | sample\n");
    fixture.write("Cargo.toml", "[workspace]\nmembers = ['app', 'world']\n[workspace.dependencies]\nfixture_alias = { package = 'sample-worlds', path = 'world' }\n");
    fixture.write("app/Cargo.toml", "[package]\nname = 'product'\nversion = '0.1.0'\n[dependencies]\nfixture_alias.workspace = true\n");
    fixture.write("app/src/lib.rs", "pub fn app() {}\n");
    fixture.write("world/Cargo.toml", "[package]\nname = 'sample-worlds'\nversion = '0.1.0'\n[package.metadata.worth.worlds]\nworkspace = 'sample'\n");
    fixture.write("world/src/lib.rs", "pub fn authored_world() {}\n");
    fixture.refuses("product dependency `sample-worlds` must be in dev-dependencies");
}

#[test]
fn law18_v2_refuses_product_integration_targets_outside_tests_directory() {
    let fixture = Fixture::new();
    std::fs::remove_file(fixture.0.join(format!("governed/tests/{TARGET}.rs"))).unwrap();
    std::fs::remove_dir(fixture.0.join("governed/tests")).unwrap();
    fixture.write(
        "governed/Cargo.toml",
        "[package]\nname = 'product'\n[[test]]\nname = 'outside'\npath = 'integration.rs'\n",
    );
    fixture.write("governed/integration.rs", "#[test] fn outside() {}\n");
    fixture.refuses("product crate cannot declare integration targets");
}

#[test]
fn law18_v2_refuses_bench_targets_hidden_inside_tests() {
    let fixture = Fixture::new();
    fixture.manifest("autotests = false", &format!("[[test]]\nname = '{TARGET}'\n[[bench]]\nname = 'hidden'\npath = 'tests/bench.rs'\nharness = false\n"));
    fixture.write("governed/tests/bench.rs", "fn main() {}\n");
    fixture.refuses("court contains product code or a production target");
}

#[test]
fn law18_v2_executable_exceptions_cannot_become_unclassed_test_harnesses() {
    let fixture = Fixture::new();
    fixture.write(
        EXCEPTIONS,
        "governed/Cargo.toml | child | public consumer main\n",
    );
    fixture.manifest("", "[[test]]\nname = 'child'\nharness = true\n");
    fixture.write("governed/tests/child.rs", "#[test] fn escaped() {}\n");
    fixture.refuses("must retain harness=false");
    fixture.refuses("`escaped` must be under");
}

#[test]
fn law18_v2_metavariable_macro_tests_must_be_generated_inside_the_class() {
    let fixture = Fixture::new();
    let source = "macro_rules! cases { ($a:meta) => { #[$a] fn escaped() {} }; } cases!(test);";
    fixture.entry(source);
    fixture.refuses("must be under its court class");
    fixture.entry(&format!("mod fast {{ {source} }}"));
    assert!(fixture.messages().is_empty(), "{:?}", fixture.messages());
}

#[test]
fn law18_v2_refuses_disabling_court_or_exception_test_execution() {
    let fixture = Fixture::new();
    fixture.manifest(
        "autotests = false",
        &format!("[[test]]\nname = '{TARGET}'\ntest = false\n"),
    );
    fixture.refuses("cannot disable test execution");
    fixture.manifest("autotests = false", &format!("[[test]]\nname = '{TARGET}'\n[[test]]\nname = 'child'\nharness = false\ntest = false\n"));
    fixture.write(
        EXCEPTIONS,
        "governed/Cargo.toml | child | public consumer main\n",
    );
    fixture.write("governed/tests/child.rs", "fn main() {}\n");
    fixture.refuses("court target `child` cannot disable test execution");
}
