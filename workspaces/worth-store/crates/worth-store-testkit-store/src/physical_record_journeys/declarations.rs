pub use {std::num::NonZeroU32, std::num::NonZeroU64, std::path::Path};



















































pub use {crate::physical_record_journeys::child_process::invocation::run_child};
pub use {worth_proof::TransitionOutcome};
pub use {worth_store::physical_runtime::AdmittedPhysicalDurabilityPolicy, worth_store::physical_runtime::AdmittedPhysicalRecordFormat, worth_store::physical_runtime::AdmittedRecordAccessPolicy, worth_store::physical_runtime::AdmittedRecordPlacementPolicy, worth_store::physical_runtime::CheckpointMemoryLimit, worth_store::physical_runtime::FilesystemMediaAdmission, worth_store::physical_runtime::GroupCommitDelay, worth_store::physical_runtime::GroupCommitLimit, worth_store::physical_runtime::IdempotencyRetentionGenerations, worth_store::physical_runtime::LiveIdempotencyBindingLimit, worth_store::physical_runtime::ManifestEntryCapacity, worth_store::physical_runtime::MediaOwnedPhysicalRuntime, worth_store::physical_runtime::PendingUnresolvedMutationLimit, worth_store::physical_runtime::PhysicalCheckpointPolicy, worth_store::physical_runtime::PhysicalDurabilityDeclaration, worth_store::physical_runtime::PhysicalIdempotencyPolicy, worth_store::physical_runtime::PhysicalRecordAccessPolicy, worth_store::physical_runtime::PhysicalRecordFormatDeclaration, worth_store::physical_runtime::PhysicalRecordInitialization, worth_store::physical_runtime::PhysicalRecordOpen, worth_store::physical_runtime::PhysicalRecordPlacementPolicy, worth_store::physical_runtime::PhysicalRuntimeAdmission, worth_store::physical_runtime::PhysicalStore, worth_store::physical_runtime::PhysicalWalPolicy, worth_store::physical_runtime::RecordBootstrapDenial, worth_store::physical_runtime::RecordReadObservation, worth_store::physical_runtime::RecordReadSession, worth_store::physical_runtime::RecordServingAdmissionOutcome, worth_store::physical_runtime::RecordStoreInitializationDenial, worth_store::physical_runtime::RecordStoreOpenDenial, worth_store::physical_runtime::RetainedWalTailLimit, worth_store::physical_runtime::ServingPhysicalRuntime, worth_store::physical_runtime::WalSegmentByteLimit, worth_store::physical_runtime::WalSegmentInventoryLimit};
pub use {worth_store_physical_backend::FilesystemAccessPosture};

pub fn configuration() -> (
    AdmittedPhysicalRecordFormat,
    AdmittedRecordPlacementPolicy,
    AdmittedRecordAccessPolicy,
) {
    let format = worth_store::physical_runtime::AdmittedPhysicalRecordFormat::admit(
        worth_store::physical_runtime::PhysicalRecordFormatDeclaration::builder().admit().unwrap(),
    );
    let placement = worth_store::physical_runtime::PhysicalRecordPlacementPolicy::builder()
        .manifest_capacity(worth_store::physical_runtime::ManifestEntryCapacity::new(64).unwrap())
        .admit(format)
        .unwrap();
    let access = worth_store::physical_runtime::PhysicalRecordAccessPolicy::builder().admit(format).unwrap();
    (format, placement, access)
}

pub fn media(root: &Path) -> MediaOwnedPhysicalRuntime {
    let runtime = worth_store::physical_runtime::PhysicalStore::admit(worth_store::physical_runtime::PhysicalRuntimeAdmission::new(root).unwrap()).unwrap();
    match runtime
        .try_admit_filesystem_media(worth_store::physical_runtime::FilesystemMediaAdmission::production(
            worth_store_physical_backend::FilesystemAccessPosture::CoordinatedServiceAccount,
        ))
        .into_raw()
    {
        worth_proof::TransitionOutcome::Success(media) => media,
        _ => panic!("real filesystem media admission must succeed"),
    }
}

pub fn durability(media: &MediaOwnedPhysicalRuntime) -> AdmittedPhysicalDurabilityPolicy {
    durability_with_group_limit(media, std::num::NonZeroU32::new(32).unwrap())
}

pub fn durability_with_group_limit(
    media: &MediaOwnedPhysicalRuntime,
    group_limit: NonZeroU32,
) -> AdmittedPhysicalDurabilityPolicy {
    durability_with_pending_limit(media, group_limit, std::num::NonZeroU32::new(1_024).unwrap())
}

pub fn durability_with_pending_limit(
    media: &MediaOwnedPhysicalRuntime,
    group_limit: NonZeroU32,
    pending_limit: NonZeroU32,
) -> AdmittedPhysicalDurabilityPolicy {
    durability_with_limits(
        media,
        group_limit,
        pending_limit,
        worth_store::physical_runtime::PhysicalWalPolicy::segmented(
            worth_store::physical_runtime::WalSegmentByteLimit::new(std::num::NonZeroU64::new(16 * 1024 * 1024).unwrap()),
            worth_store::physical_runtime::WalSegmentInventoryLimit::new(std::num::NonZeroU32::new(1_024).unwrap()),
        ),
    )
}

pub fn durability_with_wal_policy(
    media: &MediaOwnedPhysicalRuntime,
    wal: PhysicalWalPolicy,
) -> AdmittedPhysicalDurabilityPolicy {
    durability_with_limits(
        media,
        std::num::NonZeroU32::new(32).unwrap(),
        std::num::NonZeroU32::new(1_024).unwrap(),
        wal,
    )
}

