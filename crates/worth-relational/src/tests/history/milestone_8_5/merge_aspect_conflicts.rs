use super::*;

pub(super) fn certify_overlapping_aspect_strategy_conflict(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    aspect_overlap_entity: crate::facade::identity::EntityId,
    aspect_overlap_branch: &BranchId,
) {
    let _aspect_overlap_main_commit = execute_strategy_commit(
        execution,
        runtime,
        AspectFieldReconciliationInput {
            entity_id: aspect_overlap_entity,
            field_locator: strategy_field_locator(
                crate::tests::support::aspect_key("name"),
                crate::tests::support::field_key("name"),
            ),
            desired_value: worth_foundational::facade::AspectValue::String("aspect-main".into()),
        }
        .into_native_canonical_request(crate::facade::commit_strategies::StrategyCallerProvenance {
            request_origin: crate::facade::commit_strategies::StrategyRequestOrigin::Test,
            actor_identity: None,
            correlation_id: None,
        })
        .expect("native canonical strategy request"),
        None,
    );
    let _aspect_overlap_feature_commit = execute_strategy_commit(
        execution,
        runtime,
        AspectFieldReconciliationInput {
            entity_id: aspect_overlap_entity,
            field_locator: strategy_field_locator(
                crate::tests::support::aspect_key("name"),
                crate::tests::support::field_key("name"),
            ),
            desired_value: worth_foundational::facade::AspectValue::String("aspect-feature".into()),
        }
        .into_native_canonical_request(crate::facade::commit_strategies::StrategyCallerProvenance {
            request_origin: crate::facade::commit_strategies::StrategyRequestOrigin::Test,
            actor_identity: None,
            correlation_id: None,
        })
        .expect("native canonical strategy request"),
        Some(aspect_overlap_branch.clone()),
    );
    let aspect_overlap_planning = runtime
        .merge()
        .inspect_planning_scope(
            execution,
            MergePlanningRequest::new(
                BranchId("main".to_string()),
                aspect_overlap_branch.clone(),
                MergeIntent::ReconcileIntoTarget,
            ),
        )
        .expect("aspect overlap merge planning");
    let aspect_overlap_classification = aspect_overlap_planning
        .conflict_classification
        .classifications
        .iter()
        .find(|classification| {
            classification.record
                == crate::facade::transactions::RecordRef::Entity(aspect_overlap_entity)
        })
        .expect("aspect overlap classification");
    assert_eq!(
        aspect_overlap_classification.class,
        crate::merge::data::MergeConflictClass::StrategyIntentConflict
    );
}

pub(super) fn certify_disjoint_aspect_strategy_truth(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    aspect_disjoint_entity: crate::facade::identity::EntityId,
    aspect_disjoint_branch: &BranchId,
) {
    let _aspect_disjoint_main_commit = execute_strategy_commit(
        execution,
        runtime,
        AspectFieldReconciliationInput {
            entity_id: aspect_disjoint_entity,
            field_locator: strategy_field_locator(
                crate::tests::support::aspect_key("name"),
                crate::tests::support::field_key("name"),
            ),
            desired_value: worth_foundational::facade::AspectValue::String("disjoint-main".into()),
        }
        .into_native_canonical_request(crate::facade::commit_strategies::StrategyCallerProvenance {
            request_origin: crate::facade::commit_strategies::StrategyRequestOrigin::Test,
            actor_identity: None,
            correlation_id: None,
        })
        .expect("native canonical strategy request"),
        None,
    );
    let _aspect_disjoint_feature_commit = execute_strategy_commit(
        execution,
        runtime,
        ReplicaConvergenceInput {
            entity_id: aspect_disjoint_entity,
            desired_replicas: 9,
        }
        .into_native_canonical_request(crate::facade::commit_strategies::StrategyCallerProvenance {
            request_origin: crate::facade::commit_strategies::StrategyRequestOrigin::Test,
            actor_identity: None,
            correlation_id: None,
        })
        .expect("native canonical strategy request"),
        Some(aspect_disjoint_branch.clone()),
    );
    let aspect_disjoint_planning = runtime
        .merge()
        .inspect_planning_scope(
            execution,
            MergePlanningRequest::new(
                BranchId("main".to_string()),
                aspect_disjoint_branch.clone(),
                MergeIntent::ReconcileIntoTarget,
            ),
        )
        .expect("aspect disjoint merge planning");
    let aspect_disjoint_classification = aspect_disjoint_planning
        .conflict_classification
        .classifications
        .iter()
        .find(|classification| {
            classification.record
                == crate::facade::transactions::RecordRef::Entity(aspect_disjoint_entity)
        })
        .expect("aspect disjoint classification");
    assert_ne!(
        aspect_disjoint_classification.class,
        crate::merge::data::MergeConflictClass::StrategyIntentConflict
    );
    assert!(
        aspect_disjoint_classification.strategy_evidence.is_none(),
        "disjoint aspect-vs-replica intent should not synthesize strategy conflict evidence: {aspect_disjoint_classification:?}"
    );
}
