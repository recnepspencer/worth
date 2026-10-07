use std::num::{NonZeroU32, NonZeroU64};
use std::path::Path;

use sha2::{Digest, Sha256};
use worth_proof::{NonEmpty, TransitionOutcome};
use worth_store::physical_runtime::{
    PhysicalDurabilityStateReopenFailure, PhysicalMutationIdempotencyMaterial,
    PhysicalRecordInitialization, PhysicalRecordOpen, PhysicalSignalConstructionFailure,
    PhysicalWalGroupAppendOutcome, PhysicalWalOpenFailure, PhysicalWalPolicy,
    RecordBootstrapFailure, WalSegmentByteLimit, WalSegmentInventoryLimit,
};
use worth_store_physical_backend::MediaOperationRole;
use worth_store_wal::{WalArtifactStoreDenial, WalTopologyDenialKind};

use super::super::{configuration, durability_with_wal_policy, media, success};
use super::wal_append::prepared;

// One two-member group fills the current projection segment exactly.
// Keep this aligned with the independent rotation inventory oracle.
const SEGMENT_BYTES: u64 = 35_304;

#[path = "wal_reopen/interrupted_tail.rs"]
mod interrupted_tail;
#[path = "wal_reopen/member_denial.rs"]
mod member_denial;

#[test]
fn hostile_wal_inventories_fail_with_the_exact_reopen_denial() {
    for case in [
        HostileInventory::InventoryLimit,
        HostileInventory::NonCanonicalName,
        HostileInventory::EmptySegment,
        HostileInventory::SegmentIdentityMismatch,
        HostileInventory::TornFrame,
        HostileInventory::PayloadDigestMismatch,
        HostileInventory::SegmentGap,
        HostileInventory::LsnGap,
        HostileInventory::GenerationMismatch,
    ] {
        let parent = tempfile::tempdir().unwrap();
        let store_root = parent.path().join("store");
        build_three_segment_inventory(&store_root);
        case.corrupt(&store_root);
        assert_eq!(
            reopen_failure(&store_root, case.policy()),
            case.expected(),
            "hostile case {case:?} must fail for its exact cause",
        );
    }
}

#[test]
fn trailing_empty_segment_from_interrupted_rotation_is_cleaned_before_reopen() {
    let parent = tempfile::tempdir().unwrap();
    let store_root = parent.path().join("store");
    build_three_segment_inventory_with_limit(&store_root, 4);
    let trailing = segment_path(&store_root, 4, 1);
    std::fs::write(&trailing, []).unwrap();
    let media_owner = media(&store_root);
    let durability = durability_with_wal_policy(&media_owner, wal_policy(SEGMENT_BYTES, 4));
    let (format, _, access) = configuration();

    let serving = match media_owner
        .open_record_store(PhysicalRecordOpen::new(format, access, durability))
        .into_raw()
    {
        TransitionOutcome::Success(serving) => serving,
        TransitionOutcome::Failed(inspection) => match inspection.cause() {
            RecordBootstrapFailure::SignalConstruction(
                PhysicalSignalConstructionFailure::DurabilityStateReopenRejected(
                    PhysicalDurabilityStateReopenFailure::Wal(failure),
                ),
            ) => panic!("MUTANT_PREDICATE:c7-trailing-empty-wal-cleanup-omitted {failure:?}"),
            other => panic!("MUTANT_PREDICATE:c7-trailing-empty-wal-cleanup-omitted {other:?}"),
        },
        _ => panic!("MUTANT_PREDICATE:c7-trailing-empty-wal-cleanup-omitted"),
    };
    serving.close();
    if trailing.exists() {
        panic!("MUTANT_PREDICATE:c7-trailing-empty-wal-cleanup-omitted");
    }
}

#[test]
fn noncontiguous_empty_tail_is_rejected_without_cleanup() {
    let parent = tempfile::tempdir().unwrap();
    let store_root = parent.path().join("store");
    build_three_segment_inventory_with_limit(&store_root, 4);
    let noncontiguous = segment_path(&store_root, 5, 1);
    std::fs::write(&noncontiguous, []).unwrap();

    assert_eq!(
        reopen_failure(&store_root, wal_policy(SEGMENT_BYTES, 4)),
        PhysicalWalOpenFailure::Topology(WalTopologyDenialKind::NonContiguousSegment),
    );
    assert!(noncontiguous.exists());
}

