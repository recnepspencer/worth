use crate::tests::support::*;

#[test]
fn durable_log_compaction_respects_checkpoint_policy() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(test_schema_registry())
        .durable_log_policy(DurableLogPolicy {
            retention_mode: DurableLogRetentionMode::CompactAfterCheckpoint,
            max_in_memory_envelopes: 1,
            compact_after_checkpoint: true,
        })
        .build();

    create_entity(execution, &runtime, "first");
    runtime
        .durability_authority()
        .checkpoint(execution)
        .unwrap();
    create_entity(execution, &runtime, "second");
    create_entity(execution, &runtime, "third");

    assert!(runtime.durability().durable_log().len() <= 1);
}
