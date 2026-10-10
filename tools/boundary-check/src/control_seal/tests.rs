use super::*;
fn doors() -> Doors {
    let source="pub struct SignalOwnerOperationControl; impl Runtime { pub fn owner_operation_control() {} pub fn inject_fault() {} }";
    public_items::names(source)
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
        "fn work() { runtime.owner_operation_control().inject_fault(); }",
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
        "fn proof() { runtime.owner_operation_control().inject_fault(); }",
        &doors()
    )
    .is_empty());
}
#[test]
fn a_new_public_door_in_a_control_file_is_sealed_immediately() {
    let names = public_items::names(
        "impl Runtime { pub fn a_new_panic_door() {} } pub const ARMED: bool = false;",
    )
    .unwrap();
    assert_eq!(
        names,
        BTreeSet::from(["a_new_panic_door".into(), "ARMED".into()])
    );
    let doors = names
        .into_iter()
        .map(|name| (name, BTreeSet::from(["crates/owner/".into()])))
        .collect();
    assert_eq!(
        references(
            "crates/consumer/src/lib.rs",
            "fn f() { a_new_panic_door(); let _ = ARMED; }",
            &doors
        )
        .len(),
        2
    );
}
#[test]
fn guard_lifecycle_is_reached_only_through_its_sealed_type() {
    let names=public_items::names("pub struct Guard; impl Guard { pub fn release(&self) {} } impl Runtime { pub fn pause() -> Guard { Guard } }").unwrap();
    assert!(names.contains("Guard") && names.contains("pause"));
    assert!(!names.contains("release"));
}

#[test]
fn configured_files_derive_the_reviewed_control_doors() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let (diagnostics, doors) = load(&root);
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    for name in [
        "fail_next_durable_append_for_test",
        "arm_pause_once",
        "inject_panic_once",
        "pause_before_product_compare",
        "pause_product_currentness",
        "pause_stable_join_for_test",
        "place_managed_computations_on_this_thread_for_test",
        "cancel_and_retry_async_request",
    ] {
        assert!(doors.contains_key(name), "unsealed reviewed door {name}");
    }
}
