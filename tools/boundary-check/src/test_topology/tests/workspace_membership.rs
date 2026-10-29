//! Cargo membership and path selection remain enforced after v3's cutover.
use super::fixture::{COURT, TARGET};
use super::{
    super::{CONFIG, EXCEPTIONS},
    fixture::Fixture,
};

#[test]
fn law18_refuses_a_second_implicit_target_and_directory_target() {
    let f = Fixture::new();
    f.manifest("autotests=true", "");
    f.write("governed/tests/extra/main.rs", "#[test] fn extra() {}");
    f.refuses("exactly one integration-test target");
}
#[test]
fn law18_workspace_membership_does_not_annex_other_workspaces_or_fixtures() {
    let f = Fixture::new();
    let manifest = std::fs::read_to_string(f.0.join("governed/Cargo.toml")).unwrap();
    f.write(
        "governed/Cargo.toml",
        manifest.strip_prefix("[workspace]\n").unwrap(),
    );
    f.write(CONFIG, ". | sample\n");
    f.write(
        "Cargo.toml",
        "[workspace]\nmembers=['governed/member*']\nexclude=['governed/member-excluded']\n",
    );
    for name in ["member", "member-excluded"] {
        f.write(
            &format!("governed/{name}/Cargo.toml"),
            "[package]\nname='product'\nversion='0.1.0'\n",
        );
        f.write(&format!("governed/{name}/src/lib.rs"), "");
        f.write(
            &format!("governed/{name}/tests/forbidden.rs"),
            "#[test] fn forbidden() {}",
        );
    }
    f.write("other/Cargo.toml", "[package]\nname='other'\n[workspace]\n");
    f.write("other/tests/untouched.rs", "#[test] fn untouched() {}");
    let messages = f.messages();
    assert_eq!(messages.len(), 1, "{messages:?}");
    assert!(messages[0].contains("product crate"), "{messages:?}");
}
#[test]
fn law18_implicit_path_dependency_members_cannot_hide_a_second_target() {
    let f = Fixture::new();
    f.write(CONFIG, ". | sample\n");
    f.write("Cargo.toml", "[workspace]\nmembers=['owner']\n");
    f.write("owner/Cargo.toml",&format!("[package]\nname='owner'\nversion='0.1.0'\n[dependencies]\nimplicit={{package='{COURT}',path='../implicit'}}\n"));
    f.write("owner/src/lib.rs", "");
    f.write("implicit/Cargo.toml",&format!("[package]\nname='{COURT}'\nversion='0.1.0'\nautotests=false\n[lib]\npath='outside.rs'\n[package.metadata.worth.court]\nsurface='source'\n[[test]]\nname='{TARGET}'\npath='tests/{TARGET}.rs'\n[[test]]\nname='extra'\npath='tests/extra/main.rs'\n"));
    f.write("implicit/outside.rs", "");
    f.write(
        &format!("implicit/tests/{TARGET}.rs"),
        "#[test] fn canonical() {}",
    );
    f.write("implicit/tests/extra/main.rs", "#[test] fn extra() {}");
    f.refuses("exactly one integration-test target");
}
#[test]
fn law18_nested_fixture_workspace_is_governed_but_other_workspaces_are_not() {
    let f = Fixture::new();
    f.write(
        CONFIG,
        "governed | sample\ngoverned/tests/fixtures/court | nested\n",
    );
    f.write("governed/tests/fixtures/court/Cargo.toml","[package]\nname='nested-court-source'\nversion='0.1.0'\nautotests=false\n[workspace]\n[package.metadata.worth.court]\nsurface='source'\n[[test]]\nname='nested_court_source'\npath='tests/nested_court_source.rs'\n");
    f.write(
        "governed/tests/fixtures/court/tests/nested_court_source.rs",
        "mod fast { #[test] fn retired_category() {} }",
    );
    f.write("other/Cargo.toml", "[package]\nname='other'\n[workspace]\n");
    f.write("other/tests/outside.rs", "#[test] fn outside() {}");
    let messages = f.messages();
    assert_eq!(messages.len(), 1, "{messages:?}");
    assert!(messages[0].contains("invalid category path"));
}
#[test]
fn conditional_test_attributes_and_raw_paths_cannot_hide_identifiers() {
    let f = Fixture::new();
    f.entry("#[cfg_attr(test,test)] fn phase_nine_conditional() {} mod r#process { #[path=\"../leaf.rs\"] mod family; }");
    f.write("governed/tests/process/.keep", "");
    f.write("governed/tests/leaf.rs", "#[test] fn lawful() {}");
    f.refuses("phase_nine_conditional");
    f.entry("#[cfg_attr(test,test)] fn conditional() {} mod r#process { #[path=\"../leaf.rs\"] mod family; }");
    assert!(f.messages().is_empty(), "{:?}", f.messages());
}
#[test]
fn dev_dependencies_may_use_testkits_without_production_authority() {
    let f = Fixture::new();
    f.write("governed/Cargo.toml","[package]\nname='product'\n[dev-dependencies]\nfixture={package='sample-testkit-host',path='../fixture'}\n");
    assert!(!f
        .messages()
        .iter()
        .any(|m| m.contains("must be in dev-dependencies")));
    f.write(EXCEPTIONS, "# no executable exceptions\n");
}
