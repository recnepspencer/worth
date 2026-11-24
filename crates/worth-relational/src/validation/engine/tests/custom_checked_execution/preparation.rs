//! Scope preparation reads, request stops and scratch admission.
use super::*;
use crate::config::data::RelationalRuntimeConfig;
use crate::identity::data::VersionId;
use crate::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation;
use crate::schema::data::{EntityKindRegistration, SchemaId, SchemaVersionId};
use crate::storage::overlay::small_relation_partition::relation_partition_for_test;
use crate::validation::engine::test_view::invariant_view_with_custom_rules;
use std::collections::BTreeMap;

pub(super) fn checked_structural_registry() -> RelationalSchemaRegistry {
    RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(1),
            kind_name: "checked-structural".to_owned(),
            schema_id: SchemaId("test".to_owned()),
            schema_version_id: SchemaVersionId(1),
            aspect_contract_declarations: Default::default(),
        })
        .unwrap()
}

#[test]
fn checked_custom_preparation_reads_declared_structural_state() {
    let target = Arc::new(OnceLock::new());
    let preparation_visits = Arc::new(AtomicUsize::new(0));
    let evaluation_visits = Arc::new(AtomicUsize::new(0));
    let mut config = RelationalRuntimeConfig::default();
    config.schema.registry = checked_structural_registry();
    let id = PartitionId::main();
    let (partition, entities, _) = relation_partition_for_test(id, 1, &[]);
    let partitions = BTreeMap::from([(id, partition)]);
    let instrumentation = RuntimeInstrumentation::default();
    let version = VersionId(1);
    let view = invariant_view_with_custom_rules(
        &instrumentation,
        config,
        version,
        vec![CustomInvariantRegistration::new(CheckedCounterRule {
            maximum_work_units: NonZeroU64::new(100_000),
            steps: 1,
            visits: Arc::clone(&evaluation_visits),
            preparation_visits: Arc::clone(&preparation_visits),
            cancellation: None,
            preparation_steps: 2,
            preparation_scratch_bytes: 64,
            target: Some(Arc::clone(&target)),
        })
        .unwrap()],
    );
    target.set(entities[0]).unwrap();
    let request = InvariantExecutionRequest::from_profile_with_contract(
        InvariantRequestProfile::CommitBoundary,
        &view,
        InvariantObservation::committed_branch(&partitions),
        version,
        None,
        None,
    );
    let result = InvariantEngine::from_view(&view)
        .execute(
            &worth_execution::ExecutionRequest::leased(&checked_lease(
                8 * 1024 * 1024,
                1_000_000,
                CancellationToken::new(),
            )),
            request,
        )
        .expect("checked structural preparation and evaluation succeed");
    assert_eq!(preparation_visits.load(Ordering::SeqCst), 2);
    assert_eq!(evaluation_visits.load(Ordering::SeqCst), 1);
    assert!(matches!(
        result.results()[0].verdict,
        crate::validation::data::execution::InvariantVerdict::Pass
    ));
}

#[test]
fn checked_custom_preparation_observes_cancellation_mid_scan() {
    let cancellation = CancellationSource::new();
    let preparation_visits = Arc::new(AtomicUsize::new(0));
    let evaluation_visits = Arc::new(AtomicUsize::new(0));
    let partitions = BTreeMap::new();
    let instrumentation = RuntimeInstrumentation::default();
    let version = VersionId(0);
    let view = invariant_view_with_custom_rules(
        &instrumentation,
        RelationalRuntimeConfig::default(),
        version,
        vec![CustomInvariantRegistration::new(CheckedCounterRule {
            maximum_work_units: NonZeroU64::new(100_000),
            steps: 1,
            visits: Arc::clone(&evaluation_visits),
            preparation_visits: Arc::clone(&preparation_visits),
            cancellation: Some(cancellation.clone()),
            preparation_steps: 1_000,
            preparation_scratch_bytes: 64,
            target: None,
        })
        .unwrap()],
    );
    let lease = checked_lease(8 * 1024 * 1024, 10_000, cancellation.token());
    let request = InvariantExecutionRequest::from_profile_with_contract(
        InvariantRequestProfile::CommitBoundary,
        &view,
        InvariantObservation::committed_branch(&partitions),
        version,
        None,
        None,
    );
    let result = InvariantEngine::from_view(&view)
        .execute(&worth_execution::ExecutionRequest::leased(&lease), request);
    assert_eq!(preparation_visits.load(Ordering::SeqCst), 2);
    assert_eq!(evaluation_visits.load(Ordering::SeqCst), 0);
    assert!(matches!(
        result,
        Err(
            crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError::Execution {
                denial: crate::transactions::data::outcomes::transaction_commit_error::CommitExecutionDenial {
                    kind: crate::transactions::data::outcomes::transaction_commit_error::CommitExecutionDenialKind::Cause(
                        crate::execution::RelationalExecutionDenialCause::Cancelled
                    ),
                    ..
                },
                ..
            }
        )
    ));
}

#[test]
fn checked_custom_preparation_denies_memory_before_large_scratch_claim() {
    const REQUEST_MEMORY: u64 = 128 * 1024;
    // One claim larger than the entire request cannot be funded at any posture.
    let evaluation_visits = Arc::new(AtomicUsize::new(0));
    let partitions = BTreeMap::new();
    let instrumentation = RuntimeInstrumentation::default();
    let version = VersionId(0);
    let view = invariant_view_with_custom_rules(
        &instrumentation,
        RelationalRuntimeConfig::default(),
        version,
        vec![CustomInvariantRegistration::new(CheckedCounterRule {
            maximum_work_units: NonZeroU64::new(100_000),
            steps: 1,
            visits: Arc::clone(&evaluation_visits),
            preparation_visits: Arc::new(AtomicUsize::new(0)),
            cancellation: None,
            preparation_steps: 1,
            preparation_scratch_bytes: REQUEST_MEMORY + 1,
            target: None,
        })
        .unwrap()],
    );
    let lease = checked_lease(REQUEST_MEMORY, 10_000, CancellationToken::new());
    let request = InvariantExecutionRequest::from_profile_with_contract(
        InvariantRequestProfile::CommitBoundary,
        &view,
        InvariantObservation::committed_branch(&partitions),
        version,
        None,
        None,
    );
    let result = InvariantEngine::from_view(&view)
        .execute(&worth_execution::ExecutionRequest::leased(&lease), request);
    assert_eq!(evaluation_visits.load(Ordering::SeqCst), 0);
    // The claim is charged to the lease before it is made, so the request's
    // own memory policy refuses it: the claim alone exceeds the request.
    let Err(crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError::Execution {
        denial:
            crate::transactions::data::outcomes::transaction_commit_error::CommitExecutionDenial {
                kind:
                    crate::transactions::data::outcomes::transaction_commit_error::CommitExecutionDenialKind::Cause(
                        crate::execution::RelationalExecutionDenialCause::PolicyMemoryExhausted {
                            requested,
                            admitted,
                            ..
                        },
                    ),
                ..
            },
        ..
    }) = result
    else {
        panic!("the request's memory policy must refuse the claim: {result:?}");
    };
    assert!(requested > REQUEST_MEMORY, "{requested}");
    assert!(admitted <= REQUEST_MEMORY, "{admitted}");
}
