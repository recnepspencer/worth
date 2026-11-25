//! Guard only the actual memory cutover; user preparation and Drop stay outside.
use super::*;
use std::sync::{atomic::Ordering, Arc};
use worth_relational::facade::mvcc::{
    RelationalInterruptionBoundary, RelationalOperationControl, RelationalOperationInterruption,
};
mod deadline;
mod payload;
use payload::{register, Probe};

#[test]
fn real_companion_preparation_precedes_handoff_and_retired_payload_drops_after_it() {
    let host = serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let runtime = fixture().build_runtime();
    let original = create(execution, &runtime, "control", "allowed");
    let source = runtime
        .admit_branch_basis(&runtime.main_branch_identity())
        .unwrap();
    let probe = Arc::new(Probe::default());
    let _registration = register(&runtime, Arc::clone(&probe));
    let candidate = prepare(execution, &runtime, &source);
    let scoped = runtime
        .publication_port()
        .with_prepared_memory_cutover(execution, candidate, |cutover| {
            assert_eq!(probe.prepares.load(Ordering::SeqCst), 1);
            let _active = probe.enter();
            cutover.perform();
            assert!(
                probe.events().is_empty(),
                "an owning payload died under the guard"
            );
            Err::<(), _>("caller status cannot replace actual performed custody")
        })
        .unwrap();
    let (handoff, outcome) = scoped.into_parts();
    assert!(handoff.is_err());
    assert!(
        probe.events().contains(&(0, false)),
        "the original retired image must actually drop"
    );
    assert!(probe.events().iter().all(|(_, active)| !active));
    let RelationalPublicationOutcome::Performed(performed) = outcome.unwrap() else {
        panic!("the actual cutover result must survive the caller's status");
    };
    let result = runtime
        .settlement_port()
        .settle_performed_publication(performed)
        .unwrap();
    assert_eq!(changed_entities(&result).len(), 1);
    release_test_commit_snapshot(&runtime, &result);
    release_test_commit_snapshot(&runtime, &original);
}

#[test]
fn refused_cutover_keeps_prepared_user_payload_alive_until_handoff_ends() {
    let host = serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let runtime = fixture().build_runtime();
    let original = create(execution, &runtime, "control", "allowed");
    let source = runtime
        .admit_branch_basis(&runtime.main_branch_identity())
        .unwrap();
    let probe = Arc::new(Probe::default());
    let _registration = register(&runtime, Arc::clone(&probe));
    let control = RelationalOperationControl::uninterrupted().with_injected_interruption_for_test(
        RelationalInterruptionBoundary::BeforeCriticalSection,
        RelationalOperationInterruption::Cancelled,
        std::num::NonZeroUsize::MIN,
    );
    let candidate = prepare_with_control(execution, &runtime, &source, control);
    let (_, outcome) = runtime
        .publication_port()
        .with_prepared_memory_cutover(execution, candidate, |cutover| {
            let _active = probe.enter();
            cutover.perform();
            assert!(
                probe.events().is_empty(),
                "refusal destroyed a payload under the guard"
            );
        })
        .unwrap()
        .into_parts();
    assert!(matches!(
        outcome,
        Some(RelationalPublicationOutcome::Interrupted(_))
    ));
    assert!(
        probe.events().contains(&(1, false)),
        "the actual refused successor must be released"
    );
    assert!(probe.events().iter().all(|(_, active)| !active));
    assert_eq!(
        runtime
            .admit_branch_basis(source.identity())
            .unwrap()
            .descriptor(),
        source.descriptor()
    );
    release_test_commit_snapshot(&runtime, &original);
}

