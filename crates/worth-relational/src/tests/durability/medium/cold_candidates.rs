//! A global invariant discovers an untouched region after the draft is applied.
use super::*;
use crate::storage::overlay::PartitionAccess;
use crate::symbols::data::ClientKey;
use crate::transactions::data::{CommitPreparationReason, TransactionCommitError};
use crate::validation::data::{InvariantCatalog, InvariantRegistration, InvariantRule};

const REMOTE: PartitionId = PartitionId(41);

#[derive(Clone, Copy, Debug)]
enum PreparationDoor {
    Validation,
    Candidate,
    Commit,
}

fn runtime(
    attachment: Box<dyn crate::durability::medium::RelationalDurableMedium>,
) -> crate::runtime::RelationalRuntime {
    RelationalRuntimeApi::builder()
        .profile(RelationalRuntimeProfile::CertificationCore)
        .schema_registry(test_schema_registry())
        .invariant_catalog(InvariantCatalog {
            registrations: vec![InvariantRegistration::commit_boundary_blocking(
                InvariantRule::unique_entity_aspect_field(aspect_key("name"), field_key("name")),
            )],
        })
        .build_durable(attachment)
}

fn recovered(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> (crate::runtime::RelationalRuntime, Arc<MemberSource>, usize) {
    let producer = runtime(TestDurableMedium::new().attachment());
    create_entity_in_partition(execution, &producer, "remote-value", REMOTE);
    let (recovery, source, _) = fixture_from_runtime(execution, producer, false, 0);
    source.damaged.store(false, Ordering::Relaxed);
    let attachment = TestDurableMedium::new().attachment();
    for bytes in source.commits.values() {
        let artifact = crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifact::from_untrusted_bytes(bytes.clone()).unwrap();
        assert!(matches!(
            attachment.publish(&artifact),
            crate::durability::medium::RelationalDurablePublication::Committed
        ));
    }
    let mut receiving = runtime(attachment);
    let plan = receiving
        .durability()
        .recovery_plan_from_source(
            execution,
            recovery,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    receiving
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();
    assert_eq!(source.reads.load(Ordering::Relaxed), 0);
    let (_, basis) = receiving
        .observe_branch(&receiving.main_branch_identity())
        .unwrap();
    basis
        .inner
        .root
        .demand_regions_full(execution, [PartitionId::main()])
        .unwrap();
    assert!(matches!(
        basis.inner.root.probe_full_partition(REMOTE),
        crate::storage::residency::PartitionProbe::Cold
    ));
    let floor = basis
        .inner
        .root
        .get_partition(PartitionId::main())
        .unwrap()
        .entity_arena
        .slot_count();
    (receiving, source, floor)
}

fn transaction(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &crate::runtime::RelationalRuntime,
) -> crate::mvcc::BranchBoundRelationalTransaction {
    let mut transaction = test_owner_begin_transaction_for_main(runtime);
    transaction
        .push_batch(
            WorkerIntentBatch::new("candidate-probe").push(MutationIntent::Create(
                CreateIntent::Entity(crate::transactions::data::EntitySpec {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(1),
                    client_key: ClientKey::raw("candidate-value"),
                    fields: name_field_patch("candidate-value"),
                }),
            )),
            execution.allocation_policy(),
        )
        .unwrap();
    transaction
}

fn storage(error: TransactionCommitError) -> crate::storage::data::RelationalStorageUnavailable {
    let TransactionCommitError::Preparation { error, .. } = error else {
        panic!("candidate storage failure is pre-effect: {error:?}");
    };
    let CommitPreparationReason::StorageUnavailable(failure) = error.reason() else {
        panic!("storage refusal retains its native payload: {error:?}");
    };
    failure
}

#[test]
fn candidate_invariant_discovery_refuses_stickily_without_publishing_round_effects() {
    let backing = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&backing);
    for door in [
        PreparationDoor::Validation,
        PreparationDoor::Candidate,
        PreparationDoor::Commit,
    ] {
        let (runtime, source, _) = recovered(execution);
        let before = runtime.performance_access().counters();
        let diagnostics = runtime.publication.diagnostics.snapshot();
        let dictionary = runtime.services.symbols.snapshot();
        let frontier = runtime.history().latest_patch_stream_position();
        let offered = attached_test_medium(&runtime).offered().len();
        source.damaged.store(true, Ordering::Relaxed);
        let run = || {
            let transaction = transaction(execution, &runtime);
            match door {
                PreparationDoor::Validation => {
                    transaction.validate(execution, &runtime).map(|_| ())
                }
                PreparationDoor::Candidate => runtime
                    .prepare_branch_transaction(execution, transaction)
                    .map(|_| ()),
                PreparationDoor::Commit => transaction.commit(execution, &runtime).map(|_| ()),
            }
            .map_err(storage)
        };
        let failure =
            run().expect_err("an untouched cold invariant region must not become absence");
        assert_eq!(
            failure.subject,
            crate::storage::data::RelationalStorageSubject::Region(REMOTE)
        );
        assert_eq!(failure.cause, Cause::IntegrityMismatch);
        let reads = source.reads.load(Ordering::Relaxed);
        source.damaged.store(false, Ordering::Relaxed);
        assert_eq!(run(), Err(failure));
        assert_eq!(source.reads.load(Ordering::Relaxed), reads);
        let after = runtime.performance_access().counters();
        assert_eq!(
            after.preparation_packet_count,
            before.preparation_packet_count
        );
        assert_eq!(
            after.invariant_entity_slot_scans,
            before.invariant_entity_slot_scans
        );
        assert_eq!(runtime.publication.diagnostics.snapshot(), diagnostics);
        assert_eq!(runtime.services.symbols.snapshot(), dictionary);
        assert_eq!(runtime.history().latest_patch_stream_position(), frontier);
        assert_eq!(attached_test_medium(&runtime).offered().len(), offered);
    }
}

#[test]
fn a_region_retry_spends_the_applied_draft_identity_and_settles_one_round() {
    let backing = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&backing);
    let (cold, source, floor) = recovered(execution);
    let (warm, _, warm_floor) = recovered(execution);
    let (_, basis) = warm.observe_branch(&warm.main_branch_identity()).unwrap();
    basis
        .inner
        .root
        .demand_regions_full(execution, [REMOTE])
        .unwrap();
    let before_cold = cold.performance_access().counters();
    let before_warm = warm.performance_access().counters();
    let cold_commit = transaction(execution, &cold)
        .commit(execution, &cold)
        .unwrap();
    let warm_commit = transaction(execution, &warm)
        .commit(execution, &warm)
        .unwrap();
    let id = changed_entities(&cold_commit)[0];
    assert!(
        id.slot_index() > floor,
        "the discarded applied candidate cannot reuse an identity"
    );
    assert!(
        id.slot_index() < floor + 16,
        "retry growth has bounded headroom"
    );
    let warm_slot = changed_entities(&warm_commit)[0].slot_index();
    assert!((warm_floor..warm_floor + 16).contains(&warm_slot));
    assert!(
        id.slot_index() > warm_slot,
        "cold discovery abandons an extra applied identity"
    );
    let after_cold = cold.performance_access().counters();
    let after_warm = warm.performance_access().counters();
    assert!(after_cold.preparation_packet_count > before_cold.preparation_packet_count);
    assert_eq!(
        after_cold.preparation_packet_count - before_cold.preparation_packet_count,
        after_warm.preparation_packet_count - before_warm.preparation_packet_count
    );
    assert_eq!(
        after_cold.invariant_entity_slot_scans - before_cold.invariant_entity_slot_scans,
        after_warm.invariant_entity_slot_scans - before_warm.invariant_entity_slot_scans
    );
    assert!(source.reads.load(Ordering::Relaxed) < 24);
    release_test_commit_snapshot(&cold, &cold_commit);
    release_test_commit_snapshot(&warm, &warm_commit);
}
