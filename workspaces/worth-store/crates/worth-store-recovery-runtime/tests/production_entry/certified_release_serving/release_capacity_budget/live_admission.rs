//! Real competing Recovery custody must stop the first release before effects.

use super::*;
use worth_store::physical_runtime::{
    BlobReadLimits, PhysicalResidencyDimension, PhysicalResidencyRetryPosture,
    ServingPhysicalRuntime,
};

#[test]
fn competing_recovery_owner_denies_before_effects_then_partial_drop_reopens() {
    std::thread::Builder::new()
        .name("release-live-recovery-admission".to_owned())
        .stack_size(16 << 20)
        .spawn(run)
        .expect("live release admission worker")
        .join()
        .expect("live release admission worker did not panic");
}

fn run() {
    // One head closure for the release certificate, plus the drop's capture
    // custody (its envelope carries the policy's pin-scan bound, about 2.3 MB
    // here) and headroom.
    let limit = numeric_one_head_closure() + (4 << 20);
    let (world, proof, publication_record) =
        release_reopen::published_world::create(false, None, NonZeroU64::new(limit));
    let serving = world.serving();
    let source = (proof.object(), proof.generation());
    // The same fixture issuer may retry a proven-no-effect attempt. The
    // publication coordinates and evidence come from its actual issued proof.
    let retry_proof = AdmittedBlobReleaseProof::certification_admit(
        proof.store(),
        proof.object(),
        proof.generation(),
        proof.publication_allocation_epoch(),
        proof.publication_record_ordinal(),
        proof.publication_frame_sha256(),
        proof.issuer_evidence_sha256(),
    )
    .unwrap();
    assert_source_readable(serving, source);
    let selected = serving
        .certification_selected_latest_blob_publication()
        .unwrap()
        .unwrap();
    let root_before = root_generation(serving);
    let handle = serving
        .blobs()
        .unwrap()
        .reclaim(partial_request(proof, world.placement()))
        .expect("real Blob allocation and reclaim fence precede collision");
    let policy = serving.residency_observation().admitted_policy();
    assert_eq!(policy.operation_bytes(), 32 << 20);
    assert_eq!(
        policy.scope_bytes(PhysicalOperationAllocationScope::Recovery),
        limit
    );
    let active = serving.residency_observation().counters();
    let recovery_before =
        active.active_operation_bytes_for(PhysicalOperationAllocationScope::Recovery);
    let held_bytes = limit - recovery_before - 1;
    assert!(
        policy.operation_bytes() - active.active_operation_bytes() - held_bytes > limit,
        "collision must leave ample global operation headroom"
    );
    let held = serving
        .certification_physical_residency()
        .admit_operation_scope(
            PhysicalOperationAllocationScope::Recovery,
            NonZeroU64::new(held_bytes).unwrap(),
        )
        .expect("real competing Recovery grant");
    assert_eq!(recovery_bytes(serving), limit - 1);
    let before = snapshot_family(world.root());
    let counters = serving.media_counters();
    let result = handle.wait();
    let allocation = match result {
        Err(BlobReclaimFailure::ReleaseCertificateBacking(
            PhysicalRecoveryRejoinResidentDenial::OperationAllocation(allocation),
        )) => allocation,
        other => panic!("live Recovery custody must deny at release admission: {other:?}"),
    };
    let pressure = allocation
        .pressure()
        .expect("native Recovery scope pressure");
    assert_eq!(pressure.basis().store_identity(), serving.store_identity());
    assert_eq!(
        pressure.store_generation(),
        serving.residency_observation().store_generation()
    );
    assert_eq!(pressure.scope(), PhysicalOperationAllocationScope::Recovery);
    assert_eq!(
        pressure.dimension(),
        PhysicalResidencyDimension::OperationScope(PhysicalOperationAllocationScope::Recovery)
    );
    assert!(
        pressure.requested() >= numeric_one_head_closure() - recovery_before,
        "denial must fund the known first-head closure, not an unrelated small allocation"
    );
    assert_eq!(pressure.admitted(), limit - 1);
    assert_eq!(pressure.limit(), limit);
    assert_eq!(
        pressure.retry_posture(),
        PhysicalResidencyRetryPosture::AfterAllocationRelease
    );
    assert!(!pressure.effect_may_have_started());
    assert_eq!(snapshot_family(world.root()), before);
    assert_eq!(root_generation(serving), root_before);
    let after = serving.media_counters();
    assert_eq!(after.append_attempts(), counters.append_attempts());
    assert_eq!(
        after.positioned_write_attempts(),
        counters.positioned_write_attempts()
    );
    assert_eq!(after.replacements(), counters.replacements());
    assert_eq!(after.file_creates(), counters.file_creates());
    assert_eq!(
        serving
            .certification_selected_latest_blob_publication()
            .unwrap()
            .unwrap()
            .record(),
        selected.record()
    );
    assert_eq!(selected.record(), publication_record);
    assert_eq!(
        serving
            .residency_observation()
            .counters()
            .active_operation_bytes_for(PhysicalOperationAllocationScope::Blob),
        0
    );
    assert_eq!(recovery_bytes(serving), limit - 1);
    drop(held);
    assert_eq!(recovery_bytes(serving), recovery_before);
    assert_source_readable(serving, source);

    let receipt = serving
        .blobs()
        .unwrap()
        .reclaim(partial_request(retry_proof, world.placement()))
        .expect("released collision permits the same lawful request")
        .wait()
        .expect("real partial drop completes");
    assert_eq!(receipt.disposition(), BlobReclaimDisposition::Dropped);
    assert_eq!(receipt.dropped_records().len(), 1);
    assert!(receipt.remaining_payload_records() > 0);
    assert!(
        recovery_bytes(serving) > recovery_before,
        "completed drop retains its live ledger capacity"
    );
    let request = PhysicalCheckpointRequest::fuzzy(
        PhysicalCheckpointIdempotencyKey::new([0x7b; 32]),
        PhysicalCheckpointDeadline::after_milliseconds(30_000).unwrap(),
    );
    let TransitionOutcome::Success(handle) = serving.checkpoints().start(request).into_raw() else {
        panic!("partial release checkpoint must admit");
    };
    assert!(matches!(
        handle.wait(),
        PhysicalCheckpointOutcome::Completed(_)
    ));
    // Reclaim's covering checkpoint already folded the Batch. Replacing that
    // snapshot with a zero-Batch carryforward disposes its obsolete backing;
    // total Recovery usage is not the retained ledger's own capacity census.
    assert!(
        recovery_bytes(serving) > recovery_before,
        "checkpoint retains live funding for surviving release custody"
    );
    let retained = world.retained_root();
    let root = retained.path().to_path_buf();
    drop(world);
    // Real C8 selection, independent Store rejoin, Serving, and another
    // checkpoint. No live ledger or grant is handed to the recovery engine.
    super::super::recover(root, super::super::Mutation::None);
}

