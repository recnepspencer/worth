use std::{fs, num::NonZeroU64};

use worth_proof::{NonEmpty, TransitionOutcome};
use worth_store::physical_runtime::certification::CertificationPhysicalExecutionCheckpoint;
use worth_store::physical_runtime::{
    PhysicalCheckpointCaptureFailureKind, PhysicalCheckpointOutcome,
    PhysicalCheckpointProgressPhase, PhysicalCheckpointProvenNoEffectCause,
    PhysicalDataDispatchOutcome, PhysicalDataSettlementOutcome,
    PhysicalMutationIdempotencyMaterial, PhysicalOperationAllocationScope,
    PhysicalResidencyAllocationBoundaryKind, PhysicalResidencyDenial, PhysicalResidencyDimension,
    PhysicalWalGroupAppendOutcome, PhysicalWalGroupBarrierOutcome,
};
use worth_store_physical_backend::MediaOperationRole;

use super::{checkpoint_request, configuration, serving_from_initialization};

#[test]
fn live_operation_pressure_removes_created_candidate_without_capture_allocation_and_allows_retry() {
    let parent = tempfile::tempdir().unwrap();
    let store_root = parent.path().join("store");
    let serving = serving_from_initialization(&store_root);
    let (_, placement, _) = configuration();
    let submission = serving.certification_record_submission();
    let prepared = super::super::wal_append::prepared(
        &submission,
        placement,
        PhysicalMutationIdempotencyMaterial::new([0x6d; 32]),
        b"dirty checkpoint capture under live operation pressure",
    );
    let appended = match submission.append_prepared_wal_group(NonEmpty::new(prepared, Vec::new())) {
        PhysicalWalGroupAppendOutcome::Appended(appended) => appended,
        _ => panic!("capture setup requires an actual appended WAL member"),
    };
    let durable = match submission.synchronize_appended_wal_group(appended) {
        PhysicalWalGroupBarrierOutcome::Durable(durable) => {
            durable.into_members().into_vec().pop().unwrap()
        }
        _ => panic!("capture setup requires the actual WAL barrier"),
    };
    let dispatched = match submission.dispatch_wal_durable_data(durable) {
        PhysicalDataDispatchOutcome::Dispatched(dispatched) => dispatched,
        _ => panic!("capture setup requires a real data effect"),
    };
    assert_eq!(dispatched.effects().len(), 1);
    let coordinate = dispatched.effects()[0].coordinate();
    assert!(matches!(
        dispatched.settle_exact_effects(),
        PhysicalDataSettlementOutcome::Settled(_)
    ));
    let residency = serving.certification_physical_residency();
    let dirty = residency
        .admit_dirty_frame(
            residency.pin_exact(coordinate).unwrap(),
            |source, target| {
                target.copy_from_slice(source);
                let last = target.len() - 1;
                target[last] ^= 1;
            },
        )
        .unwrap();
    assert_eq!(serving.residency_observation().counters().dirty_frames(), 1);

    let gate = serving.certification_pause_physical_execution_at(
        CertificationPhysicalExecutionCheckpoint::BeforeBackendDispatch,
    );
    let handle = match serving
        .checkpoints()
        .start(checkpoint_request(0x6d))
        .into_raw()
    {
        TransitionOutcome::Success(handle) => handle,
        TransitionOutcome::Failed(failure) => panic!("checkpoint funding failed: {failure:?}"),
        _ => panic!("checkpoint must be funded before competing pressure"),
    };
    let identity = handle.identity();
    let candidate = store_root.join(format!(
        "staging/checkpoint-{:016x}.candidate",
        identity.sequence().get()
    ));
    assert!(gate.await_arrival());
    assert!(!candidate.exists());

    let observation = serving.residency_observation();
    let limit = observation.admitted_policy().operation_bytes();
    let scope = PhysicalOperationAllocationScope::Maintenance;
    // Maintenance may use the progress headroom that Recovery intentionally preserves.
    // Saturate observed live operation occupancy, not a predicted capture-window size.
    let blocker = residency
        .admit_operation_scope(
            scope,
            NonZeroU64::new(limit - observation.counters().active_operation_bytes() - 1).unwrap(),
        )
        .unwrap();
    assert_eq!(
        serving
            .residency_observation()
            .counters()
            .active_operation_bytes(),
        limit - 1
    );
    let blocked = serving.residency_observation().allocations();
    let trace_start = residency.allocation_trace().event_count();
    let creation_release = gate.release_arrival(0).unwrap();
    assert!(creation_release.await_resumption());
    assert!(gate.await_arrivals(2));
    assert_eq!(
        handle.progress().phase(),
        PhysicalCheckpointProgressPhase::CandidateCleanup
    );
    assert!(
        candidate.exists(),
        "native capture refusal follows actual candidate creation"
    );
    assert_eq!(handle.progress().current_capture_bytes(), 0);
    assert_eq!(handle.progress().peak_capture_bytes(), 0);
    assert!(!store_root.join("families/checkpoint.current").exists());
    assert!(residency
        .allocation_trace()
        .events()
        .skip(trace_start)
        .all(|event| {
            event.kind() != PhysicalResidencyAllocationBoundaryKind::Actualization
                || event.scope() != Some(scope)
        }));
    let during = serving.residency_observation().allocations();
    let dimension = PhysicalResidencyDimension::OperationScope(scope);
    assert_eq!(
        during.for_dimension(dimension).admissions(),
        blocked.for_dimension(dimension).admissions()
    );
    assert_eq!(
        during.for_dimension(dimension).admitted_units(),
        blocked.for_dimension(dimension).admitted_units()
    );
    let before_cleanup = serving.media_counters();
    gate.release();
    let no_effect = match handle.wait() {
        PhysicalCheckpointOutcome::ProvenNoEffect(no_effect) => no_effect,
        other => panic!("capture admission denial must reconcile the candidate: {other:?}"),
    };
    assert_eq!(no_effect.identity(), identity);
    let PhysicalCheckpointProvenNoEffectCause::FailedAndCandidateRemoved(
        PhysicalCheckpointCaptureFailureKind::ResidencyAdmission(
            PhysicalResidencyDenial::Pressure(pressure),
        ),
    ) = no_effect.cause()
    else {
        panic!(
            "capture must preserve the native pressure cause: {:?}",
            no_effect.cause()
        );
    };
    assert_eq!(pressure.store(), serving.store_identity());
    assert_eq!(pressure.pool().get(), blocked.pool_incarnation());
    assert_eq!(
        pressure.dimension(),
        PhysicalResidencyDimension::OperationBytes
    );
    assert_eq!(pressure.scope(), scope);
    assert_eq!(pressure.current(), limit - 1);
    assert_eq!(pressure.limit(), limit);
    assert!(pressure.requested() > 1);
    assert!(!pressure.effect_may_have_started());
    assert!(!candidate.exists());
    assert!(!store_root.join("families/checkpoint.current").exists());
    let after_cleanup = serving.media_counters();
    assert_eq!(
        after_cleanup.identified_operation_attempts_for(MediaOperationRole::Delete)
            - before_cleanup.identified_operation_attempts_for(MediaOperationRole::Delete),
        1
    );
    assert!(
        after_cleanup.completed_operations_for(MediaOperationRole::Delete)
            > before_cleanup.completed_operations_for(MediaOperationRole::Delete)
    );
    assert_eq!(
        after_cleanup.partial_effects_for(MediaOperationRole::Delete),
        before_cleanup.partial_effects_for(MediaOperationRole::Delete)
    );
    assert_eq!(
        after_cleanup.indeterminate_effects_for(MediaOperationRole::Delete),
        before_cleanup.indeterminate_effects_for(MediaOperationRole::Delete)
    );

    drop(blocker);
    let retry = match serving
        .checkpoints()
        .start(checkpoint_request(0x6e))
        .into_raw()
    {
        TransitionOutcome::Success(handle) => handle,
        TransitionOutcome::Failed(failure) => {
            panic!("checkpoint retry funding failed: {failure:?}")
        }
        _ => panic!("released pressure must permit a distinct checkpoint attempt"),
    };
    let published = match retry.wait() {
        PhysicalCheckpointOutcome::Completed(published) => published,
        other => panic!("checkpoint retry after native pressure failed: {other:?}"),
    };
    assert_eq!(published.dirty_records(), 1);
    let bytes = fs::read(store_root.join("families/checkpoint.current")).unwrap();
    assert_eq!(published.encoded_bytes(), bytes.len() as u64);
    assert_eq!(
        super::checkpoint_records(&bytes)
            .iter()
            .filter(|record| record[9] == 2)
            .count(),
        1
    );
    dirty.discard().unwrap();
    let shutdown = serving.close();
    assert_eq!(shutdown.checkpoint().proven_no_effect(), 1);
    assert_eq!(shutdown.checkpoint().completed(), 1);
    assert_eq!(
        shutdown.checkpoint().latest_publication(),
        Some(published.basis().identity())
    );
    assert!(!shutdown.checkpoint().requires_inspection());
}
