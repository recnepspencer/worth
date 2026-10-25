use super::*;

#[test]
fn receipt_only_batches_release_each_bounded_published_snapshot() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime = relational_runtime_with_intent_strategy();
    let left = create_entity(
        request_execution,
        &mut runtime,
        "left",
        BranchId("main".to_string()),
    );
    let right = create_entity(
        request_execution,
        &mut runtime,
        "right",
        BranchId("main".to_string()),
    );
    crate::runtime::fork_branch_from_exact_source(
        request_execution,
        &mut runtime,
        BranchId("batch-receipt-capacity".to_string()),
        &BranchId("main".to_string()),
    )
    .unwrap();
    let baseline = runtime
        .storage_access()
        .storage_stats(request_execution)
        .unwrap()
        .published_snapshot_handle_count;
    assert_eq!(baseline, 0);

    for ordinal in 0..70 {
        let branch = "batch-receipt-capacity";
        let receipt = crate::effect_lifecycle::effect_batch()
            .using_basis(crate::effect_lifecycle::EffectAuthoringBasis::from(
                branch_mutation_basis(),
            ))
            .push(raw_mutation_effect_with_binding(
                runtime_workflow_binding_for_branch(
                    branch_snapshot_identity(&runtime, branch),
                    branch,
                ),
                left,
                native_name_patch(format!("left-{ordinal}")),
            ))
            .push(raw_mutation_effect_with_binding(
                runtime_workflow_binding_for_branch(
                    branch_snapshot_identity(&runtime, branch),
                    branch,
                ),
                right,
                native_name_patch(format!("right-{ordinal}")),
            ))
            .admit()
            .unwrap()
            .lower()
            .unwrap()
            .execute_receipt_with(
                request_execution,
                EffectExecutionAuthority::relational(&mut runtime),
            )
            .unwrap();
        assert!(matches!(
            receipt.target_evidence(),
            EffectReceiptTargetEvidence::BatchMutation {
                component_count: 2,
                ..
            }
        ));
    }
    assert_eq!(
        runtime
            .storage_access()
            .storage_stats(request_execution)
            .unwrap()
            .published_snapshot_handle_count,
        baseline
    );
}
