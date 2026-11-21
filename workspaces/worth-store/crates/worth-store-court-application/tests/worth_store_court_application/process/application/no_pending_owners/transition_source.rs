//! A reopened pending transition readmits its exact durable prefix on its
//! original planning root, read from the selected admission image.
use super::*;
pub(super) mod interrupted;

#[test]
fn transition_source_recovers_the_exact_durable_prefix_and_original_planning_root() {
    let directory = tempfile::tempdir().unwrap();
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    interrupted::home(directory.path(), &execution);
    let fixture = super::transition_open_original::reopen(directory.path(), &execution);
    let resumed = interrupted::resume(&fixture, &execution);
    let head = fixture
        .home
        .world()
        .observation_port()
        .observe_root_product_branch()
        .unwrap();
    assert_eq!(resumed.catalogs.durable_native_steps(), 1);
    assert_eq!(resumed.catalogs.predecessor(), resumed.root.basis());
    assert_eq!(resumed.catalogs.current(), head.basis().relational_basis());
    assert_ne!(
        resumed.root.basis().truth_version(),
        resumed.catalogs.current().truth_version(),
        "the historical planning root is not the current execution basis"
    );
    assert_eq!(
        fixture
            .home
            .relational()
            .runtime()
            .config()
            .schema
            .registry
            .authoritative_schema_basis()
            .unwrap()
            .unwrap()
            .1,
        crate::schema_transition::TARGET,
        "the durable target step's successor schema is readmitted"
    );
    drop((resumed, head));
    close(fixture.home, &execution);
}

#[test]
fn transition_source_reads_its_planning_predecessor_from_the_selected_admission_image() {
    let directory = tempfile::tempdir().unwrap();
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    // The predecessor is the close checkpoint's head; only the step is tail.
    let original = interrupted::home(directory.path(), &execution);
    assert_eq!(original.predecessor_rows, 1);
    let fixture = super::transition_open_original::reopen(directory.path(), &execution);
    let resumed = interrupted::resume(&fixture, &execution);
    let native = fixture.home.relational().runtime();
    assert_eq!(
        interrupted::rows(native, resumed.root.basis(), &execution),
        original.predecessor_rows
    );
    assert_eq!(
        interrupted::rows(native, resumed.catalogs.current(), &execution),
        original.predecessor_rows + 1
    );
    drop(resumed);
    close(fixture.home, &execution);
}
