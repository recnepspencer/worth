use super::*;

#[test]
fn phase3_foreign_deferred_repair_reports_affinity_and_preserves_source_recovery() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let source = persisted_runtime_with_test_schema();
    create_entity(execution, &source, "phase3-foreign-deferred-source-anchor");
    let foreign = persisted_runtime_with_test_schema();
    create_entity(execution, &foreign, "phase3-foreign-deferred-target-anchor");

    let performed = perform_write(execution, &source, "main", "phase3-foreign-deferred-write");
    let commit_id = performed.canonical_commit().commit.commit_id;
    let source_durable_before = source.durability().durable_log().len();
    let foreign_durable_before = foreign.durability().durable_log().len();
    let source_settlement = source.settlement_port();
    source.durability.arm_append_failure_for_test();
    let deferred_error = source_settlement
        .settle_performed_publication(performed)
        .expect_err("the injected append fault leaves exact deferred repair authority");
    let deferred = deferred_error
        .deferred_settlement()
        .expect("durability deferral carries the performed route");

    match foreign
        .settlement_port()
        .repair_deferred_publication_settlement(deferred)
    {
        Err(DeferredPublicationSettlementError::ForeignRuntime {
            expected_runtime_instance_id,
            actual_runtime_instance_id,
        }) => {
            assert_eq!(expected_runtime_instance_id, foreign.runtime_instance_id());
            assert_eq!(actual_runtime_instance_id, source.runtime_instance_id());
        }
        outcome => panic!("foreign deferred repair must return exact affinity: {outcome:?}"),
    }

    assert_eq!(
        foreign.durability().durable_log().len(),
        foreign_durable_before
    );
    assert!(source_settlement.retains_pending_settlement(commit_id));
    let recovered = source_settlement
        .repair_deferred_publication_settlement(deferred)
        .expect("the source runtime still owns the exact deferred route");
    assert_eq!(recovered.commit_id, commit_id);
    assert_eq!(
        source.durability().durable_log().len(),
        source_durable_before + 1
    );
    assert!(!source_settlement.retains_pending_settlement(commit_id));
}
