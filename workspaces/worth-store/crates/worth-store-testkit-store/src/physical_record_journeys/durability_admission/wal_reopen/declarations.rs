pub use {std::num::NonZeroU32, std::num::NonZeroU64};
pub use {std::path::Path};

pub use {sha2::Digest, sha2::Sha256};
pub use {worth_proof::NonEmpty, worth_proof::TransitionOutcome};
pub use {worth_store::physical_runtime::PhysicalDurabilityStateReopenFailure, worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial, worth_store::physical_runtime::PhysicalRecordInitialization, worth_store::physical_runtime::PhysicalRecordOpen, worth_store::physical_runtime::PhysicalSignalConstructionFailure, worth_store::physical_runtime::PhysicalWalGroupAppendOutcome, worth_store::physical_runtime::PhysicalWalOpenFailure, worth_store::physical_runtime::PhysicalWalPolicy, worth_store::physical_runtime::RecordBootstrapFailure, worth_store::physical_runtime::WalSegmentByteLimit, worth_store::physical_runtime::WalSegmentInventoryLimit};
pub use {worth_store_physical_backend::MediaOperationRole};
pub use {worth_store_wal::WalArtifactStoreDenial, worth_store_wal::WalTopologyDenialKind};

pub use {crate::physical_record_journeys::configuration, crate::physical_record_journeys::durability_with_wal_policy, crate::physical_record_journeys::media, crate::physical_record_journeys::success};
pub use {crate::physical_record_journeys::durability_admission::wal_append::prepared};

// One two-member group fills the current projection segment exactly.
// Keep this aligned with the independent rotation inventory oracle.
pub const SEGMENT_BYTES: u64 = 35_304;

#[derive(Debug, Clone, Copy)]
pub enum HostileInventory {
    InventoryLimit,
    NonCanonicalName,
    EmptySegment,
    SegmentIdentityMismatch,
    TornFrame,
    PayloadDigestMismatch,
    SegmentGap,
    LsnGap,
    GenerationMismatch,
}

impl HostileInventory {
    pub fn policy(self) -> PhysicalWalPolicy {
        let inventory = if matches!(self, Self::InventoryLimit) {
            2
        } else {
            3
        };
        wal_policy(SEGMENT_BYTES, inventory)
    }

    pub fn expected(self) -> PhysicalWalOpenFailure {
        match self {
            Self::InventoryLimit => worth_store::physical_runtime::PhysicalWalOpenFailure::InventoryLimitExceeded,
            Self::NonCanonicalName => worth_store::physical_runtime::PhysicalWalOpenFailure::NonCanonicalArtifact,
            Self::EmptySegment => worth_store::physical_runtime::PhysicalWalOpenFailure::EmptySegment,
            Self::SegmentIdentityMismatch => worth_store::physical_runtime::PhysicalWalOpenFailure::SegmentInspection(
                worth_store_wal::WalArtifactStoreDenial::StoreBindingMismatch,
            ),
            Self::TornFrame => {
                worth_store::physical_runtime::PhysicalWalOpenFailure::SegmentInspection(worth_store_wal::WalArtifactStoreDenial::InvalidFrame)
            }
            Self::PayloadDigestMismatch => {
                worth_store::physical_runtime::PhysicalWalOpenFailure::SegmentInspection(worth_store_wal::WalArtifactStoreDenial::DigestMismatch)
            }
            Self::SegmentGap => {
                worth_store::physical_runtime::PhysicalWalOpenFailure::Topology(worth_store_wal::WalTopologyDenialKind::NonContiguousSegment)
            }
            Self::LsnGap => worth_store::physical_runtime::PhysicalWalOpenFailure::Topology(worth_store_wal::WalTopologyDenialKind::Gap),
            Self::GenerationMismatch => {
                worth_store::physical_runtime::PhysicalWalOpenFailure::Topology(worth_store_wal::WalTopologyDenialKind::WrongGeneration)
            }
        }
    }

