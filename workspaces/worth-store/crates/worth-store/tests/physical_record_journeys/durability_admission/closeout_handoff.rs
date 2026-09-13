use worth_proof::TransitionOutcome;
use worth_signal::facade::TemporalDuration;
use worth_store::physical_runtime::{
    lower_physical_durability_performance_receipt, PageBasisPerformanceExpectation,
    PhysicalDurabilityCloseoutOutcome, PhysicalDurabilityPerformanceContract,
    PhysicalDurabilityPerformanceEvidenceDenial, PhysicalManifestCapacityTransition,
    PhysicalMutationDeadline, PhysicalMutationIdempotencyMaterial,
    PhysicalMutationPreparationSuccess, PhysicalMutationRequest, PhysicalRecordInitialization,
    PhysicalRecordOpen, PhysicalRecoveryAttemptBindingFact, PhysicalRecoveryOperationFate,
    RecordAppendBatch,
};

use super::super::durability;
use super::{configuration, media, success};

#[test]
fn clean_close_constructs_the_only_c8_handoff_from_exact_terminal_facts() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let serving = initialized(&root);
    let store = serving.store_identity();

    let shutdown = serving.close();
    let handoff = shutdown
        .durability_closeout()
        .recovery_handoff()
        .expect("clean close produces the sealed C8 handoff");
    assert_eq!(handoff.store_identity(), store);
    assert_eq!(handoff.roots().current().generation(), 1);
    assert!(handoff.roots().previous().is_none());
    assert!(handoff.checkpoint().completed().is_none());
    assert!(handoff.wal_tail().segments().is_empty());
    assert_eq!(handoff.operation_fates().counts().unresolved(), 0);
    assert_eq!(handoff.operation_fates().counts().completed(), 0);
    assert!(handoff.operation_fates().facts().is_empty());
    assert_eq!(
        handoff.backend_evidence().admission_identity(),
        handoff.durability_policy().admission_basis_identity()
    );
    assert_eq!(handoff.recovery_allocation().store_identity(), store);
    assert!(!handoff.residue().requires_inspection());
}

#[test]
fn abort_never_mints_a_recovery_handoff() {
    let parent = tempfile::tempdir().unwrap();
    let shutdown = initialized(&parent.path().join("store")).abort();
    if !matches!(
        shutdown.durability_closeout(),
        PhysicalDurabilityCloseoutOutcome::NotProducedForAbort
    ) {
        panic!("MUTANT_PREDICATE:abort-recovery-handoff-minted");
    }
    assert!(shutdown.durability_closeout().recovery_handoff().is_none());
}

#[test]
fn published_and_reopened_closeout_retains_current_and_immediate_previous_roots() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let serving = initialized(&root);
    let initialized_generation = serving
        .observer()
        .acquisition_snapshot()
        .unwrap()
        .root_generation();
    let (_, placement, _) = configuration();
    let published = serving.certification_publish_single_durable_mutation(
        placement,
        PhysicalManifestCapacityTransition::PreserveCurrent,
        PhysicalMutationIdempotencyMaterial::new([0x71; 32]),
        RecordAppendBatch::try_from_iter([b"closeout-root-lineage".as_slice()]).unwrap(),
    );
    let published_generation = published.current_root().generation();
    assert_eq!(published_generation, initialized_generation + 1);
    let first = serving.close();
    let handoff = first
        .durability_closeout()
        .recovery_handoff()
        .expect("published close retains recovery roots");
    assert_eq!(handoff.roots().current().generation(), published_generation);
    assert_eq!(
        handoff.roots().previous().unwrap().manifest().generation(),
        initialized_generation
    );
    drop(first);

    let (format, _, access) = configuration();
    let reopened_media = media(&root);
    let policy = durability(&reopened_media);
    let reopened =
        success(reopened_media.open_record_store(PhysicalRecordOpen::new(format, access, policy)));
    let reopened_close = reopened.close();
    let reopened_handoff = reopened_close
        .durability_closeout()
        .recovery_handoff()
        .expect("reopened close retains recovery roots");
    assert_eq!(
        reopened_handoff.roots().current().generation(),
        published_generation
    );
    let reopened_previous_generation = reopened_handoff
        .roots()
        .previous()
        .map(|root| root.manifest().generation());
    if reopened_previous_generation != Some(initialized_generation) {
        panic!("MUTANT_PREDICATE:reopened-previous-root-dropped");
    }
}