#[test]
fn oversized_first_segment_is_rejected_before_its_payload_is_read() {
    let empty_parent = tempfile::tempdir().unwrap();
    let empty_root = empty_parent.path().join("store");
    initialize_empty_store(&empty_root);
    let empty_media = media(&empty_root);
    let empty_observer = empty_media.observer();
    let before_empty = empty_observer.media_counters();
    let (format, _, access) = configuration();
    let empty_durability = durability_with_wal_policy(&empty_media, wal_policy(SEGMENT_BYTES, 3));
    success(empty_media.open_record_store(PhysicalRecordOpen::new(
        format,
        access,
        empty_durability,
    )))
    .close();
    let empty_reads = empty_observer
        .media_counters()
        .attempts_for(MediaOperationRole::PositionedRead)
        - before_empty.attempts_for(MediaOperationRole::PositionedRead);

    let oversized_parent = tempfile::tempdir().unwrap();
    let oversized_root = oversized_parent.path().join("store");
    build_three_segment_inventory(&oversized_root);
    let first_segment = segment_path(&oversized_root, 1, 1);
    let observed = std::fs::metadata(first_segment).unwrap().len();
    let admitted = observed - 1;
    let oversized_media = media(&oversized_root);
    let oversized_observer = oversized_media.observer();
    let before_oversized = oversized_observer.media_counters();
    let durability = durability_with_wal_policy(&oversized_media, wal_policy(admitted, 3));
    let (format, _, access) = configuration();
    let failure = open_failure_from_media(
        oversized_media,
        PhysicalRecordOpen::new(format, access, durability),
    );
    assert_eq!(
        failure,
        PhysicalWalOpenFailure::SegmentByteLimitExceeded { admitted, observed },
    );
    let oversized_reads = oversized_observer
        .media_counters()
        .attempts_for(MediaOperationRole::PositionedRead)
        - before_oversized.attempts_for(MediaOperationRole::PositionedRead);
    assert_eq!(
        oversized_reads, empty_reads,
        "oversized WAL rejection must add no payload read to ordinary bootstrap I/O",
    );
}

#[derive(Debug, Clone, Copy)]
enum HostileInventory {
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
    fn policy(self) -> PhysicalWalPolicy {
        let inventory = if matches!(self, Self::InventoryLimit) {
            2
        } else {
            3
        };
        wal_policy(SEGMENT_BYTES, inventory)
    }

    fn expected(self) -> PhysicalWalOpenFailure {
        match self {
            Self::InventoryLimit => PhysicalWalOpenFailure::InventoryLimitExceeded,
            Self::NonCanonicalName => PhysicalWalOpenFailure::NonCanonicalArtifact,
            Self::EmptySegment => PhysicalWalOpenFailure::EmptySegment,
            Self::SegmentIdentityMismatch => PhysicalWalOpenFailure::SegmentInspection(
                WalArtifactStoreDenial::StoreBindingMismatch,
            ),
            Self::TornFrame => {
                PhysicalWalOpenFailure::SegmentInspection(WalArtifactStoreDenial::InvalidFrame)
            }
            Self::PayloadDigestMismatch => {
                PhysicalWalOpenFailure::SegmentInspection(WalArtifactStoreDenial::DigestMismatch)
            }
            Self::SegmentGap => {
                PhysicalWalOpenFailure::Topology(WalTopologyDenialKind::NonContiguousSegment)
            }
            Self::LsnGap => PhysicalWalOpenFailure::Topology(WalTopologyDenialKind::Gap),
            Self::GenerationMismatch => {
                PhysicalWalOpenFailure::Topology(WalTopologyDenialKind::WrongGeneration)
            }
        }
    }