    pub fn corrupt(self, store_root: &Path) {
        match self {
            Self::InventoryLimit => {}
            Self::NonCanonicalName => std::fs::rename(
                segment_path(store_root, 1, 1),
                wal_directory(store_root).join("segment-01-generation-1.wal"),
            )
            .unwrap(),
            Self::EmptySegment => std::fs::write(segment_path(store_root, 1, 1), []).unwrap(),
            Self::SegmentIdentityMismatch => {
                corrupt_first_frame_segment_identity(&segment_path(store_root, 1, 1), 9)
            }
            Self::TornFrame => {
                let path = segment_path(store_root, 1, 1);
                let length = std::fs::metadata(&path).unwrap().len();
                std::fs::OpenOptions::new()
                    .write(true)
                    .open(path)
                    .unwrap()
                    .set_len(length - 1)
                    .unwrap();
            }
            Self::PayloadDigestMismatch => {
                let path = segment_path(store_root, 1, 1);
                let mut bytes = std::fs::read(&path).unwrap();
                bytes[116] ^= 0xff;
                std::fs::write(path, bytes).unwrap();
            }
            Self::SegmentGap => std::fs::remove_file(segment_path(store_root, 2, 1)).unwrap(),
            Self::LsnGap => shift_segment_lsns(&segment_path(store_root, 3, 1), 1),
            Self::GenerationMismatch => {
                let path = segment_path(store_root, 3, 1);
                replace_segment_generation(&path, 2);
                std::fs::rename(path, segment_path(store_root, 3, 2)).unwrap();
            }
        }
    }
}

pub fn build_three_segment_inventory(store_root: &Path) {
    build_three_segment_inventory_with_limit(store_root, 3);
}

pub fn build_three_segment_inventory_with_limit(store_root: &Path, inventory_limit: u32) {
    build_three_segment_inventory_with_policy(store_root, SEGMENT_BYTES, inventory_limit);
}

pub fn build_three_segment_inventory_with_policy(
    store_root: &Path,
    segment_bytes: u64,
    inventory_limit: u32,
) {
    let media_owner = media(store_root);
    let durability =
        durability_with_wal_policy(&media_owner, wal_policy(segment_bytes, inventory_limit));
    let (format, placement, access) = configuration();
    let serving = success(
        media_owner.initialize_record_store(worth_store::physical_runtime::PhysicalRecordInitialization::new(
            format, placement, access, durability,
        )),
    );
    let submission = serving.certification_record_submission();
    append_group(
        &submission,
        vec![prepared(
            &submission,
            placement,
            worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([1; 32]),
            b"first",
        )],
    );
    for (first, second, payload) in [(2, 3, b"second-x"), (4, 5, b"third-xx")] {
        append_group(
            &submission,
            vec![
                prepared(
                    &submission,
                    placement,
                    worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([first; 32]),
                    payload,
                ),
                prepared(
                    &submission,
                    placement,
                    worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([second; 32]),
                    payload,
                ),
            ],
        );
    }
    serving.close();
}

pub fn initialize_empty_store(store_root: &Path) {
    let media_owner = media(store_root);
    let durability = durability_with_wal_policy(&media_owner, wal_policy(SEGMENT_BYTES, 3));
    let (format, placement, access) = configuration();
    success(
        media_owner.initialize_record_store(worth_store::physical_runtime::PhysicalRecordInitialization::new(
            format, placement, access, durability,
        )),
    )
    .close();
}

pub fn append_group(
    submission: &worth_store::physical_runtime::certification::CertificationPhysicalRecordSubmission,
    members: Vec<worth_store::physical_runtime::PreparedPhysicalMutation>,
) {
    let members = worth_proof::NonEmpty::try_from_vec(members)
        .unwrap_or_else(|_| unreachable!("hostile WAL worlds use nonempty groups"));
    if !matches!(
        submission.append_prepared_wal_group(members),
        worth_store::physical_runtime::PhysicalWalGroupAppendOutcome::Appended(_)
    ) {
        panic!("hostile WAL world construction must append every admitted group");
    }
}

