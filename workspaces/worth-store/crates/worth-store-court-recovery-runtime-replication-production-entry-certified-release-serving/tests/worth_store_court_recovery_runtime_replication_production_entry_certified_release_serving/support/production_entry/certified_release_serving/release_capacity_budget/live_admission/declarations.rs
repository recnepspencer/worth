//! Real competing Recovery custody must stop the first release before effects.

pub(crate) use {worth_proof::AdmittedBlobReleaseProof, worth_store::physical_runtime::BlobReclaimDisposition, worth_store::physical_runtime::BlobReclaimFailure, worth_store::physical_runtime::BlobReclaimLimits, worth_store::physical_runtime::BlobReclaimRequest, worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::Mutation, std::num::NonZeroU16, std::num::NonZeroU64, worth_store::physical_runtime::PhysicalCheckpointDeadline, worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey, worth_store::physical_runtime::PhysicalCheckpointOutcome, worth_store::physical_runtime::PhysicalCheckpointRequest, worth_store::physical_runtime::PhysicalMutationDeadline, worth_store::physical_runtime::PhysicalOperationAllocationScope, worth_store::physical_runtime::PhysicalRecoveryRejoinResidentDenial, worth_proof::TransitionOutcome, worth_store_testkit_recovery_runtime::production_entry::blob_scope_admission::admitted_blob_scope, crate::production_entry::certified_release_serving::release_capacity_budget::funded_one_head_limit, crate::production_entry::certified_release_serving::release_capacity_budget::numeric_one_head_closure, worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery::recover, worth_store_testkit_recovery_runtime::production_entry::release_reopen, worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery_request::request, crate::production_entry::certified_release_serving::release_capacity_budget::snapshot_family};
pub(crate) use {worth_store::physical_runtime::BlobReadLimits, worth_store::physical_runtime::PhysicalResidencyDimension, worth_store::physical_runtime::PhysicalResidencyRetryPosture, worth_store::physical_runtime::ServingPhysicalRuntime};

pub(crate) fn run() {
    let limit = funded_one_head_limit();
    let (world, proof, publication_record) =
        worth_store_testkit_recovery_runtime::production_entry::release_reopen::published_world::create(false, None, std::num::NonZeroU64::new(limit));
    let serving = world.serving();
    let source = (proof.object(), proof.generation());
    // The same fixture issuer may retry a proven-no-effect attempt. The
    // publication coordinates and evidence come from its actual issued proof.
    let retry_proof = worth_proof::AdmittedBlobReleaseProof::certification_admit(
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
        policy.scope_bytes(worth_store::physical_runtime::PhysicalOperationAllocationScope::Recovery),
        limit
    );
    let active = serving.residency_observation().counters();
    let recovery_before =
        active.active_operation_bytes_for(worth_store::physical_runtime::PhysicalOperationAllocationScope::Recovery);
    let held_bytes = limit - recovery_before - 1;
    assert!(
        policy.operation_bytes() - active.active_operation_bytes() - held_bytes > limit,
        "collision must leave ample global operation headroom"
    );
    let held = serving
        .certification_physical_residency()
        .admit_operation_scope(
            worth_store::physical_runtime::PhysicalOperationAllocationScope::Recovery,
            std::num::NonZeroU64::new(held_bytes).unwrap(),
        )
        .expect("real competing Recovery grant");
    assert_eq!(recovery_bytes(serving), limit - 1);
    let before = snapshot_family(world.root());
    let counters = serving.media_counters();
    let result = handle.wait();
    let allocation = match result {
        Err(worth_store::physical_runtime::BlobReclaimFailure::ReleaseCertificateBacking(
            worth_store::physical_runtime::PhysicalRecoveryRejoinResidentDenial::OperationAllocation(allocation),
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
    assert_eq!(pressure.scope(), worth_store::physical_runtime::PhysicalOperationAllocationScope::Recovery);
    assert_eq!(
        pressure.dimension(),
        worth_store::physical_runtime::PhysicalResidencyDimension::OperationScope(worth_store::physical_runtime::PhysicalOperationAllocationScope::Recovery)
    );
    assert!(
        pressure.requested() >= numeric_one_head_closure() - recovery_before,
        "denial must fund the known first-head closure, not an unrelated small allocation"
    );
    assert_eq!(pressure.admitted(), limit - 1);
    assert_eq!(pressure.limit(), limit);
    assert_eq!(
        pressure.retry_posture(),
        worth_store::physical_runtime::PhysicalResidencyRetryPosture::AfterAllocationRelease
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
            .active_operation_bytes_for(worth_store::physical_runtime::PhysicalOperationAllocationScope::Blob),
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
    assert_eq!(receipt.disposition(), worth_store::physical_runtime::BlobReclaimDisposition::Dropped);
    assert_eq!(receipt.dropped_records().len(), 1);
    assert!(receipt.remaining_payload_records() > 0);
    assert!(
        recovery_bytes(serving) > recovery_before,
        "completed drop retains its live ledger capacity"
    );
    let request = worth_store::physical_runtime::PhysicalCheckpointRequest::fuzzy(
        worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey::new([0x7b; 32]),
        worth_store::physical_runtime::PhysicalCheckpointDeadline::after_milliseconds(30_000).unwrap(),
    );
    let worth_proof::TransitionOutcome::Success(handle) = serving.checkpoints().start(request).into_raw() else {
        panic!("partial release checkpoint must admit");
    };
    assert!(matches!(
        handle.wait(),
        worth_store::physical_runtime::PhysicalCheckpointOutcome::Completed(_)
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
    worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery::recover(root, worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::Mutation::None);
}

pub(crate) fn partial_request(
    proof: AdmittedBlobReleaseProof,
    placement: worth_store::physical_runtime::AdmittedRecordPlacementPolicy,
) -> BlobReclaimRequest<'static> {
    worth_store::physical_runtime::BlobReclaimRequest::released(
        proof,
        placement,
        worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
        worth_store::physical_runtime::BlobReclaimLimits::new(
            std::num::NonZeroU64::new(1024).unwrap(),
            std::num::NonZeroU64::new(64 << 20).unwrap(),
            std::num::NonZeroU16::new(1).unwrap(),
        )
        .unwrap(),
    )
}

pub(crate) fn assert_source_readable(serving: &ServingPhysicalRuntime, source: ([u8; 16], u64)) {
    let scope = admitted_blob_scope("c11.recovery.release.scope");
    let blobs = serving.blobs().unwrap();
    let limits = worth_store::physical_runtime::BlobReadLimits::new(std::num::NonZeroU64::new(256).unwrap());
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

pub(crate) fn recovery_bytes(serving: &ServingPhysicalRuntime) -> u64 {
    serving
        .residency_observation()
        .counters()
        .active_operation_bytes_for(worth_store::physical_runtime::PhysicalOperationAllocationScope::Recovery)
}

pub(crate) fn root_generation(serving: &ServingPhysicalRuntime) -> u64 {
    serving
        .records()
        .unwrap()
        .protected_root()
        .root()
        .generation()
        .get()
}