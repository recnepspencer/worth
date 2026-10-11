mod support;

use worth_relational::facade::runtime::RelationalRuntimeApi;

fn main() {
    let host_request = support::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(support::demo_schema_registry())
        .build();

    let (created, entity_id) = support::create_entity(request_execution, &runtime, "first");
    let runtime_instance_id = runtime.main_branch_identity().runtime_instance_id();
    let retained = runtime
        .snapshots()
        .retained_snapshot_for_commit(runtime_instance_id, &created.commit)
        .expect("exact retained commit snapshot");
    let snapshot = retained.snapshot_handle();
    let _updated = support::update_entity(request_execution, &runtime, entity_id, "first-updated");

    let read_path = runtime
        .read_truth()
        .inspect_snapshot_read_path(snapshot)
        .expect("snapshot read path");
    let snap_read = runtime
        .read_truth()
        .read_snapshot(snapshot, request_execution)
        .expect("fixture read request remains live")
        .expect("snapshot read");

    let authoritative_aspect_state = snap_read
        .get_entity(entity_id)
        .expect("snapshot entity")
        .authoritative_aspect_state
        .clone();
    println!("snapshot diagnostics entries={}", read_path.entries.len());
    println!("snapshot preserved authoritative aspect state={authoritative_aspect_state:?}");
}
