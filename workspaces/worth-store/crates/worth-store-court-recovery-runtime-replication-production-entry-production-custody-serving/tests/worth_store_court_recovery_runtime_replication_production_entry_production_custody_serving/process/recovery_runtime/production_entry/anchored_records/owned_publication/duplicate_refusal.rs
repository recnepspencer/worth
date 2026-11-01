//! Genuine colliding admission, not a fabricated terminal or cancellation result.
use super::*;
use sha2::{Digest, Sha256};
use worth_proof::{NonEmpty, TransitionOutcome};

#[test]
fn refused_duplicate_observes_only_the_original_managed_fate_and_retires_its_inputs() {
    duplicate_refusal(true);
}
#[test]
fn refused_duplicate_without_a_local_attempt_is_immediately_inspection_only() {
    duplicate_refusal(false);
}
fn duplicate_refusal(managed: bool) {
    let world = world(if managed {
        "owned-duplicate-managed"
    } else {
        "owned-duplicate-wal-only"
    });
    let records = world.serving().anchored_records().unwrap();
    let memory = SerialMemoryBudget::new(None);
    let cancellation = CancellationSource::new();
    let port = Port::new(&memory, 1, Delivery::RefuseRegistration);
    let (operation, recovery) = start(&world, 1, &memory, &cancellation, None);
    let original_memory = SerialMemoryBudget::new(None);
    // A real original caller retains the original input ticket through its step.
    let original_input = original_memory.reserve(MEMBER_BYTES as u64).unwrap();
    let original_cancellation = CancellationSource::new();
    let prepared = prepare_colliding_member(
        &world,
        operation.observe().identity(),
        &original_memory,
        &original_cancellation,
    );
    let identity = prepared.mutation_identity();
    assert_eq!(
        prepared.disposition(),
        PhysicalMutationAdmissionDisposition::Fresh
    );
    let gate = managed.then(|| pause(world.serving(), PhysicalMutationCheckpoint::AfterGroupSeal));
    let mut sealed = None;
    if let Some(gate) = &gate {
        let handle = prepared.start();
        assert!(gate.0.await_arrival());
        assert_eq!(handle.identity(), identity);
        drop(handle); // Losing the original observer does not stop accepted work.
    } else {
        let appended = world
            .serving()
            .certification_record_submission()
            .append_prepared_wal_group(NonEmpty::new(prepared, Vec::new()));
        let PhysicalWalGroupAppendOutcome::Appended(actual) = appended else {
            panic!("real WAL group was not sealed");
        };
        sealed = Some(actual);
    }
    let failure = match operation.step(&records, &port) {
        AnchoredRecordPublicationStep::Failed(failure) => failure,
        _ => panic!("refused duplicate cannot advance"),
    };
    let cause = AnchoredRecordStepSettlementFailure::UnstartedCancellationRefused(
        PhysicalPreSealCancellationDenial::GroupAlreadySealed,
    );
    assert!(
        matches!(failure.cause(), AnchoredRecordOperationFailure::StepSettlement(actual) if *actual == cause)
    );
    assert_eq!(failure.observe().accepted_mutation(), Some(identity));
    let registration = port.take_registration();
    assert_eq!(registration.mutation_identity(), identity);
    let stopped = failure.into_stopped();
    drop(recovery);
    drop(port);
    cancellation.cancel();
    assert_eq!(
        memory.charged_memory_bytes(),
        0,
        "refused duplicate retained its payload ticket or original control backing"
    );
    assert!(
        original_memory.charged_memory_bytes() > MEMBER_BYTES as u64,
        "duplicate observation displaced the original step controls"
    );
    assert_eq!(
        world.serving().physical_mutation_observation().started(),
        u64::from(managed)
    );
    if let Some(gate) = gate {
        let stopped = match stopped.settle(&records) {
            AnchoredRecordStoppedPoll::Pending(same) => same,
            _ => panic!("actual original worker remains paused"),
        };
        assert!(
            registration.terminal().is_none(),
            "no early terminal from saved base fact"
        );
        gate.0.release();
        // Read-only registration can retain the actual fate after controller loss.
        drop(stopped);
        let terminal = wait_terminal(&registration);
        assert_eq!(terminal.mutation_identity(), identity);
        assert_eq!(
            terminal.settlement_failure(),
            Some(cause),
            "actual fact erased this call's refusal"
        );
        let PhysicalMutationOutcome::Completed(actual) = terminal.outcome() else {
            panic!("original managed fate was replaced");
        };
        assert_eq!(actual.persisted_records().len(), 1);
        assert_eq!(world.serving().physical_mutation_observation().started(), 1);
        assert_eq!(
            world.serving().physical_mutation_observation().completed(),
            1
        );
        assert_eq!(original_memory.charged_memory_bytes(), MEMBER_BYTES as u64);
    } else {
        assert!(registration.terminal().is_none());
        let failure = match stopped.settle(&records) {
            AnchoredRecordStoppedPoll::Failed(failure) => failure,
            _ => panic!("no local attempt cannot become Pending or a fabricated terminal"),
        };
        assert!(
            matches!(failure.cause(), AnchoredRecordOperationFailure::StepSettlement(actual) if *actual == cause)
        );
        assert!(failure.observe().step_terminal().is_none());
        assert_eq!(failure.observe().accepted_mutation(), Some(identity));
        drop(failure);
    }
    assert!(
        records.read(name(), 1024).unwrap().is_none(),
        "no worker or observer submits the anchor"
    );
    drop(sealed);
    drop(original_input);
    assert_eq!(original_memory.charged_memory_bytes(), 0);
    assert!(registration
        .terminal()
        .as_ref()
        .is_none_or(|t| t.settlement_failure() == Some(cause)));
}
fn prepare_colliding_member(
    world: &worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld,
    identity: AnchoredRecordPublicationIdentity,
    memory: &SerialMemoryBudget,
    cancellation: &CancellationSource,
) -> PreparedPhysicalMutation {
    // Adversarial setup deliberately aliases the door's first-member key using
    // its ACTUAL issued correlation. Actual owner outcomes are the oracle.
    let mut sha = Sha256::new();
    sha.update(b"worth.store.anchored-record.members.v1");
    sha.update(identity.runtime_identity().get().to_le_bytes());
    sha.update([identity.name().as_str().len() as u8]);
    sha.update(identity.name().as_str().as_bytes());
    sha.update(identity.generation().to_le_bytes());
    sha.update(identity.content_sha256());
    sha.update(0_u64.to_le_bytes());
    let submission = world.serving().record_submission();
    let key = submission
        .issue_idempotency_key(PhysicalMutationIdempotencyMaterial::new(
            sha.finalize().into(),
        ))
        .unwrap();
    let request = PhysicalMutationRequest::platform_durable(
        key,
        PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
    )
    .with_interruption_view(view(memory, cancellation, None));
    let batch = RecordAppendBatch::builder()
        .push_owned(vec![1; MEMBER_BYTES])
        .build()
        .unwrap();
    let TransitionOutcome::Success(PhysicalMutationPreparationSuccess::Prepared(actual)) =
        submission
            .prepare_durable_append(batch, world.placement(), request)
            .into_raw()
    else {
        panic!("real original member preparation");
    };
    actual
}
