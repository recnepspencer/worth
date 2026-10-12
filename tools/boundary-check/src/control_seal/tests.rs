use super::*;
use std::fs;

#[test]
fn a_test_door_in_an_unlisted_file_is_sealed() {
    let root = std::env::temp_dir().join(format!("unlisted-control-door-{}", std::process::id()));
    let owner = "workspaces/worth-query/crates/worth-query-execution/";
    let file = root.join(owner).join("src/unlisted.rs");
    fs::create_dir_all(file.parent().unwrap()).unwrap();
    fs::write(
        file,
        "pub fn unlisted_fault_for_test() {}\npub(crate) fn unlisted_crate_fault_for_test() {}\npub struct Runtime { pub(crate) fault_slot_for_test: bool }\n",
    )
    .unwrap();
    let (diagnostics, doors) = load(&root);
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    assert!(doors.contains_key("unlisted_crate_fault_for_test"));
    assert!(doors.contains_key("fault_slot_for_test"));
    let diagnostics = references(
        "crates/ordinary/src/lib.rs",
        "fn work() { runtime.unlisted_fault_for_test(); }",
        &doors,
    );
    fs::remove_dir_all(root).unwrap();
    assert_eq!(diagnostics.len(), 1);
    assert!(diagnostics[0].message().contains("unlisted_fault_for_test"));
}

#[test]
fn literal_macro_doors_and_crate_visible_doors_are_discovered() {
    let names = public_items::test_names("macro_rules! doors { () => { pub fn macro_fault_for_test() {} }; } impl Runtime { pub(crate) fn local_fault_for_test() {} }").unwrap();
    assert_eq!(
        names,
        BTreeSet::from(["macro_fault_for_test".into(), "local_fault_for_test".into()])
    );
}

#[test]
fn parameterized_macro_doors_cannot_bypass_discovery() {
    let root =
        std::env::temp_dir().join(format!("parameterized-control-door-{}", std::process::id()));
    let file = root.join("crates/worth-execution/src/lib.rs");
    fs::create_dir_all(file.parent().unwrap()).unwrap();
    fs::write(
        &file,
        "macro_rules! emit { ($name:ident) => { pub fn $name() {} }; } emit!(hidden_for_test);",
    )
    .unwrap();
    let (diagnostics, doors) = load(&root);
    fs::remove_dir_all(root).unwrap();
    assert_eq!(
        diagnostics.len(),
        1,
        "ambiguous declarations fail closed: {diagnostics:?}"
    );
    assert_eq!(diagnostics[0].code(), DiagnosticCode::Bc7012ControlSeal);
    assert!(diagnostics[0].message().contains("hidden_for_test"));
    assert!(!doors.contains_key("hidden_for_test"));
}

#[test]
fn a_macro_reference_cannot_grant_control_ownership() {
    let source = "macro_rules! passthrough { ($($item:tt)*) => { $($item)* }; }
        passthrough! { fn production(runtime: &Runtime) { runtime.operation_control_for_test(); } }";
    assert!(public_items::test_names(source).is_err());
    let doors = BTreeMap::from([(
        "operation_control_for_test".into(),
        BTreeSet::from(["crates/worth-execution/".into()]),
    )]);
    let denied = references(
        "workspaces/worth-query/crates/worth-query-host/src/lib.rs",
        source,
        &doors,
    );
    assert_eq!(denied.len(), 1);
    assert_eq!(denied[0].code(), DiagnosticCode::Bc7012ControlSeal);
}

#[test]
fn thread_local_storage_does_not_grant_ownership_of_its_control_types() {
    let source = "thread_local! { static SLOT: Cell<ForeignForTest> = const { Cell::new(ForeignForTest::new()) }; }
        pub fn local_for_test() {}";
    assert_eq!(
        public_items::test_names(source).unwrap(),
        BTreeSet::from(["local_for_test".into()])
    );
    let doors = BTreeMap::from([
        (
            "ForeignForTest".into(),
            BTreeSet::from(["crates/worth-execution/".into()]),
        ),
        (
            "local_for_test".into(),
            BTreeSet::from(["workspaces/worth-query/crates/worth-query-host/".into()]),
        ),
    ]);
    let denied = references(
        "workspaces/worth-query/crates/worth-query-host/src/lib.rs",
        source,
        &doors,
    );
    assert_eq!(denied.len(), 1);
    assert_eq!(denied[0].code(), DiagnosticCode::Bc7012ControlSeal);
}

