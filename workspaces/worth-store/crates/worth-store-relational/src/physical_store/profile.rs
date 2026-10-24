//! The one physical Store profile a Relational home is created and recovered
//! under. Initialization and every later recovery must agree on it, so it is
//! declared once here rather than by callers.

use std::num::{NonZeroU32, NonZeroU64};

use worth_store::physical_runtime::{
    AdmittedPhysicalRecordFormat, AdmittedRecordPlacementPolicy, CheckpointMemoryLimit,
    ConfiguredPhysicalDurabilityDeclaration, GroupCommitDelay, GroupCommitLimit,
    IdempotencyRetentionGenerations, LiveIdempotencyBindingLimit, ManifestEntryCapacity,
    PendingUnresolvedMutationLimit, PhysicalCheckpointPolicy, PhysicalDurabilityDeclaration,
    PhysicalIdempotencyPolicy, PhysicalRecordAccessPolicy, PhysicalRecordFormatDeclaration,
    PhysicalRecordPlacementPolicy, PhysicalWalPolicy, RetainedWalTailLimit, WalSegmentByteLimit,
    WalSegmentInventoryLimit,
};
use worth_store_recovery_runtime::{
    PhysicalRecoveryLimitDeclaration, PhysicalRecoveryStaticConfiguration,
};

/// Bytes one WAL segment may hold before the WAL rolls to the next.
const WAL_SEGMENT_BYTES: u64 = 64 << 20;
/// Bytes recovery may admit along each byte-bounded dimension.
const RECOVERY_BYTES: u64 = 512 << 20;

pub(crate) fn record_format() -> AdmittedPhysicalRecordFormat {
    AdmittedPhysicalRecordFormat::admit(
        PhysicalRecordFormatDeclaration::builder()
            .admit()
            .expect("the default record format admits"),
    )
}

pub(crate) fn placement() -> AdmittedRecordPlacementPolicy {
    PhysicalRecordPlacementPolicy::builder()
        .manifest_capacity(ManifestEntryCapacity::new(64).expect("nonzero manifest capacity"))
        .admit(record_format())
        .expect("the home placement admits under the default format")
}

pub(crate) fn access() -> worth_store::physical_runtime::AdmittedRecordAccessPolicy {
    PhysicalRecordAccessPolicy::builder()
        .admit(record_format())
        .expect("the default access policy admits")
}

pub(crate) fn durability() -> ConfiguredPhysicalDurabilityDeclaration {
    PhysicalDurabilityDeclaration::builder()
        .group_commit(
            GroupCommitLimit::new(NonZeroU32::new(32).expect("nonzero")),
            GroupCommitDelay::new(NonZeroU64::new(1).expect("nonzero")),
        )
        .wal(PhysicalWalPolicy::segmented(
            WalSegmentByteLimit::new(NonZeroU64::new(WAL_SEGMENT_BYTES).expect("nonzero")),
            WalSegmentInventoryLimit::new(NonZeroU32::new(1024).expect("nonzero")),
        ))
        .idempotency(PhysicalIdempotencyPolicy::new(
            IdempotencyRetentionGenerations::new(NonZeroU64::new(4).expect("nonzero")),
            PendingUnresolvedMutationLimit::new(NonZeroU32::new(1024).expect("nonzero")),
            LiveIdempotencyBindingLimit::new(NonZeroU32::new(4096).expect("nonzero")),
        ))
        .checkpoint(PhysicalCheckpointPolicy::fuzzy(
            CheckpointMemoryLimit::new(NonZeroU64::new(64 << 20).expect("nonzero")),
            RetainedWalTailLimit::new(NonZeroU64::new(256 << 20).expect("nonzero")),
        ))
}

pub(crate) fn recovery_configuration() -> PhysicalRecoveryStaticConfiguration {
    PhysicalRecoveryStaticConfiguration::current().with_durability_declaration(durability())
}

pub(crate) fn recovery_limits() -> PhysicalRecoveryLimitDeclaration {
    PhysicalRecoveryLimitDeclaration {
        selector_candidates: 4,
        checkpoint_candidates: 64,
        manifest_bytes: RECOVERY_BYTES,
        manifest_entries: 1 << 20,
        wal_segments: 1024,
        wal_frames: 1 << 20,
        wal_bytes: RECOVERY_BYTES,
        redo_targets: 1 << 20,
        redo_bytes: RECOVERY_BYTES,
        distinct_pages_and_extents: 1 << 20,
        operation_bindings: 1 << 20,
        staging_bytes: RECOVERY_BYTES,
        recovery_memory_bytes: RECOVERY_BYTES,
        dirty_frames: 1 << 16,
        concurrent_commands: 8,
        publication_effects: 4096,
        cleanup_candidates: 1 << 16,
        cleanup_bytes: RECOVERY_BYTES,
        observation_bytes: RECOVERY_BYTES,
    }
}