pub fn reopen_failure(store_root: &Path, policy: PhysicalWalPolicy) -> PhysicalWalOpenFailure {
    let media_owner = media(store_root);
    let durability = durability_with_wal_policy(&media_owner, policy);
    let (format, _, access) = configuration();
    open_failure_from_media(
        media_owner,
        worth_store::physical_runtime::PhysicalRecordOpen::new(format, access, durability),
    )
}

pub fn open_failure_from_media(
    media: worth_store::physical_runtime::MediaOwnedPhysicalRuntime,
    request: PhysicalRecordOpen,
) -> PhysicalWalOpenFailure {
    match media.open_record_store(request).into_raw() {
        worth_proof::TransitionOutcome::Failed(inspection) => match inspection.cause() {
            worth_store::physical_runtime::RecordBootstrapFailure::SignalConstruction(
                worth_store::physical_runtime::PhysicalSignalConstructionFailure::DurabilityStateReopenRejected(
                    worth_store::physical_runtime::PhysicalDurabilityStateReopenFailure::Wal(failure),
                ),
            ) => failure,
            other => panic!(
                "WAL corruption must fail through the WAL initialization boundary, got {other:?}"
            ),
        },
        _ => panic!("WAL corruption must require typed inspection"),
    }
}

pub fn wal_policy(bytes: u64, inventory: u32) -> PhysicalWalPolicy {
    worth_store::physical_runtime::PhysicalWalPolicy::segmented(
        worth_store::physical_runtime::WalSegmentByteLimit::new(std::num::NonZeroU64::new(bytes).unwrap()),
        worth_store::physical_runtime::WalSegmentInventoryLimit::new(std::num::NonZeroU32::new(inventory).unwrap()),
    )
}

pub fn shift_segment_lsns(path: &Path, delta: u64) {
    let mut bytes = std::fs::read(path).unwrap();
    let mut offset = 0usize;
    while offset < bytes.len() {
        let payload_bytes = read_u64(&bytes, offset + 44) as usize;
        let payload_end = offset + 116 + payload_bytes;
        let frame_end = payload_end + 32;
        for field in [offset + 28, offset + 36] {
            let shifted = read_u64(&bytes, field) + delta;
            bytes[field..field + 8].copy_from_slice(&shifted.to_le_bytes());
        }
        let digest = sha2::Sha256::digest(&bytes[offset..payload_end]);
        bytes[payload_end..frame_end].copy_from_slice(&digest);
        offset = frame_end;
    }
    std::fs::write(path, bytes).unwrap();
}

pub fn corrupt_first_frame_segment_identity(path: &Path, segment: u64) {
    let mut bytes = std::fs::read(path).unwrap();
    bytes[12..20].copy_from_slice(&segment.to_le_bytes());
    std::fs::write(path, bytes).unwrap();
}

pub fn replace_segment_generation(path: &Path, generation: u64) {
    let mut bytes = std::fs::read(path).unwrap();
    let mut offset = 0usize;
    while offset < bytes.len() {
        bytes[offset + 20..offset + 28].copy_from_slice(&generation.to_le_bytes());
        let payload_bytes = read_u64(&bytes, offset + 44) as usize;
        let payload_end = offset + 116 + payload_bytes;
        let frame_end = payload_end + 32;
        let digest = sha2::Sha256::digest(&bytes[offset..payload_end]);
        bytes[payload_end..frame_end].copy_from_slice(&digest);
        offset = frame_end;
    }
    std::fs::write(path, bytes).unwrap();
}

pub fn read_u64(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap())
}

pub fn wal_directory(store_root: &Path) -> std::path::PathBuf {
    store_root.join("families").join("wal")
}

pub fn segment_path(store_root: &Path, segment: u64, generation: u64) -> std::path::PathBuf {
    wal_directory(store_root).join(format!("segment-{segment}-generation-{generation}.wal"))
}