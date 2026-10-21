//! Exact private CAD reach uses the ordinary discovery and source validator.
use super::*;

const OWNER: &str = super::super::private_cad::OWNER;
const CALLER: &str = "crates/worth-cad-entry/tests/program_declaration/installation.rs";
const ARM: &str = "arm_cad_installation_omission_for_test";
const SOURCE: &str = "pub enum CadInstallationOmissionForTest { InitialProvider, PreserveProvider }
pub struct CadInstallationOmissionGuardForTest;
pub struct CadInstallationBindingIdentitiesForTest;
pub fn arm_cad_installation_omission_for_test<S>() -> CadInstallationOmissionGuardForTest { todo!() }
pub fn cad_installation_binding_identities_for_test<S>() -> CadInstallationBindingIdentitiesForTest { todo!() }";
const USE: &str = "use worth_cad_entry::facade::{CadInstallationOmissionForTest, arm_cad_installation_omission_for_test, cad_installation_binding_identities_for_test};
fn case() { let ids = cad_installation_binding_identities_for_test::<Schema>();
let guard = arm_cad_installation_omission_for_test::<Schema>(); }";

fn fixture() -> std::path::PathBuf {
    let root = worth_harness::tooling::fixture_path::unique_path("private-cad-control-reach");
    let file = root.join(OWNER);
    fs::create_dir_all(file.parent().unwrap()).unwrap();
    fs::write(file, SOURCE).unwrap();
    root
}

#[test]
fn actual_cad_door_allows_only_named_owner_hooks_facade_and_test_leaf() {
    let root = fixture();
    for (path, source) in [
        ("crates/worth-cad-entry/src/contribution.rs", "pub(crate) mod installation_omission_for_test;"),
        ("crates/worth-cad-entry/src/families/chamfer/realization/mod.rs", "pub(crate) fn configure() { arm_cad_installation_omission_for_test::<S>(); }"),
        ("crates/worth-cad-entry/src/schema/chamfer_correspondence.rs", "pub(crate) fn declare_chamfer_correspondence() { arm_cad_installation_omission_for_test::<S>(); }"),
        ("crates/worth-cad-entry/src/facade.rs", "pub use crate::contribution::installation_omission_for_test::arm_cad_installation_omission_for_test;"),
        (CALLER, USE),
    ] {
        let file = root.join(path);
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(file, source).unwrap();
    }
    let (diagnostics, doors) = load(&root);
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    assert_eq!(doors[ARM], BTreeSet::from([OWNER.into()]));
    let diagnostics = crate::sealed_sources::validate(&root);
    fs::remove_dir_all(root).unwrap();
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
}

#[test]
fn cad_neighbors_ordinary_consumers_and_helper_doors_remain_denied() {
    let root = fixture();
    let (_, doors) = load(&root);
    fs::remove_dir_all(root).unwrap();
    for path in [
        "crates/worth-cad-entry/tests/program_declaration/neighbor.rs",
        "crates/worth-cad-entry/tests/program_declaration.rs",
        "crates/worth-cad-entry/src/ordinary.rs",
        "crates/worthy-studio-application/src/ordinary.rs",
        "crates/worth-cad-entry/src/contribution/installation_omission_for_test.rs.neighbor.rs",
    ] {
        for source in [
            USE,
            "pub fn helper_for_test() { arm_cad_installation_omission_for_test::<S>(); }",
        ] {
            let diagnostics = references(path, source, &doors);
            assert!(
                diagnostics
                    .iter()
                    .any(|d| d.code() == DiagnosticCode::Bc7012ControlSeal
                        && d.subject() == path
                        && d.message().contains(ARM)),
                "{path}: {diagnostics:?}"
            );
        }
    }
}

#[test]
fn cad_reexport_cannot_claim_control_ownership() {
    let root = fixture();
    let path = "crates/worth-cad-entry/tests/program_declaration/neighbor.rs";
    let file = root.join(path);
    fs::create_dir_all(file.parent().unwrap()).unwrap();
    fs::write(
        file,
        "pub use worth_cad_entry::facade::arm_cad_installation_omission_for_test;",
    )
    .unwrap();
    let (diagnostics, doors) = load(&root);
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    assert_eq!(doors[ARM], BTreeSet::from([OWNER.into()]));
    let diagnostics = crate::sealed_sources::validate(&root);
    fs::remove_dir_all(root).unwrap();
    assert!(
        diagnostics
            .iter()
            .any(|d| d.subject() == path && d.message().contains(ARM)),
        "{diagnostics:?}"
    );
}

#[test]
fn cad_aliases_are_denied_even_in_the_named_facade_or_caller() {
    let root = fixture();
    let (_, doors) = load(&root);
    fs::remove_dir_all(root).unwrap();
    for path in ["crates/worth-cad-entry/src/facade.rs", CALLER] {
        for source in [
            "pub use crate::owner::arm_cad_installation_omission_for_test as ordinary;",
            "type Ordinary = CadInstallationOmissionGuardForTest;",
            "pub const ORDINARY: fn() = arm_cad_installation_omission_for_test;",
            "pub fn ordinary() { arm_cad_installation_omission_for_test::<S>(); }",
            "impl Application { pub fn ordinary() { arm_cad_installation_omission_for_test::<S>(); } }",
        ] {
            let diagnostics = references(path, source, &doors);
            assert!(
                diagnostics
                    .iter()
                    .any(|d| d.message().contains("may not be aliased")),
                "{path}: {diagnostics:?}"
            );
        }
        assert!(references(
            path,
            "pub use crate::owner::arm_cad_installation_omission_for_test;",
            &doors
        )
        .is_empty());
    }
}

