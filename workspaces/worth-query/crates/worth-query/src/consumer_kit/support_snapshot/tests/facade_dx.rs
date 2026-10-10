use crate::facade;

use super::support_snapshot_workspace;

#[test]
fn public_consumer_kit_facade_projects_exports_loads_and_compares_snapshot() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let workspace = support_snapshot_workspace(request_execution);
    let matrix = workspace.public_support_matrix();

    let snapshot = facade::consumer_kit::project_workspace_support_snapshot(&workspace);
    let terminal_json_document = snapshot
        .to_canonical_terminal_json_document()
        .expect("facade snapshot export should encode");
    let loaded = facade::consumer_kit::load_support_snapshot_terminal_json_document(
        &terminal_json_document.to_external_terminal_json_document(),
        facade::consumer_kit::WorthQuerySupportSnapshotSchemaVersion::current(),
    )
    .expect("facade loader should accept current support snapshot schema");

    loaded
        .assert_equivalent_to_live_matrix(&matrix)
        .expect("facade-loaded snapshot should compare to live support matrix");
}
