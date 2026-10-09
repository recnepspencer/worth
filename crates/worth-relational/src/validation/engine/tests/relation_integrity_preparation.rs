use super::validation_engine_fixtures::*;
use crate::mvcc::{
    RelationalInterruptionBoundary as Boundary, RelationalOperationControl,
    RelationalOperationInterruption as Interruption, RelationalTransactionIntent,
};
use crate::transactions::data::{BulkEntityCreateIntent, TransactionCommitError};
use crate::validation::engine::{InvariantPreparationControl, InvariantRuntimeView};
use worth_execution::ExecutionAllocationPolicy as Allocation;

#[test]
fn complete_relation_scope_retains_more_than_former_touched_entity_limit() {
    let runtime = runtime_with_cardinality_minimum();
    let plan = entity_plan(32_769);
    let request = InvariantExecutionRequest::from_profile_with_contract(
        InvariantRequestProfile::CertificationBoundary,
        &runtime,
        InvariantObservation::committed(runtime.storage_access().current_edition()),
        runtime.current_version_id(),
        Some(&plan),
        Some(crate::validation::data::InvariantPlanContract::from_merged_plan(&plan)),
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
    let result = InvariantEngine::new(&runtime).execute(request);
    assert!(result.results().iter().any(|result| matches!(
        &result.verdict,
        crate::validation::data::InvariantVerdict::Violation(violation)
            if matches!(&violation.fields, InvariantViolationFields::RelationCardinalityEndpoint {
                contract_id, entity_id: crate::transactions::data::EntityReference::Created(created),
                count: 0, limit: 1, ..
            } if contract_id.as_str() == "min_one" && created.client_key.as_raw_str() == Some("node-32768"))
    )));
}

#[test]
fn relation_scope_preparation_interruption_preserves_head_and_allows_retry() {
    for interruption in [Interruption::Cancelled, Interruption::TimedOut] {
        let runtime = runtime_with_partition_isolation();
        let identity = runtime.main_branch_identity();
        let (_, basis) = runtime.observe_branch(&identity).unwrap();
        let before = runtime
            .branch_reference_state(&crate::history::data::BranchId("main".into()))
            .unwrap();
        // The original validation boundary polls only three times. Visit eight
        // requires the scope collector's actual per-item safe points.
        let control = RelationalOperationControl::uninterrupted().with_injected_interruption(
            Boundary::ProposalValidation,
            interruption,
            8,
        );
        let mut transaction = runtime
            .begin_branch_transaction_with_control(
                &basis,
                RelationalTransactionIntent::ordinary(),
                control,
            )
            .unwrap();
        transaction
            .push_batch(entity_batch(3), Allocation::SystemAllocation)
            .unwrap();
        let error = match transaction.validate(&runtime, Allocation::SystemAllocation) {
            Err(error) => error,
            Ok(_) => panic!("scope scan must observe the requested stop"),
        };
        match error {
            TransactionCommitError::Interrupted {
                interruption: event,
                ..
            } => {
                assert_eq!(event.interruption(), interruption);
                assert_eq!(event.boundary(), Boundary::ProposalValidation);
            }
            other => panic!("expected exact native interruption, got {other:?}"),
        }
        assert_eq!(
            runtime
                .branch_reference_state(&crate::history::data::BranchId("main".into()))
                .unwrap(),
            before
        );
        let mut retry = runtime
            .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
            .unwrap();
        retry
            .push_batch(entity_batch(3), Allocation::SystemAllocation)
            .unwrap();
        let outcome = retry
            .commit(&runtime, Allocation::SystemAllocation)
            .unwrap();
        for index in 0..3 {
            assert!(outcome
                .created_entity(&crate::transactions::data::CreatedEntityRef {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(1),
                    client_key: ClientKey::raw(format!("node-{index}")),
                })
                .is_some());
        }
        crate::tests::support::release_test_commit_snapshot(&runtime, &outcome);
    }
}

#[test]
fn relation_scope_preparation_preserves_supplied_allocation_stop() {
    let runtime = runtime_with_cardinality_minimum();
    let cancellation = worth_execution::CancellationSource::new();
    let lease = crate::tests::support::test_execution_authority()
        .request_lease(worth_execution::LeaseRequest {
            policy: worth_foundational::ExecutionRequestPolicy::new(
                worth_foundational::ExecutionPosture::Automatic,
                worth_foundational::DeterminismContract::CanonicalBitwise,
                worth_foundational::ExecutionBudget::new(
                    std::num::NonZeroUsize::new(1).unwrap(),
                    0,
                    1,
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
    let view = InvariantRuntimeView::from_runtime(&runtime);
    let plan = entity_plan(1);
    let error = match InvariantExecutionRequest::from_profile_with_contract_at_current_version(
        InvariantRequestProfile::CertificationBoundary,
        &view,
        InvariantObservation::committed(runtime.storage_access().current_edition()),
        runtime.current_version_id(),
        runtime.current_version_id(),
        Some(&plan),
        Some(crate::validation::data::InvariantPlanContract::from_merged_plan(&plan)),
        &control,
    ) {
        Err(error) => error,
        Ok(_) => panic!("cancelled supplied lease must remain stopped"),
    };
    match error {
        TransactionCommitError::Conflict { error, .. } => {
            let denial = error.allocation_denial().expect("full allocation cause");
            assert_eq!(
                denial.kind(),
                worth_execution::ExecutionAllocationDenialKind::Cancelled
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

fn entity_batch(count: usize) -> WorkerIntentBatch {
    WorkerIntentBatch::new("complete-relation-scope").push(entity_intent(count))
}

fn entity_intent(count: usize) -> MutationIntent {
    MutationIntent::Create(CreateIntent::BulkEntities(BulkEntityCreateIntent {
        partition_id: PartitionId::main(),
        kind_id: KindId(1),
        client_keys: (0..count)
            .map(|index| ClientKey::raw(format!("node-{index}")))
            .collect(),
        field_patches: vec![crate::transactions::data::AspectFieldPatch::default(); count],
    }))
}
