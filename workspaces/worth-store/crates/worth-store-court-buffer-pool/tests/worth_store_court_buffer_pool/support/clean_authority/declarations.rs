pub(crate) use {
    worth_store_buffer_pool::PhysicalFrameKey,
    worth_store_buffer_pool::PhysicalOperationAllocationScope,
    worth_store_buffer_pool::PhysicalResidencyDenial,
    worth_store_buffer_pool::PhysicalResidencyLimits,
    worth_store_buffer_pool::PhysicalResidencyPool,
    worth_store_buffer_pool::PhysicalResidencyPoolOwner,
    worth_store_buffer_pool::PhysicalSpeculativeWorkKind,
};
pub(crate) use {
    worth_store_physical_format::store_namespace::ProposedStoreIdentity,
    worth_store_physical_format::store_namespace::StableStoreIdentity,
    worth_store_physical_format::store_namespace::StoreNamespaceIdentityRecord,
    worth_store_physical_format::store_namespace::StoreNamespaceVersion,
    worth_store_physical_format::RecordArtifactFile,
    worth_store_physical_format::RecordFrameCoordinate,
};

pub(crate) fn owned_pool(
    identity_byte: u8,
) -> (
    PhysicalResidencyPool,
    worth_store_buffer_pool::CandidateFrameCleanAuthority,
    worth_store_buffer_pool::FrameWritebackCleanAuthority,
) {
    worth_store_buffer_pool::PhysicalResidencyPoolOwner::open(
        store(identity_byte),
        residency_limits(),
    )
    .unwrap()
    .into_parts()
}

pub(crate) fn foreground_write(
    pool: &PhysicalResidencyPool,
) -> worth_store_buffer_pool::ForegroundWriteAllocationGrant {
    pool.begin_foreground_write_operation(
        worth_store_buffer_pool::PhysicalResidencyPool::candidate_batch_operation_bytes(
            std::num::NonZeroUsize::MIN,
        )
        .unwrap(),
    )
    .unwrap()
}

pub(crate) fn writeback_claim(
    pool: &PhysicalResidencyPool,
    key: PhysicalFrameKey,
) -> Result<worth_store_buffer_pool::PhysicalWritebackClaim, PhysicalResidencyDenial> {
    let allocation = pool.begin_foreground_write_operation(
        std::num::NonZeroU64::new(u64::from(key.coordinate().length())).unwrap(),
    )?;
    pool.claim_writeback(allocation, &[key])
}

pub(crate) fn frame_key(store: StableStoreIdentity, ordinal: u64) -> PhysicalFrameKey {
    let artifact = worth_store_physical_format::RecordArtifactFile::RootManifest {
        generation: ordinal,
    };
    let coordinate =
        worth_store_physical_format::RecordFrameCoordinate::new(artifact, 0, 8).unwrap();
    worth_store_buffer_pool::PhysicalFrameKey::new(store, coordinate)
}

pub(crate) fn store(identity_byte: u8) -> StableStoreIdentity {
    worth_store_physical_format::store_namespace::StoreNamespaceIdentityRecord::new(
        worth_store_physical_format::store_namespace::StoreNamespaceVersion::CURRENT,
        worth_store_physical_format::store_namespace::ProposedStoreIdentity::from_nonzero_bytes(
            [identity_byte; 16],
        )
        .unwrap(),
    )
    .published_identity()
}

pub(crate) fn residency_limits() -> PhysicalResidencyLimits {
    let bytes = std::num::NonZeroU64::new(4096).unwrap();
    let candidate_operation =
        worth_store_buffer_pool::PhysicalResidencyPool::candidate_batch_operation_bytes(
            std::num::NonZeroUsize::MIN,
        )
        .unwrap();
    let operation = std::num::NonZeroU64::new(candidate_operation.get() + 8).unwrap();
    let frames = std::num::NonZeroU32::new(4).unwrap();
    let mut limits = worth_store_buffer_pool::PhysicalResidencyLimits::builder()
        .total_bytes(std::num::NonZeroU64::new(12_288 + operation.get()).unwrap())
        .resident_bytes(bytes)
        .metadata_bytes(bytes)
        .frame_entries(frames)
        .pinned_frames(frames)
        .pin_leases(frames)
        .dirty_frames(frames)
        .dirty_replacement_bytes(bytes)
        .operation_bytes(operation);
    for scope in [
        worth_store_buffer_pool::PhysicalOperationAllocationScope::ForegroundRead,
        worth_store_buffer_pool::PhysicalOperationAllocationScope::ForegroundWrite,
        worth_store_buffer_pool::PhysicalOperationAllocationScope::Recovery,
        worth_store_buffer_pool::PhysicalOperationAllocationScope::Scrub,
        worth_store_buffer_pool::PhysicalOperationAllocationScope::Maintenance,
        worth_store_buffer_pool::PhysicalOperationAllocationScope::Verification,
        worth_store_buffer_pool::PhysicalOperationAllocationScope::Blob,
    ] {
        limits = limits.scope_bytes(scope, operation);
    }
    for kind in [
        worth_store_buffer_pool::PhysicalSpeculativeWorkKind::Prefetch,
        worth_store_buffer_pool::PhysicalSpeculativeWorkKind::ReadAhead,
        worth_store_buffer_pool::PhysicalSpeculativeWorkKind::WriteBehind,
    ] {
        limits = limits.speculative_frames(kind, frames);
    }
    limits.admit(std::num::NonZeroU64::MIN).unwrap()
}