#[test]
fn unused_cutover_token_releases_original_preflight_without_moving_the_head() {
    let host = serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let runtime = fixture().build_runtime();
    let original = create(execution, &runtime, "control", "allowed");
    let source = runtime
        .admit_branch_basis(&runtime.main_branch_identity())
        .unwrap();
    let probe = Arc::new(Probe::default());
    let _registration = register(&runtime, Arc::clone(&probe));
    let candidate = prepare(execution, &runtime, &source);
    let (caller, outcome) = runtime
        .publication_port()
        .with_prepared_memory_cutover(execution, candidate, |_cutover| {
            let _active = probe.enter();
            assert!(probe.events().is_empty());
            7
        })
        .unwrap()
        .into_parts();
    assert_eq!(caller, 7);
    assert!(outcome.is_none());
    assert!(probe.events().contains(&(1, false)));
    assert_eq!(
        runtime
            .admit_branch_basis(source.identity())
            .unwrap()
            .descriptor(),
        source.descriptor()
    );
    // Original source and companion remain usable after skipped work releases.
    let result = perform(execution, &runtime, prepare(execution, &runtime, &source));
    assert_eq!(changed_entities(&result).len(), 1);
    release_test_commit_snapshot(&runtime, &result);
    release_test_commit_snapshot(&runtime, &original);
}

#[test]
fn unwinding_handoff_drops_user_payload_after_the_guard_unwinds() {
    let host = serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let runtime = fixture().build_runtime();
    let original = create(execution, &runtime, "control", "allowed");
    let source = runtime
        .admit_branch_basis(&runtime.main_branch_identity())
        .unwrap();
    let probe = Arc::new(Probe::default());
    let _registration = register(&runtime, Arc::clone(&probe));
    let candidate = prepare(execution, &runtime, &source);
    let caught = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _ = runtime.publication_port().with_prepared_memory_cutover(
            execution,
            candidate,
            |_cutover| {
                let _active = probe.enter();
                panic!("stop after actual preflight, before cutover");
            },
        );
    }));
    assert!(caught.is_err());
    assert!(probe.events().contains(&(1, false)));
    assert!(probe.events().iter().all(|(_, active)| !active));
    assert_eq!(
        runtime
            .admit_branch_basis(source.identity())
            .unwrap()
            .descriptor(),
        source.descriptor()
    );
    release_test_commit_snapshot(&runtime, &original);
}

#[test]
fn attached_medium_refuses_scoped_cutover_before_callback_or_medium_access() {
    let host = serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let runtime = worth_relational::facade::runtime::RelationalRuntimeApi::builder()
        .schema_registry(fixture().build_registry())
        .build_durable(Box::new(
            super::super::super::unaccessed_medium::UnaccessedMedium,
        ))
        .unwrap();
    let source = runtime
        .admit_branch_basis(&runtime.main_branch_identity())
        .unwrap();
    let candidate = prepare(execution, &runtime, &source);
    let result =
        runtime
            .publication_port()
            .with_prepared_memory_cutover(execution, candidate, |_cutover| {
                panic!("unsupported durability entered the guard callback")
            });
    assert!(matches!(
        result,
        Err(RelationalPublicationOutcome::Denied(
            RelationalPublicationDenial::ScopedCutoverDurabilityUnsupported
        ))
    ));
    assert_eq!(
        runtime
            .admit_branch_basis(source.identity())
            .unwrap()
            .descriptor(),
        source.descriptor()
    );
}