#[test]
fn a_neighbor_of_a_test_door_is_still_production_code() {
    let mut doors = doors();
    let owner = "workspaces/worth-query/crates/worth-query-host/";
    doors.insert("helper_for_test".into(), BTreeSet::from([owner.into()]));
    let source = "pub fn helper_for_test() { runtime.owner_operation_control_for_test(); } fn private_wrapper_for_test() { runtime.owner_operation_control_for_test(); } fn production() { runtime.owner_operation_control_for_test(); }";
    assert_eq!(
        references(&format!("{owner}src/lib.rs"), source, &doors).len(),
        1
    );
    let private = "pub fn helper_for_test() {} fn private_wrapper_for_test() { runtime.owner_operation_control_for_test(); }";
    assert_eq!(
        references(&format!("{owner}src/lib.rs"), private, &doors).len(),
        1,
        "a private wrapper is not a sealed public test door"
    );
}
fn doors() -> Doors {
    let source="pub struct SignalOwnerOperationControlForTest; impl Runtime { pub fn owner_operation_control_for_test() {} pub fn inject_fault_for_test() {} }";
    public_items::test_names(source)
        .unwrap()
        .into_iter()
        .map(|name| (name, BTreeSet::from(["crates/worth-signal/".into()])))
        .collect()
}
#[test]
fn production_consumer_cannot_reach_a_control_door() {
    let path = "crates/ordinary/src/lib.rs";
    let diagnostics = references(
        path,
        "fn work() { runtime.owner_operation_control_for_test().inject_fault_for_test(); }",
        &doors(),
    );
    assert_eq!(diagnostics.len(), 2);
    assert!(diagnostics
        .iter()
        .all(|d| d.subject() == path && d.message().contains(path)));
}
#[test]
fn recorded_test_caller_can_reach_the_control_door() {
    let path = entries(CALLERS)
        .find(|(owner, _)| *owner == "crates/worth-signal/")
        .unwrap()
        .1;
    assert!(references(
        path,
        "fn proof() { runtime.owner_operation_control_for_test().inject_fault_for_test(); }",
        &doors()
    )
    .is_empty());
}
#[test]
fn a_new_convention_door_is_sealed_immediately() {
    let names = public_items::test_names(
        "impl Runtime { pub fn a_new_panic_door_for_test() {} } pub const armed_for_test: bool = false;",
    )
    .unwrap();
    assert_eq!(
        names,
        BTreeSet::from(["a_new_panic_door_for_test".into(), "armed_for_test".into()])
    );
    let doors = names
        .into_iter()
        .map(|name| (name, BTreeSet::from(["crates/owner/".into()])))
        .collect();
    assert_eq!(
        references(
            "crates/consumer/src/lib.rs",
            "fn f() { a_new_panic_door_for_test(); let _ = armed_for_test; }",
            &doors
        )
        .len(),
        2
    );
}
#[test]
fn guard_lifecycle_is_reached_only_through_its_sealed_type() {
    let names=public_items::test_names("pub struct GuardForTest; impl GuardForTest { pub fn release(&self) {} } impl Runtime { pub fn pause_for_test() -> GuardForTest { GuardForTest } }").unwrap();
    assert!(names.contains("GuardForTest") && names.contains("pause_for_test"));
    assert!(!names.contains("release"));
}