#[test]
fn completed_but_unobserved_identity_survives_into_operation_fates() {
    let parent = tempfile::tempdir().unwrap();
    let serving = initialized(&parent.path().join("store"));
    let (_, placement, _) = configuration();
    let submission = serving.record_submission();
    let key = submission
        .issue_idempotency_key(PhysicalMutationIdempotencyMaterial::new([0x72; 32]))
        .unwrap();
    let prepared = match submission
        .prepare_durable_append(
            RecordAppendBatch::try_from_iter([b"completed-unobserved-closeout".as_slice()])
                .unwrap(),
            placement,
            PhysicalMutationRequest::platform_durable(
                key,
                PhysicalMutationDeadline::at(
                    TemporalDuration::temporal_duration(1_000_000).unwrap(),
                ),
            ),
        )
        .into_raw()
    {
        TransitionOutcome::Success(PhysicalMutationPreparationSuccess::Prepared(prepared)) => {
            prepared
        }
        _ => panic!("closeout mutation preparation must succeed"),
    };
    let identity = prepared.mutation_identity();
    let handle = prepared.start();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while std::time::Instant::now() < deadline
        && handle.progress().phase()
            != worth_store::physical_runtime::PhysicalMutationProgressPhase::Terminal
    {
        std::thread::yield_now();
    }
    assert_eq!(
        handle.progress().phase(),
        worth_store::physical_runtime::PhysicalMutationProgressPhase::Terminal
    );
    drop(handle);

    let shutdown = serving.close();
    let operation_fates = shutdown
        .durability_closeout()
        .recovery_handoff()
        .unwrap()
        .operation_fates();
    if operation_fates.counts().completed_unobserved() != 1
        || operation_fates.completed_unobserved().len() != 1
    {
        panic!("MUTANT_PREDICATE:completed-unobserved-identity-dropped");
    }
    assert_eq!(operation_fates.counts().completed(), 1);
    assert_eq!(operation_fates.counts().completed_unobserved(), 1);
    assert_eq!(operation_fates.completed_unobserved().len(), 1);
    assert_eq!(
        operation_fates.completed_unobserved()[0].mutation_identity(),
        identity
    );
    let [fact] = operation_fates.facts() else {
        panic!("completed closeout must enumerate one exact operation fact")
    };
    assert_eq!(fact.mutation_identity(), identity);
    assert!(matches!(
        fact.attempt(),
        PhysicalRecoveryAttemptBindingFact::WalBound(_)
    ));
    let PhysicalRecoveryOperationFate::Completed(completed) = fact.fate() else {
        panic!("completed closeout operation must retain completed fate")
    };
    assert_eq!(completed.completed_breadth().current_root_generation(), 2);
    assert_eq!(completed.persisted_records().len(), 1);
}

#[test]
fn unstarted_preparation_remains_an_exact_bounded_unresolved_handoff_fact() {
    let parent = tempfile::tempdir().unwrap();
    let serving = initialized(&parent.path().join("store"));
    let (_, placement, _) = configuration();
    let submission = serving.record_submission();
    let key = submission
        .issue_idempotency_key(PhysicalMutationIdempotencyMaterial::new([0x73; 32]))
        .unwrap();
    let prepared = match submission
        .prepare_durable_append(
            RecordAppendBatch::try_from_iter([b"unresolved-closeout".as_slice()]).unwrap(),
            placement,
            PhysicalMutationRequest::platform_durable(
                key,
                PhysicalMutationDeadline::at(
                    TemporalDuration::temporal_duration(1_000_000).unwrap(),
                ),
            ),
        )
        .into_raw()
    {
        TransitionOutcome::Success(PhysicalMutationPreparationSuccess::Prepared(prepared)) => {
            prepared
        }
        _ => panic!("closeout mutation preparation must succeed"),
    };
    let expected_identity = prepared.mutation_identity();
    let expected_key = prepared.idempotency_identity();
    let expected_fingerprint = prepared.request_fingerprint();
    drop(prepared);

    let shutdown = serving.close();
    let operations = shutdown
        .durability_closeout()
        .recovery_handoff()
        .unwrap()
        .operation_fates();
    assert_eq!(operations.counts().unresolved(), 1);
    let [fact] = operations.facts() else {
        panic!("unresolved closeout must enumerate one exact operation fact")
    };
    assert_eq!(fact.mutation_identity(), expected_identity);
    assert_eq!(fact.idempotency_identity(), expected_key);
    assert_eq!(fact.request_fingerprint(), expected_fingerprint);
    assert!(matches!(
        fact.attempt(),
        PhysicalRecoveryAttemptBindingFact::Unsealed
    ));
    assert!(matches!(
        fact.fate(),
        PhysicalRecoveryOperationFate::Unresolved
    ));
}

#[test]
fn every_governed_performance_claim_requires_exact_closeout_counters() {
    let parent = tempfile::tempdir().unwrap();
    let shutdown = initialized(&parent.path().join("store")).close();
    let summary = shutdown.performance();
    for contract in summary.contracts() {
        lower_physical_durability_performance_receipt(contract, summary)
            .expect("exact observed counters lower one Store performance receipt");
    }
    let mismatch = PhysicalDurabilityPerformanceContract::PageBasis(
        PageBasisPerformanceExpectation::new(1, 1, 1),
    );
    if !matches!(
        lower_physical_durability_performance_receipt(mismatch, summary),
        Err(PhysicalDurabilityPerformanceEvidenceDenial::CounterMismatch)
    ) {
        panic!("MUTANT_PREDICATE:performance-counter-mismatch-accepted");
    }
}

fn initialized(root: &std::path::Path) -> worth_store::physical_runtime::ServingPhysicalRuntime {
    let media = media(root);
    let policy = durability(&media);
    let (format, placement, access) = configuration();
    success(
        media.initialize_record_store(PhysicalRecordInitialization::new(
            format, placement, access, policy,
        )),
    )
}