pub fn durability_with_limits(
    media: &MediaOwnedPhysicalRuntime,
    group_limit: NonZeroU32,
    pending_limit: NonZeroU32,
    wal: PhysicalWalPolicy,
) -> AdmittedPhysicalDurabilityPolicy {
    durability_with_idempotency_and_wal_limits(
        media,
        group_limit,
        pending_limit,
        std::num::NonZeroU32::new(4_096).unwrap(),
        wal,
    )
}

pub fn durability_with_idempotency_limits(
    media: &MediaOwnedPhysicalRuntime,
    pending_limit: NonZeroU32,
    live_limit: NonZeroU32,
) -> AdmittedPhysicalDurabilityPolicy {
    durability_with_idempotency_and_wal_limits(
        media,
        std::num::NonZeroU32::new(32).unwrap(),
        pending_limit,
        live_limit,
        worth_store::physical_runtime::PhysicalWalPolicy::segmented(
            worth_store::physical_runtime::WalSegmentByteLimit::new(std::num::NonZeroU64::new(16 * 1024 * 1024).unwrap()),
            worth_store::physical_runtime::WalSegmentInventoryLimit::new(std::num::NonZeroU32::new(1_024).unwrap()),
        ),
    )
}

pub fn durability_with_idempotency_and_wal_limits(
    media: &MediaOwnedPhysicalRuntime,
    group_limit: NonZeroU32,
    pending_limit: NonZeroU32,
    live_limit: NonZeroU32,
    wal: PhysicalWalPolicy,
) -> AdmittedPhysicalDurabilityPolicy {
    let basis = media.physical_durability_admission_basis().unwrap();
    match worth_store::physical_runtime::PhysicalDurabilityDeclaration::builder()
        .group_commit(
            worth_store::physical_runtime::GroupCommitLimit::new(group_limit),
            worth_store::physical_runtime::GroupCommitDelay::new(std::num::NonZeroU64::new(1).unwrap()),
        )
        .wal(wal)
        .idempotency(worth_store::physical_runtime::PhysicalIdempotencyPolicy::new(
            worth_store::physical_runtime::IdempotencyRetentionGenerations::new(std::num::NonZeroU64::new(4).unwrap()),
            worth_store::physical_runtime::PendingUnresolvedMutationLimit::new(pending_limit),
            worth_store::physical_runtime::LiveIdempotencyBindingLimit::new(live_limit),
        ))
        .checkpoint(worth_store::physical_runtime::PhysicalCheckpointPolicy::fuzzy(
            worth_store::physical_runtime::CheckpointMemoryLimit::new(std::num::NonZeroU64::new(16 * 1024 * 1024).unwrap()),
            worth_store::physical_runtime::RetainedWalTailLimit::new(std::num::NonZeroU64::new(64 * 1024 * 1024).unwrap()),
        ))
        .admit(basis)
        .into_raw()
    {
        worth_proof::TransitionOutcome::Success(policy) => policy,
        _ => panic!("qualified test media must admit the explicit durability policy"),
    }
}

pub fn serving_from_initialization(root: &Path) -> ServingPhysicalRuntime {
    let (format, placement, access) = configuration();
    let media = media(root);
    let durability = durability(&media);
    success(
        media.initialize_record_store(worth_store::physical_runtime::PhysicalRecordInitialization::new(
            format, placement, access, durability,
        )),
    )
}

pub fn serving_from_open(root: &Path) -> ServingPhysicalRuntime {
    let (format, _, access) = configuration();
    let media = media(root);
    let durability = durability(&media);
    success(media.open_record_store(worth_store::physical_runtime::PhysicalRecordOpen::new(format, access, durability)))
}

pub trait BootstrapDenialReason {
    fn reason(&self) -> RecordBootstrapDenial;
}

impl BootstrapDenialReason for RecordStoreInitializationDenial {
    fn reason(&self) -> RecordBootstrapDenial {
        self.reason()
    }
}

impl BootstrapDenialReason for RecordStoreOpenDenial {
    fn reason(&self) -> RecordBootstrapDenial {
        self.reason()
    }
}

pub fn success<Denial>(outcome: RecordServingAdmissionOutcome<Denial>) -> ServingPhysicalRuntime
where
    Denial: BootstrapDenialReason,
{
    match outcome.into_raw() {
        worth_proof::TransitionOutcome::Success(serving) => serving,
        worth_proof::TransitionOutcome::Denied(denial) => {
            panic!(
                "record-serving progression must succeed; denied by {:?}",
                denial.reason()
            )
        }
        worth_proof::TransitionOutcome::Deferred(deferred) => match deferred {},
        worth_proof::TransitionOutcome::Stale(stale) => {
            panic!(
                "record-serving progression must succeed; stale because {:?}",
                stale.reason()
            )
        }
        worth_proof::TransitionOutcome::RebindRequired(rebind) => {
            panic!(
                "record-serving progression must succeed; rebind required because {:?}",
                rebind.reason()
            )
        }
        worth_proof::TransitionOutcome::Failed(failure) => {
            panic!(
                "record-serving progression must succeed; inspection required because {:?}",
                failure.cause()
            )
        }
    }
}

pub fn read_record(
    mut session: RecordReadSession,
    expected_bytes: usize,
) -> (Vec<u8>, RecordReadObservation) {
    let mut bytes = vec![0_u8; expected_bytes];
    let mut completed = 0;
    while completed < bytes.len() {
        let count = session.read_next(&mut bytes[completed..]).unwrap();
        assert!(count > 0, "the record ended before its declared length");
        completed += count;
    }
    assert_eq!(session.read_next(&mut [0_u8; 1]).unwrap(), 0);
    (bytes, session.observation())
}