#[test]
fn repository_conventions_derive_the_reviewed_control_doors() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let (diagnostics, doors) = load(&root);
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    for name in [
        "fail_next_durable_append_for_test",
        "arm_pause_once_for_test",
        "inject_panic_once_for_test",
        "pause_before_product_compare_for_test",
        "pause_product_currentness_for_test",
        "pause_stable_join_for_test",
        "place_managed_computations_on_this_thread_for_test",
        "cancel_and_retry_async_request_for_test",
        "consumed_output_custody_for_test",
    ] {
        assert!(doors.contains_key(name), "unsealed reviewed door {name}");
    }
    let source = "fn proof() { runtime.consumed_output_custody_for_test(); }";
    let refused = references("crates/ordinary/src/lib.rs", source, &doors);
    assert_eq!(refused.len(), 1);
    assert!(refused[0]
        .message()
        .contains("consumed_output_custody_for_test"));
    let callers: Vec<_> = entries(CALLERS)
        .filter(|(_, path)| {
            path.ends_with("custody_stops/balance.rs")
                || path.ends_with("custody_stops/reclamation.rs")
        })
        .collect();
    assert_eq!(
        callers.len(),
        2,
        "both consumed-custody proof callers must be named"
    );
    for (_, caller) in callers {
        assert!(references(caller, source, &doors).is_empty());
    }
}

#[test]
fn real_repository_has_no_unsealed_test_callers() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let (mut diagnostics, doors) = load(&root);
    println!("automatically discovered test doors: {}", doors.len());
    fs::write(
        std::env::temp_dir().join("one-build-discovered-test-doors.json"),
        serde_json::to_vec_pretty(&doors).unwrap(),
    )
    .unwrap();
    let mut shipping = Vec::new();
    for scope in SCOPES {
        shipping.extend(
            shipping_sources::files_under(&root.join(scope))
                .unwrap()
                .into_iter()
                .map(|(path, _)| {
                    path.strip_prefix(&root)
                        .unwrap()
                        .to_string_lossy()
                        .replace('\\', "/")
                }),
        );
    }
    fs::write(
        std::env::temp_dir().join("one-build-shipping-sources.json"),
        serde_json::to_vec_pretty(&shipping).unwrap(),
    )
    .unwrap();
    for file in crate::governed_sources::files_under(&root).unwrap() {
        if file.extension().is_none_or(|ext| ext != "rs") {
            continue;
        }
        let path = file
            .strip_prefix(&root)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        diagnostics.extend(references(
            &path,
            &fs::read_to_string(file).unwrap(),
            &doors,
        ));
    }
    assert!(diagnostics.is_empty(), "{diagnostics:#?}");
}

#[test]
fn new_associated_and_foreign_doors_are_derived_without_a_name_list() {
    let names = public_items::test_names("impl Runtime { pub const new_control_for_test: u8 = 1; } unsafe extern \"C\" { pub fn new_foreign_fault_for_test(); }").unwrap();
    assert!(names.contains("new_control_for_test"));
    assert!(names.contains("new_foreign_fault_for_test"));
}
#[test]
fn literal_macro_constants_are_discovered() {
    let names = public_items::test_names("macro_rules! doors { () => { pub const armed_for_test: bool = false; pub const fn charge_for_test() -> u8 { 0 } }; }").unwrap();
    assert_eq!(
        names,
        BTreeSet::from(["armed_for_test".into(), "charge_for_test".into()])
    );
}

#[test]
fn implicit_public_trait_doors_are_derived() {
    let names = public_items::test_names(
        "pub trait ControlForTest { fn new_fault_for_test(); const armed_for_test: bool; type HandleForTest; }",
    )
    .unwrap();
    assert!(names.contains("new_fault_for_test"));
    assert!(names.contains("armed_for_test"));
    assert!(names.contains("HandleForTest"));
    let names =
        public_items::test_names("#[macro_export] macro_rules! new_fault_for_test { () => {} }")
            .unwrap();
    assert!(names.contains("new_fault_for_test"));
}

#[test]
fn raw_declarations_and_ordinary_references_share_a_seal() {
    let names = public_items::test_names("impl Runtime { pub fn r#inject_fault_for_test() {} } #[r#macro_export] macro_rules! r#new_fault_for_test { () => {} }").unwrap();
    assert!(names.contains("inject_fault_for_test") && names.contains("new_fault_for_test"));
    let doors = names
        .into_iter()
        .map(|name| (name, BTreeSet::from(["crates/owner/".into()])))
        .collect();
    assert_eq!(
        references(
            "crates/consumer/src/lib.rs",
            "fn work() { runtime.inject_fault_for_test(); new_fault_for_test!(); }",
            &doors
        )
        .len(),
        2
    );
}
