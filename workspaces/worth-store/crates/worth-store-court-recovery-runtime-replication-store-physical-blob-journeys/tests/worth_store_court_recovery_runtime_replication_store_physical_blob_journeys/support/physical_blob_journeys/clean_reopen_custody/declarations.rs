//! Ordinary reopen's own checkpoint custody on tier-anchored roots, and the
//! retained released-drop WAL member that still requires C.8.

pub(crate) use {std::fs, std::path::Path, std::time::Duration};

pub(crate) use worth_proof::TransitionOutcome;
pub(crate) use {
    worth_store::physical_runtime::PhysicalCheckpointCaptureFailureKind,
    worth_store::physical_runtime::PhysicalCheckpointDeadline,
    worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey,
    worth_store::physical_runtime::PhysicalCheckpointOutcome,
    worth_store::physical_runtime::PhysicalCheckpointRequest,
    worth_store::physical_runtime::PhysicalCheckpointStartFailure,
    worth_store::physical_runtime::ServingPhysicalRuntime,
};

pub(crate) use {
    crate::physical_blob_journeys::blob_crash::establish_recovery_frontier,
    crate::physical_blob_journeys::blob_crash::kill_at,
    crate::physical_blob_journeys::blob_reclaim_released_crash::TIER_ROLE,
    worth_store_testkit_store::fixture::assert_anchored_open_requires_recovered_custody,
    worth_store_testkit_store::fixture::assert_released_open_requires_recovered_custody,
    worth_store_testkit_store::fixture::configuration,
    worth_store_testkit_store::fixture::placement,
    worth_store_testkit_store::fixture::serving_from_initialization,
    worth_store_testkit_store::fixture::serving_from_open,
};

pub(crate) const TIER_CERTIFICATE_KIND: u8 = 6;
pub(crate) const RELEASE_CERTIFICATE_KIND: u8 = 7;

/// Activates the tier epoch and certifies it in a selected checkpoint.
pub(crate) fn certify_tier_epoch(serving: &ServingPhysicalRuntime) {
    serving
        .certification_activate_tier_epoch(placement())
        .unwrap();
    checkpoint(serving, 0x64);
}

pub(crate) fn checkpoint(serving: &ServingPhysicalRuntime, key: u8) {
    let request = worth_store::physical_runtime::PhysicalCheckpointRequest::fuzzy(
        worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey::new([key; 32]),
        worth_store::physical_runtime::PhysicalCheckpointDeadline::after_milliseconds(30_000)
            .unwrap(),
    );
    let worth_proof::TransitionOutcome::Success(handle) =
        serving.checkpoints().start(request).into_raw()
    else {
        panic!("checkpoint must admit")
    };
    assert!(matches!(
        handle.wait(),
        worth_store::physical_runtime::PhysicalCheckpointOutcome::Completed(_)
    ));
}

/// Kinds of the selected checkpoint's certificate records, in stream order.
pub(crate) fn certificate_kinds(root: &Path) -> Vec<u8> {
    let bytes = std::fs::read(root.join("families/checkpoint.current")).unwrap();
    let mut kinds = Vec::new();
    let mut offset = 0;
    while offset < bytes.len() {
        let payload = u32::from_le_bytes(bytes[offset + 12..offset + 16].try_into().unwrap());
        let kind = bytes[offset + 9];
        if matches!(kind, TIER_CERTIFICATE_KIND | RELEASE_CERTIFICATE_KIND) {
            kinds.push(kind);
        }
        offset += 16 + payload as usize + 4;
    }
    kinds
}
