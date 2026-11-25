use super::validation_engine_fixtures::*;
use worth_relational_contracts::internal_port::config::data::runtime_config::RelationalRuntimeConfig;
use worth_relational_contracts::internal_port::identity::data::primitives::VersionId;
use worth_relational_contracts::internal_port::runtime::operation_control::interruption_event::RelationalInterruptionBoundary as Boundary;
use worth_relational_contracts::internal_port::runtime::operation_control::RelationalOperationControl;
use worth_relational_branch::internal_port::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation;
use worth_relational_contracts::internal_port::transactions::data::intents::mutation_intent::BulkEntityCreateIntent;
use worth_relational_contracts::internal_port::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use crate::validation::engine::test_view::invariant_view;
use crate::validation::engine::InvariantPreparationControl;
use std::collections::BTreeMap;
use worth_execution::ExecutionAllocationPolicy as Allocation;

#[test]
fn complete_relation_scope_retains_more_than_former_touched_entity_limit() {
    let mut config = RelationalRuntimeConfig::default();
    config.schema.registry = cardinality_minimum_registry();
    let instrumentation = RuntimeInstrumentation::default();
    let view = invariant_view(&instrumentation, config, VersionId(0));
    let partitions = BTreeMap::new();
    let plan = entity_plan(32_769);
    let request = InvariantExecutionRequest::from_profile_with_contract(
        InvariantRequestProfile::CertificationBoundary,
        &view,
        InvariantObservation::committed_branch(&partitions),
        VersionId(0),
        Some(&plan),
        Some(worth_relational_contracts::internal_port::validation::data::contracts::InvariantPlanContract::from_merged_plan(&plan)),
    );
    let scope = request
        .relation_integrity_scopes()
        .unwrap()
        .scope_for(KindId(2))
        .unwrap();
    assert_eq!(scope.created_candidate_entities.len(), 32_769);
    for name in ["node-0", "node-32768"] {
        assert!(scope
            .created_candidate_entities
            .iter()
            .any(|entity| entity.client_key.as_raw_str() == Some(name)));
    }
    // Full candidate retention must still reach the real minimum-cardinality
    // rule, rather than a preparation count refusal or a silently empty scope.
    let result = InvariantEngine::from_view(&view).execute_fixture(request);
    assert!(result.results().iter().any(|result| matches!(
        &result.verdict,
        worth_relational_contracts::internal_port::validation::data::execution::InvariantVerdict::Violation(violation)
            if matches!(&violation.fields, InvariantViolationFields::RelationCardinalityEndpoint {
                contract_id, entity_id: worth_relational_contracts::internal_port::transactions::data::primitives::EntityReference::Created(created),
                count: 0, limit: 1, ..
            } if contract_id.as_str() == "min_one" && created.client_key.as_raw_str() == Some("node-32768"))
    )));
}

#[test]
fn relation_scope_preparation_preserves_supplied_allocation_stop() {
    let mut config = RelationalRuntimeConfig::default();
    config.schema.registry = cardinality_minimum_registry();
    let instrumentation = RuntimeInstrumentation::default();
    let view = invariant_view(&instrumentation, config, VersionId(0));
    let partitions = BTreeMap::new();
    let cancellation = worth_execution::CancellationSource::new();
    let lease = worth_harness::facade::execution_fixture::test_execution_authority()
        .request_lease(worth_execution::LeaseRequest {
            policy: worth_foundational::ExecutionRequestPolicy::new(
                worth_foundational::ExecutionPosture::Automatic,
                worth_foundational::DeterminismContract::CanonicalBitwise,
                worth_foundational::ExecutionBudget::new(
                    std::num::NonZeroUsize::new(1).unwrap(),
                    Some(0),
                    Some(1),
                ),
            ),
            deadline: None,
            cancellation: cancellation.token(),
        })
        .unwrap();
    cancellation.cancel();
    let control = InvariantPreparationControl::new(
        RelationalOperationControl::uninterrupted(),
        Allocation::Execution(&lease),
        Boundary::ProposalValidation,
    );
    let plan = entity_plan(1);
    let error = match InvariantExecutionRequest::from_profile_with_contract_at_current_version(
        &worth_execution::ExecutionRequest::leased(&lease),
        InvariantRequestProfile::CertificationBoundary,
        &view,
        InvariantObservation::committed_branch(&partitions),
        VersionId(0),
        VersionId(0),
        Some(&plan),
        Some(worth_relational_contracts::internal_port::validation::data::contracts::InvariantPlanContract::from_merged_plan(&plan)),
        &control,
    ) {
        Err(error) => error,
        Ok(_) => panic!("cancelled supplied lease must remain stopped"),
    };
    match error {
        TransactionCommitError::Conflict { error, .. } => {
            let denial = error.allocation_denial().expect("full allocation cause");
            assert_eq!(
                denial.classify(),
                worth_execution::ExecutionAllocationDenialClass::Request(
                    worth_execution::WorkCeilingDenial::Stopped(
                        worth_execution::MapKernelStop::Cancelled
                    )
                )
            );
            assert_eq!(denial.requested_payload_bytes(), None);
        }
        other => panic!("expected supplied allocation stop, got {other:?}"),
    }
}

fn entity_plan(count: usize) -> MergedCommitPlan {
    MergedCommitPlan {
        transaction_id: TransactionId(20),
        merged_intents: vec![entity_intent(count)],
    }
}

fn entity_intent(count: usize) -> MutationIntent {
    MutationIntent::Create(CreateIntent::BulkEntities(BulkEntityCreateIntent {
        partition_id: PartitionId::main(),
        kind_id: KindId(1),
        client_keys: (0..count)
            .map(|index| ClientKey::raw(format!("node-{index}")))
            .collect(),
        field_patches: vec![
            worth_relational_contracts::internal_port::transactions::data::aspect_field_patch::AspectFieldPatch::default(
            );
            count
        ],
    }))
}