fn partial_request(
    proof: AdmittedBlobReleaseProof,
    placement: worth_store::physical_runtime::AdmittedRecordPlacementPolicy,
) -> BlobReclaimRequest<'static> {
    BlobReclaimRequest::released(
        proof,
        placement,
        PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
        BlobReclaimLimits::new(
            NonZeroU64::new(1024).unwrap(),
            NonZeroU64::new(64 << 20).unwrap(),
            NonZeroU16::new(1).unwrap(),
        )
        .unwrap(),
    )
}

fn assert_source_readable(serving: &ServingPhysicalRuntime, source: ([u8; 16], u64)) {
    let scope = admitted_blob_scope("c11.recovery.release.scope");
    let blobs = serving.blobs().unwrap();
    let limits = BlobReadLimits::new(NonZeroU64::new(256).unwrap());
    let published = blobs
        .resolve_publication(source.0, source.1, &scope, limits)
        .expect("genuine selected publication is ordinarily resolvable");
    let mut read = blobs.read(published, &scope, 0, 128 << 10, limits).unwrap();
    let mut bytes = vec![0; 128 << 10];
    let mut used = 0;
    loop {
        let count = read.read_next(&mut bytes[used..]).unwrap();
        if count == 0 {
            break;
        }
        used += count;
    }
    assert_eq!(used, bytes.len());
    assert!(bytes[..64 << 10].iter().all(|byte| *byte == 0x31));
    assert!(bytes[64 << 10..].iter().all(|byte| *byte == 0x52));
}

fn recovery_bytes(serving: &ServingPhysicalRuntime) -> u64 {
    serving
        .residency_observation()
        .counters()
        .active_operation_bytes_for(PhysicalOperationAllocationScope::Recovery)
}

fn root_generation(serving: &ServingPhysicalRuntime) -> u64 {
    serving
        .records()
        .unwrap()
        .protected_root()
        .root()
        .generation()
        .get()
}
