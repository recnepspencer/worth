mod support;

use worth_relational::facade::{
    durability::RecoveryVerificationMode, runtime::RelationalRuntimeApi,
};

fn main() {
    let host_request = support::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(support::demo_schema_registry())
        .build();

    let (_created, entity_id) = support::create_entity(execution, &runtime, "durable");
    let _updated = support::update_entity(execution, &runtime, entity_id, "durable-updated");
    let _deleted = support::delete_entity(execution, &runtime, entity_id);

    let validation = runtime
        .validation()
        .certification_state(execution)
        .expect("invariant fixture request");
    let recovery_plan = runtime
        .durability()
        .recovery_plan(
            execution,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .expect("original request admits native recovery input");

    println!(
        "validation_results={} blocking_violation={} durable_log_len={} recovery_segments={}",
        validation.summary().result_count(),
        validation.summary().has_blocking_violation(),
        runtime.durability().durable_commit_count(),
        recovery_plan.cursor.segment_ids.len()
    );
}
