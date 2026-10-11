mod support;

use worth_relational::facade::runtime::RelationalRuntimeApi;

fn main() {
    let host_request = support::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = RelationalRuntimeApi::builder()
        .runtime_setup(|runtime| {
            runtime.runtime_name("basic-runtime");
        })
        .schema_setup(|schema| {
            schema.schema_registry(support::demo_schema_registry());
        })
        .build();

    let (_left_commit, left) = support::create_entity(request_execution, &runtime, "left");
    let (_right_commit, right) = support::create_entity(request_execution, &runtime, "right");
    let (edge_commit, relation) =
        support::create_relation(request_execution, &runtime, left, right, "connects");

    let current = runtime
        .read_truth()
        .read_snapshot(&edge_commit.snapshot, request_execution)
        .expect("fixture read request remains live")
        .expect("read final snapshot");
    println!(
        "runtime={} entities={} relations={}",
        runtime.config().execution.runtime_name,
        current.entities().len(),
        current.relations().len()
    );
    println!("created entity={left:?} relation={relation:?}");
}
