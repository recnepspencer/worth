use super::*;

#[test]
fn replay_contract_preserves_aspect_bearing_patch_and_history_surfaces() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations);
    let created = create_entity_outcome(execution, &runtime, "before");
    let entity = changed_entities(&created)[0];
    let updated = update_entity(execution, &runtime, entity, "after");
    let source = create_entity(execution, &runtime, "source");
    let target = create_entity(execution, &runtime, "target");
    let relation_outcome = create_relation_outcome(execution, &runtime, source, target, "r1");
    let relation = changed_relations(&relation_outcome)[0];
    let expected_entity_history = runtime
        .history()
        .entity_aspect_history(execution, &BranchId("main".to_string()), entity, None)
        .unwrap();
    let expected_relation_history = runtime
        .history()
        .relation_aspect_history(execution, &BranchId("main".to_string()), relation, None)
        .unwrap();
    let expected_entity_digest = runtime
        .history()
        .entity_aspect_history_with_trace(execution, &BranchId("main".to_string()), entity, None)
        .unwrap()
        .aspect_history_digest();
    let expected_relation_digest = runtime
        .history()
        .relation_aspect_history_with_trace(
            execution,
            &BranchId("main".to_string()),
            relation,
            None,
        )
        .unwrap()
        .aspect_history_digest();

    let replay = runtime
        .replay_authority()
        .replay_commit(
            RelationalReplayRequest {
                commit_id: relation_outcome.commit.commit_id,
                branch_id: BranchId("main".to_string()),
                execution_mode: ReplayExecutionMode::SerialDeterministic,
                verification_mode: ReplayVerificationMode::NormalRecoveryVerification,
            },
            execution,
        )
        .unwrap();

    assert!(runtime.replay().compare_outcome(&replay));
    assert!(replay
        .compared_surfaces
        .contains(&ReplayObservableSurface::Patch));
    assert!(replay
        .compared_surfaces
        .contains(&ReplayObservableSurface::Diagnostics));
    assert!(replay
        .compared_surfaces
        .contains(&ReplayObservableSurface::History));
    assert_eq!(expected_entity_history.len(), 2);
    assert_eq!(expected_relation_history.len(), 1);
    assert_eq!(expected_entity_digest.entry_count, 2);
    assert_eq!(expected_relation_digest.entry_count, 1);
    let _ = assert_patch_truth_invariants(&updated);
    let _ = assert_patch_truth_invariants(&relation_outcome);
}
