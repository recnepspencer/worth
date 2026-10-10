use super::*;

pub(super) struct PrimaryStrategyConflictProof {
    pub(super) main_commit: crate::facade::transactions::CommitResult,
    pub(super) feature_commit: crate::facade::transactions::CommitResult,
}

pub(super) fn certify_primary_strategy_conflict(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    entity: crate::facade::identity::EntityId,
    feature_branch: &BranchId,
) -> PrimaryStrategyConflictProof {
    let main_commit = execute_strategy_commit(
        execution,
        runtime,
        IntentReconciliationInput {
            entity_id: entity,
            desired_aspect_fields: strategy_name_and_replicas_patch("service-main", 1),
        }
        .into_native_canonical_request(crate::facade::commit_strategies::StrategyCallerProvenance {
            request_origin: crate::facade::commit_strategies::StrategyRequestOrigin::Test,
            actor_identity: None,
            correlation_id: None,
        })
        .expect("native canonical strategy request"),
        None,
    );
    let feature_commit = execute_strategy_commit(
        execution,
        runtime,
        ReplicaConvergenceInput {
            entity_id: entity,
            desired_replicas: 7,
        }
        .into_native_canonical_request(crate::facade::commit_strategies::StrategyCallerProvenance {
            request_origin: crate::facade::commit_strategies::StrategyRequestOrigin::Test,
            actor_identity: None,
            correlation_id: None,
        })
        .expect("native canonical strategy request"),
        Some(feature_branch.clone()),
    );
    let planning = runtime
        .merge()
        .inspect_planning_scope(
            execution,
            MergePlanningRequest::new(
                BranchId("main".to_string()),
                feature_branch.clone(),
                MergeIntent::ReconcileIntoTarget,
            ),
        )
        .expect("merge planning");
    let classification = planning
        .conflict_classification
        .classifications
        .iter()
        .find(|classification| {
            classification.record == crate::facade::transactions::RecordRef::Entity(entity)
        })
        .expect("strategy conflict classification");
    assert_eq!(
        classification.class,
        crate::merge::data::MergeConflictClass::StrategyIntentConflict
    );
    let policy_record = planning
        .policy_resolution
        .records
        .iter()
        .find(|record| record.record == crate::facade::transactions::RecordRef::Entity(entity))
        .expect("strategy policy record");
    assert_eq!(
        policy_record.proof_boundary.decision_boundary,
        crate::merge::data::MergePolicyDecisionBoundary::RequiresManualResolution {
            class: crate::merge::data::MergeManualResolutionClass::StrategyIntentConflict,
        }
    );
    PrimaryStrategyConflictProof {
        main_commit,
        feature_commit,
    }
}