#[test]
fn only_actual_registration_hook_items_may_consume_controls_as_ordinary_functions() {
    let root = fixture();
    let (_, doors) = load(&root);
    fs::remove_dir_all(root).unwrap();
    for (path, name) in [
        (
            "crates/worth-cad-entry/src/families/chamfer/realization/mod.rs",
            "configure",
        ),
        (
            "crates/worth-cad-entry/src/schema/chamfer_correspondence.rs",
            "declare_chamfer_correspondence",
        ),
    ] {
        let source =
            format!("pub(crate) fn {name}() {{ arm_cad_installation_omission_for_test::<S>(); }}");
        assert!(references(path, &source, &doors).is_empty());
        let helper =
            "pub(crate) fn ordinary_helper() { arm_cad_installation_omission_for_test::<S>(); }";
        assert!(!references(path, helper, &doors).is_empty());
        for source in [
            "fn ordinary_helper() { arm_cad_installation_omission_for_test::<S>(); }".to_owned(),
            format!(
                "fn outer() {{ fn {name}() {{ arm_cad_installation_omission_for_test::<S>(); }} }}"
            ),
            format!(
                "mod nested {{ fn {name}() {{ arm_cad_installation_omission_for_test::<S>(); }} }}"
            ),
        ] {
            assert!(!references(path, &source, &doors).is_empty(), "{source}");
        }
    }
}

#[test]
fn private_helpers_cannot_forward_an_opaque_control_export() {
    let root = fixture();
    let (_, doors) = load(&root);
    fs::remove_dir_all(root).unwrap();
    let source = "fn hidden<S>() -> CadInstallationOmissionGuardForTest {
        arm_cad_installation_omission_for_test::<S>()
    }
    pub fn ordinary<S>() -> impl Sized { hidden::<S>() }";
    for path in [OWNER, "crates/worth-cad-entry/src/facade.rs"] {
        assert!(
            references(path, source, &doors)
                .iter()
                .any(|diagnostic| diagnostic.code() == DiagnosticCode::Bc7012ControlSeal),
            "private helper escaped at {path}"
        );
    }
    assert!(references(CALLER, USE, &doors).is_empty());
    assert!(references(
        OWNER,
        &format!("{SOURCE}\nfn hidden<S>() {{ arm_cad_installation_omission_for_test::<S>(); }}"),
        &doors
    )
    .is_empty());
    // Owner exports are checked structurally, even when no sealed token appears
    // in the forwarding function's own body or signature.
    assert!(!references(
        OWNER,
        "pub fn ordinary<S>() -> impl Sized { hidden::<S>() }",
        &doors
    )
    .is_empty());
    for source in [
        "pub const ORDINARY: fn() -> CadInstallationOmissionGuardForTest = hidden;",
        "pub struct Ordinary;",
        "pub trait Ordinary { fn acquire() -> impl Sized { hidden() } }",
    ] {
        assert!(!references(OWNER, source, &doors).is_empty(), "{source}");
    }
}

#[test]
fn private_methods_and_nonowner_sealed_receivers_cannot_forward_controls() {
    let root = fixture();
    let (_, doors) = load(&root);
    fs::remove_dir_all(root).unwrap();
    for path in [OWNER, "crates/worth-cad-entry/src/facade.rs"] {
        let source = "impl Application {
            fn hidden<S>() -> CadInstallationOmissionGuardForTest {
                arm_cad_installation_omission_for_test::<S>()
            }
            pub fn ordinary<S>() -> impl Sized { Self::hidden::<S>() }
        }";
        assert!(!references(path, source, &doors).is_empty(), "{path}");
    }
    let source = "impl CadInstallationOmissionGuardForTest {
        fn hidden() { arm_cad_installation_omission_for_test::<S>(); }
    }";
    assert!(!references("crates/worth-cad-entry/src/facade.rs", source, &doors).is_empty());
    assert!(references(OWNER, source, &doors).is_empty());
    assert!(!references(
        "crates/worth-cad-entry/src/facade.rs",
        "trait Hidden { fn acquire() -> impl Sized { arm_cad_installation_omission_for_test::<S>() } }",
        &doors,
    ).is_empty());
}

#[test]
fn a_new_cad_helper_definition_cannot_establish_control_ownership() {
    let root = fixture();
    let path = "crates/worth-cad-entry/src/contribution/neighbor.rs";
    fs::write(
        root.join(path),
        "pub fn new_trampoline_for_test() { arm_cad_installation_omission_for_test::<S>(); }",
    )
    .unwrap();
    let (diagnostics, doors) = load(&root);
    fs::remove_dir_all(root).unwrap();
    assert!(
        diagnostics.iter().any(|d| d.subject() == path),
        "{diagnostics:?}"
    );
    assert!(!doors.contains_key("new_trampoline_for_test"));
    assert_eq!(
        references(
            path,
            "pub fn new_trampoline_for_test() { arm_cad_installation_omission_for_test::<S>(); }",
            &doors
        )
        .len(),
        1
    );
}

#[test]
fn private_cad_mount_is_the_same_seal_and_old_unit_hook_is_not_a_compiled_door() {
    let root = fixture();
    let mount = root.join("crates/worth-cad-entry/src/contribution.rs");
    fs::write(&mount, "pub(crate) mod installation_omission_for_test;").unwrap();
    let (diagnostics, doors) = load(&root);
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    assert_eq!(
        doors["installation_omission_for_test"],
        BTreeSet::from([OWNER.into()])
    );
    fs::write(
        &mount,
        "#[cfg(test)] pub(crate) mod installation_omission_for_test;",
    )
    .unwrap();
    let (diagnostics, doors) = load(&root);
    fs::remove_dir_all(root).unwrap();
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    assert!(!doors.contains_key(ARM));
}