#[test]
fn relation_scope_preparation_interruption_preserves_head_and_allows_retry() {
    use worth_relational::facade::{
        identity::{KindId, PartitionId},
        symbols::ClientKey,
        transactions::{TransactionCommitError, WorkerIntentBatch},
    };

    let host_request = serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    for interruption in [
        RelationalOperationInterruption::Cancelled,
        RelationalOperationInterruption::TimedOut,
    ] {
        let runtime = worth_relational::facade::runtime::RelationalRuntimeApi::builder()
            .schema_registry(partition_isolation_registry())
            .build()
            .unwrap();
        let identity = runtime.main_branch_identity();
        let (_, basis) = runtime.observe_branch(&identity).unwrap();
        let before = runtime
            .branch_reference_state(&worth_relational::facade::history::BranchId("main".into()))
            .unwrap();
        // The original validation boundary polls only three times. Visit eight
        // requires the scope collector's actual per-item safe points.
        let control = RelationalOperationControl::uninterrupted()
            .with_injected_interruption_for_test(
                RelationalInterruptionBoundary::ProposalValidation,
                interruption,
                std::num::NonZeroUsize::new(8).expect("positive fixture visit"),
            );
        let mut transaction = runtime
            .begin_branch_transaction_with_control(
                &basis,
                worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
                control,
            )
            .unwrap();
        transaction
            .push_batch(
                WorkerIntentBatch::new("complete-relation-scope").push(entity_intent(3)),
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap();
        let error = match transaction.validate(execution, &runtime) {
            Err(error) => error,
            Ok(_) => panic!("scope scan must observe the requested stop"),
        };
        match error {
            TransactionCommitError::Interrupted {
                interruption: event,
                ..
            } => {
                assert_eq!(event.interruption(), interruption);
                assert_eq!(
                    event.boundary(),
                    RelationalInterruptionBoundary::ProposalValidation
                );
            }
            other => panic!("expected exact native interruption, got {other:?}"),
        }
        assert_eq!(
            runtime
                .branch_reference_state(&worth_relational::facade::history::BranchId("main".into()))
                .unwrap(),
            before
        );
        let mut retry = runtime
            .begin_branch_transaction(
                &basis,
                worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
            )
            .unwrap();
        retry
            .push_batch(
                WorkerIntentBatch::new("complete-relation-scope").push(entity_intent(3)),
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap();
        let outcome = retry.commit(execution, &runtime).unwrap();
        for index in 0..3 {
            assert!(outcome
                .created_entity(&worth_relational::facade::transactions::CreatedEntityRef {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(1),
                    client_key: ClientKey::raw(format!("node-{index}")),
                })
                .is_some());
        }
        release_test_commit_snapshot(&runtime, &outcome);
    }
}

fn partition_isolation_registry() -> worth_relational::facade::schema::RelationalSchemaRegistry {
    use worth_relational::facade::{
        config::{CascadeDeletePolicy, CrossContextPolicy},
        identity::KindId,
        schema::{
            EntityKindRegistration, KindAspectContractDeclarations,
            PartitionIsolationContractDeclaration, PartitionIsolationMode,
            RelationIntegrityDeclarations, RelationKindRegistration, RelationalSchemaRegistry,
            SchemaId, SchemaVersionId,
        },
    };
    RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(1),
            kind_name: "geom.vertex".to_string(),
            schema_id: SchemaId("test".to_string()),
            schema_version_id: SchemaVersionId(1),
            aspect_contract_declarations: KindAspectContractDeclarations::default(),
        })
        .and_then(|registry| {
            registry.register_relation_kind(RelationKindRegistration {
                kind_id: KindId(2),
                kind_name: "geom.edge".to_string(),
                schema_id: SchemaId("test".to_string()),
                schema_version_id: SchemaVersionId(1),
                cross_context_policy: CrossContextPolicy::AllowExplicit,
                cascade_delete_policy: CascadeDeletePolicy::CascadeDeleteRelations,
                aspect_contract_declarations: KindAspectContractDeclarations::default(),
                relation_integrity: RelationIntegrityDeclarations::default()
                    .with_partition_isolation_contracts(vec![
                        PartitionIsolationContractDeclaration {
                            contract_id: "same_partition".into(),
                            isolation_mode: PartitionIsolationMode::SamePartitionEndpoints,
                        },
                    ]),
            })
        })
        .unwrap()
}

fn entity_intent(count: usize) -> worth_relational::facade::transactions::MutationIntent {
    use worth_relational::facade::{
        identity::{KindId, PartitionId},
        symbols::ClientKey,
        transactions::{AspectFieldPatch, BulkEntityCreateIntent, CreateIntent, MutationIntent},
    };
    MutationIntent::Create(CreateIntent::BulkEntities(BulkEntityCreateIntent {
        partition_id: PartitionId::main(),
        kind_id: KindId(1),
        client_keys: (0..count)
            .map(|index| ClientKey::raw(format!("node-{index}")))
            .collect(),
        field_patches: vec![AspectFieldPatch::default(); count],
    }))
}
