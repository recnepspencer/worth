use super::*;

pub(super) struct ControllerSequenceProof {
    pub(super) branch: BranchId,
    pub(super) idempotent_commit: crate::facade::transactions::CommitResult,
}

pub(super) fn certify_controller_sequence_shared_truth(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
) -> ControllerSequenceProof {
    let controller_sequence_entity = create_entity(execution, runtime, "controller-sequence");
    let _controller_initial_intent = execute_strategy_commit(
        execution,
        runtime,
        IntentReconciliationInput {
            entity_id: controller_sequence_entity,
            desired_aspect_fields: strategy_name_and_replicas_patch("controller-main", 2),
        }
        .into_native_canonical_request(crate::facade::commit_strategies::StrategyCallerProvenance {
            request_origin: crate::facade::commit_strategies::StrategyRequestOrigin::Test,
            actor_identity: None,
            correlation_id: None,
        })
        .expect("native canonical strategy request"),
        None,
    );
    let controller_sequence_branch =
        create_branch_from_main(runtime, "controller-sequence-feature");
    let _controller_feature_converge = execute_strategy_commit(
        execution,
        runtime,
        ReplicaConvergenceInput {
            entity_id: controller_sequence_entity,
            desired_replicas: 7,
        }
        .into_native_canonical_request(crate::facade::commit_strategies::StrategyCallerProvenance {
            request_origin: crate::facade::commit_strategies::StrategyRequestOrigin::Test,
            actor_identity: None,
            correlation_id: None,
        })
        .expect("native canonical strategy request"),
        Some(controller_sequence_branch.clone()),
    );
    let _controller_narrowed_intent = execute_strategy_commit(
        execution,
        runtime,
        IntentReconciliationInput {
            entity_id: controller_sequence_entity,
            desired_aspect_fields: crate::transactions::data::AspectFieldPatch::from_locator(
                crate::transactions::data::planned_single_field_locator(
                    worth_foundational::facade::AspectKey::new("name")
                        .expect("valid test aspect key"),
                    worth_foundational::facade::FieldKey::new("name")
                        .expect("valid test field key"),
                ),
                worth_foundational::facade::AspectValue::String(
                    worth_foundational::facade::InternedString::Raw(
                        "controller-renamed".to_string(),
                    ),
                ),
            ),
        }
        .into_native_canonical_request(crate::facade::commit_strategies::StrategyCallerProvenance {
            request_origin: crate::facade::commit_strategies::StrategyRequestOrigin::Test,
            actor_identity: None,
            correlation_id: None,
        })
        .expect("native canonical strategy request"),
        None,
    );
    let controller_feature_idempotent_commit = execute_strategy_commit(
        execution,
        runtime,
        ReplicaConvergenceInput {
            entity_id: controller_sequence_entity,
            desired_replicas: 7,
        }
        .into_native_canonical_request(crate::facade::commit_strategies::StrategyCallerProvenance {
            request_origin: crate::facade::commit_strategies::StrategyRequestOrigin::Test,
            actor_identity: None,
            correlation_id: None,
        })
        .expect("native canonical strategy request"),
        Some(controller_sequence_branch.clone()),
    );
    assert_eq!(
        controller_feature_idempotent_commit
            .change_summary()
            .expect("controller idempotent change summary")
            .changed_record_count,
        0
    );
    let controller_sequence_planning = runtime
        .merge()
        .inspect_planning_scope(
            execution,
            MergePlanningRequest::new(
                BranchId("main".to_string()),
                controller_sequence_branch.clone(),
                MergeIntent::ReconcileIntoTarget,
            ),
        )
        .expect("controller sequence merge planning");
    let controller_sequence_classification = controller_sequence_planning
        .conflict_classification
        .classifications
        .iter()
        .find(|classification| {
            classification.record
                == crate::facade::transactions::RecordRef::Entity(controller_sequence_entity)
        })
        .expect("controller sequence classification");
    assert_ne!(
        controller_sequence_classification.class,
        crate::merge::data::MergeConflictClass::StrategyIntentConflict
    );
    assert_eq!(
        controller_sequence_classification.class,
        crate::merge::data::MergeConflictClass::DivergentVisibleState,
        "branch-local name and replica changes should remain explicit visible divergence"
    );
    ControllerSequenceProof {
        branch: controller_sequence_branch,
        idempotent_commit: controller_feature_idempotent_commit,
    }
}