    fn corrupt(self, store_root: &Path) {
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

fn build_three_segment_inventory(store_root: &Path) {
    build_three_segment_inventory_with_limit(store_root, 3);
}

fn build_three_segment_inventory_with_limit(store_root: &Path, inventory_limit: u32) {
    build_three_segment_inventory_with_policy(store_root, SEGMENT_BYTES, inventory_limit);
}

fn build_three_segment_inventory_with_policy(
    store_root: &Path,
    segment_bytes: u64,
    inventory_limit: u32,
) {
    let media_owner = media(store_root);
    let durability =
        durability_with_wal_policy(&media_owner, wal_policy(segment_bytes, inventory_limit));
    let (format, placement, access) = configuration();
    let serving = success(
        media_owner.initialize_record_store(PhysicalRecordInitialization::new(
            format, placement, access, durability,
        )),
    );
    let submission = serving.certification_record_submission();
    append_group(
        &submission,
        vec![prepared(
            &submission,
            placement,
            PhysicalMutationIdempotencyMaterial::new([1; 32]),
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
                    PhysicalMutationIdempotencyMaterial::new([first; 32]),
                    payload,
                ),
                prepared(
                    &submission,
                    placement,
                    PhysicalMutationIdempotencyMaterial::new([second; 32]),
                    payload,
                ),
            ],
        );
    }
    serving.close();
}

fn initialize_empty_store(store_root: &Path) {
    let media_owner = media(store_root);
    let durability = durability_with_wal_policy(&media_owner, wal_policy(SEGMENT_BYTES, 3));
    let (format, placement, access) = configuration();
    success(
        media_owner.initialize_record_store(PhysicalRecordInitialization::new(
            format, placement, access, durability,
        )),
    )
    .close();
}

fn append_group(
    submission: &worth_store::physical_runtime::certification::CertificationPhysicalRecordSubmission,
    members: Vec<worth_store::physical_runtime::PreparedPhysicalMutation>,
) {
    let members = NonEmpty::try_from_vec(members)
        .unwrap_or_else(|_| unreachable!("hostile WAL worlds use nonempty groups"));
    if !matches!(
        submission.append_prepared_wal_group(members),
        PhysicalWalGroupAppendOutcome::Appended(_)
    ) {
        panic!("hostile WAL world construction must append every admitted group");
    }
}

fn reopen_failure(store_root: &Path, policy: PhysicalWalPolicy) -> PhysicalWalOpenFailure {
    let media_owner = media(store_root);
    let durability = durability_with_wal_policy(&media_owner, policy);
    let (format, _, access) = configuration();
    open_failure_from_media(
        media_owner,
        PhysicalRecordOpen::new(format, access, durability),
    )
}

fn open_failure_from_media(
    media: worth_store::physical_runtime::MediaOwnedPhysicalRuntime,
    request: PhysicalRecordOpen,
) -> PhysicalWalOpenFailure {
    match media.open_record_store(request).into_raw() {
        TransitionOutcome::Failed(inspection) => match inspection.cause() {
            RecordBootstrapFailure::SignalConstruction(
                PhysicalSignalConstructionFailure::DurabilityStateReopenRejected(
                    PhysicalDurabilityStateReopenFailure::Wal(failure),
                ),
            ) => failure,
            other => panic!(
                "WAL corruption must fail through the WAL initialization boundary, got {other:?}"
            ),
        },
        _ => panic!("WAL corruption must require typed inspection"),
    }
}

fn wal_policy(bytes: u64, inventory: u32) -> PhysicalWalPolicy {
    PhysicalWalPolicy::segmented(
        WalSegmentByteLimit::new(NonZeroU64::new(bytes).unwrap()),
        WalSegmentInventoryLimit::new(NonZeroU32::new(inventory).unwrap()),
    )
}

fn shift_segment_lsns(path: &Path, delta: u64) {
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
        let digest = Sha256::digest(&bytes[offset..payload_end]);
        bytes[payload_end..frame_end].copy_from_slice(&digest);
        offset = frame_end;
    }
    std::fs::write(path, bytes).unwrap();
}

fn corrupt_first_frame_segment_identity(path: &Path, segment: u64) {
    let mut bytes = std::fs::read(path).unwrap();
    bytes[12..20].copy_from_slice(&segment.to_le_bytes());
    std::fs::write(path, bytes).unwrap();
}

fn replace_segment_generation(path: &Path, generation: u64) {
    let mut bytes = std::fs::read(path).unwrap();
    let mut offset = 0usize;
    while offset < bytes.len() {
        bytes[offset + 20..offset + 28].copy_from_slice(&generation.to_le_bytes());
        let payload_bytes = read_u64(&bytes, offset + 44) as usize;
        let payload_end = offset + 116 + payload_bytes;
        let frame_end = payload_end + 32;
        let digest = Sha256::digest(&bytes[offset..payload_end]);
        bytes[payload_end..frame_end].copy_from_slice(&digest);
        offset = frame_end;
    }
    std::fs::write(path, bytes).unwrap();
}

fn read_u64(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap())
}

fn wal_directory(store_root: &Path) -> std::path::PathBuf {
    store_root.join("families").join("wal")
}

fn segment_path(store_root: &Path, segment: u64, generation: u64) -> std::path::PathBuf {
    wal_directory(store_root).join(format!("segment-{segment}-generation-{generation}.wal"))
}
