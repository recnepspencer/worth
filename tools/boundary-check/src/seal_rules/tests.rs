use super::*;

#[test]
fn native_home_composition_capabilities_have_distinct_caller_classes() {
    let attachment =
        "workspaces/worth-store/crates/worth-store-application/src/world_section/attachment.rs";
    let readmission =
        "workspaces/worth-store/crates/worth-store-application/src/world_section/readmission.rs";
    for (factory, caller, other) in [
        ("world_composition_runtime", attachment, readmission),
        ("world_recovery_authority", readmission, attachment),
    ] {
        let reference = format!("{factory}();");
        let allowed = format!(
            "native-home-composition {attachment}\nnative-world-readmission {readmission}\n"
        );
        let mut input = Inputs::fixture(&[(caller, &reference)], "");
        input.files.insert(
            format!("{CONFIG}certification-construction-callers.txt"),
            allowed.clone(),
        );
        assert!(check(&input).is_empty(), "{factory}: intended exact caller");
        let mut foreign = Inputs::fixture(&[(other, &reference)], "");
        foreign.files.insert(
            format!("{CONFIG}certification-construction-callers.txt"),
            allowed,
        );
        assert_eq!(
            check(&foreign).len(),
            1,
            "{factory}: the other capability's caller is insufficient"
        );
    }
}
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
    for (factory, class) in [
        ("certification_query", "installation"),
        ("readmit_world_home", "world-home"),
        ("world_composition_runtime", "native-home-composition"),
        ("world_recovery_authority", "native-world-readmission"),
        (
            "discover_recovered_associated_effects",
            "native-recovered-inventory",
        ),
        ("query_runtime_handle", "query-registered-native"),
        ("prepare_home_program", "query-home-preparation"),
        ("install_home_native_schema", "query-home-preparation"),
        (
            "finish_fresh_home_native_readmission",
            "query-home-preparation",
        ),
        ("attach_home_registered_source", "query-home-preparation"),
        ("attach_home_world", "query-home-preparation"),
        (
            "native_for_schema_installation",
            "native-home-query-preparation",
        ),
        ("query_finalization_parts", "native-home-query-preparation"),
        ("initialize_new_home", "native-home-query-preparation"),
        ("query_composition_parts", "native-home-query-preparation"),
        ("attach_unrecovered_home", "native-home-attachment"),
        (
            "home_schema_installation_runtime",
            "native-home-schema-installation",
        ),
        (
            "home_query_finalization_parts",
            "native-home-query-finalization",
        ),
        (
            "query_publication_home_binding",
            "query-world-commit-preparation",
        ),
        (
            "prepare_query_publication_without_signal",
            "query-world-commit-preparation",
        ),
        (
            "bind_query_native_proposal",
            "query-world-commit-preparation",
        ),
        (
            "complete_query_native_preparation",
            "query-world-commit-preparation",
        ),
    ] {
        let reference = format!("{factory}();");
        let mut input = Inputs::fixture(&[("app/src/lib.rs", &reference)], "");
        assert_eq!(check(&input).len(), 1);
        for denied in [
            "other app/src/lib.rs".to_owned(),
            format!("{class} app/src/sibling.rs"),
        ] {
            input.files.insert(
                format!("{CONFIG}certification-construction-callers.txt"),
                denied,
            );
            assert_eq!(check(&input).len(), 1);
        }
        input.files.insert(
            format!("{CONFIG}certification-construction-callers.txt"),
            format!("{class} app/src/lib.rs"),
        );
        assert!(check(&input).is_empty());
    }
}
