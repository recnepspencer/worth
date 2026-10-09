//! One terminal head retirement over a one-leaf selected head tree, shared by
//! the C.9 validation, replay, admission and root-step tests.

use sha2::{Digest, Sha256};
use worth_store_physical_format::store_namespace::{
    ProposedStoreIdentity, StableStoreIdentity, StoreNamespaceIdentityRecord, StoreNamespaceVersion,
};
use worth_store_physical_format::{
    BlobGenerationPublicationV1, BlobReclaimSourceBasisV1, CurrentPhysicalRecordPlacement,
    DurableExtentRecordPlacement, DurablePhysicalRootManifest, ExtentArenaId, ExtentArenaRange,
    ExtentChunkCoordinate, PersistedPhysicalDataFrameSubject, PersistedPhysicalRecoveryFrame,
    PersistedPhysicalRecoveryOperation, PersistedPhysicalRecoveryProjection,
    PersistedPhysicalRecoveryRootState, PersistedRecordIdentity,
    PersistedTerminalReleaseHeadRetirementV1, PhysicalExtentId, PhysicalGeneration,
    PhysicalGenerationAuthority, PhysicalRecordFormatDeclaration, RecordArtifactFile,
    RecordFrameCoordinate, ReleaseCustodyHeadBlockV1, ReleaseCustodyHeadEntryV1,
    ReleaseCustodyHeadKeyV1, ReleaseCustodyHeadMutationV1, ReleaseCustodyHeadPathNodeV1,
    ReleaseCustodyHeadTransitionLimitsV1, ReleaseCustodyHeadTransitionV1,
    ReleasedGenerationReclaimBasisV1, CANONICAL_REDO_V3_DOMAIN,
};
use worth_store_wal::{LogSequenceNumber, WalLsnRange};

use crate::{PhysicalRedoMemberInput, RecoveryOperationFate};

pub(crate) const TREE: u64 = 6;
pub(crate) const SOURCE_GENERATION: u64 = 11;
const STORE: [u8; 16] = [7; 16];

pub(crate) fn format() -> PhysicalRecordFormatDeclaration {
    PhysicalRecordFormatDeclaration::builder().admit().unwrap()
}

pub(crate) fn store() -> StableStoreIdentity {
    store_of(STORE)
}

pub(crate) fn store_of(bytes: [u8; 16]) -> StableStoreIdentity {
    StoreNamespaceIdentityRecord::new(
        StoreNamespaceVersion::CURRENT,
        ProposedStoreIdentity::from_nonzero_bytes(bytes).unwrap(),
    )
    .published_identity()
}

pub(crate) fn record(ordinal: u64) -> PersistedRecordIdentity {
    PersistedRecordIdentity::new([1; 16], ordinal).unwrap()
}

fn basis() -> ReleasedGenerationReclaimBasisV1 {
    let publication = BlobGenerationPublicationV1::new(
        STORE,
        [2; 16],
        [3; 16],
        3,
        record(3),
        [4; 32],
        8,
        [5; 32],
        64 * 1024,
        [6; 32],
    )
    .unwrap();
    ReleasedGenerationReclaimBasisV1::new(
        publication,
        record(4),
        Sha256::digest(publication.encode()).into(),
        [6; 32],
    )
    .unwrap()
}

fn head(
    object: [u8; 16],
    generation: u64,
    source_basis_digest: [u8; 32],
    terminal: bool,
) -> ReleaseCustodyHeadEntryV1 {
    ReleaseCustodyHeadEntryV1::new(
        ReleaseCustodyHeadKeyV1::new(object, generation).unwrap(),
        record(9),
        [5; 32],
        record(7),
        [8; 32],
        record(8),
        [9; 32],
        source_basis_digest,
        None,
        9,
        1,
        terminal,
    )
    .unwrap()
}

/// The terminal head of exactly the object key the source basis names.
pub(crate) fn terminal_head() -> ReleaseCustodyHeadEntryV1 {
    let digest = BlobReclaimSourceBasisV1::ReleasedGeneration(basis()).digest(STORE);
    head(basis().object(), basis().generation(), digest, true)
}

/// The same key while its release is still in flight.
pub(crate) fn nonterminal_head() -> ReleaseCustodyHeadEntryV1 {
    let digest = BlobReclaimSourceBasisV1::ReleasedGeneration(basis()).digest(STORE);
    head(basis().object(), basis().generation(), digest, false)
}

pub(crate) fn survivor() -> ReleaseCustodyHeadEntryV1 {
    head([9; 16], 1, [0x33; 32], false)
}

/// A selected head tree holding `entries`, and the WAL claim that retires
/// its terminal head.
pub(crate) struct SelectedTerminalHead {
    pub(crate) retirement: PersistedTerminalReleaseHeadRetirementV1,
    pub(crate) source: DurablePhysicalRootManifest,
    pub(crate) source_frame: Vec<u8>,
}

/// A one-leaf head tree written at generation 10 as block 1.
pub(crate) fn source_leaf(
    entries: Vec<ReleaseCustodyHeadEntryV1>,
    format: PhysicalRecordFormatDeclaration,
) -> ReleaseCustodyHeadPathNodeV1 {
    let leaf = ReleaseCustodyHeadBlockV1::leaf(TREE, 10, 1, entries, format).unwrap();
    ReleaseCustodyHeadPathNodeV1::new(leaf.reference(format), leaf.encode(format))
}

pub(crate) fn selected_terminal_head(
    entries: Vec<ReleaseCustodyHeadEntryV1>,
) -> SelectedTerminalHead {
    selected_terminal_head_in(entries, format())
}

pub(crate) fn selected_terminal_head_in(
    entries: Vec<ReleaseCustodyHeadEntryV1>,
    format: PhysicalRecordFormatDeclaration,
) -> SelectedTerminalHead {
    let node = source_leaf(entries, format);
    let root = node.reference();
    let source_frame = node.frame().to_vec();
    let path = vec![node];
    let planned = ReleaseCustodyHeadTransitionV1::plan(
        Some(root),
        2,
        &path,
        ReleaseCustodyHeadMutationV1::RetireTerminal {
            expected_prior: terminal_head(),
        },
        SOURCE_GENERATION + 1,
        TREE,
        format,
        ReleaseCustodyHeadTransitionLimitsV1::new(1, 1, 8 * 65_536).unwrap(),
    )
    .unwrap();
    let retirement = PersistedTerminalReleaseHeadRetirementV1::new(
        SOURCE_GENERATION,
        record(2),
        [0x44; 32],
        TREE,
        basis(),
        path,
        planned,
        format,
    )
    .unwrap();
    let source = DurablePhysicalRootManifest::builder(SOURCE_GENERATION, TREE, 4, 1)
        .release_custody_head_root(Some(root))
        .next_release_custody_head_block(2)
        .admit()
        .unwrap();
    SelectedTerminalHead {
        retirement,
        source,
        source_frame,
    }
}

pub(crate) fn projection(
    retirement: PersistedTerminalReleaseHeadRetirementV1,
) -> PersistedPhysicalRecoveryProjection {
    PersistedPhysicalRecoveryProjection::new_with_operation(
        SOURCE_GENERATION,
        PersistedPhysicalRecoveryRootState::new(4096, 1, 32, vec![], None, None).unwrap(),
        vec![],
        vec![],
        vec![],
        vec![],
        vec![],
        PersistedPhysicalRecoveryOperation::TerminalReleaseHeadRetired(retirement),
    )
    .unwrap()
}

/// An ordinary one-record projection, which retires nothing.
pub(crate) fn data_projection() -> PersistedPhysicalRecoveryProjection {
    let bytes = b"redo-record";
    let length = bytes.len() as u64;
    let extent = PhysicalGenerationAuthority::for_canonical_physical_format()
        .record_extent_cell(PhysicalExtentId::from_raw(3).unwrap())
        .with_extent_generation(PhysicalGeneration::from_raw(4).unwrap());
    let range = ExtentArenaRange::new(ExtentArenaId::new(2).unwrap(), 0, 53_248).unwrap();
    let placement =
        DurableExtentRecordPlacement::legacy_unknown(record(9), extent, length, range).unwrap();
    let chunk = ExtentChunkCoordinate::new(record(9), extent, length, 0, length as u32).unwrap();
    let coordinate = RecordFrameCoordinate::new(
        RecordArtifactFile::ExtentArena { arena: 2 },
        0,
        length as u32,
    )
    .unwrap();
    let frame = PersistedPhysicalRecoveryFrame::new(
        PersistedPhysicalDataFrameSubject::ExtentChunk(chunk),
        coordinate,
        bytes,
    )
    .unwrap();
    PersistedPhysicalRecoveryProjection::new_with_operation(
        SOURCE_GENERATION,
        PersistedPhysicalRecoveryRootState::new(4096, 1, 32, vec![], None, None).unwrap(),
        vec![record(9)],
        vec![frame],
        vec![CurrentPhysicalRecordPlacement::Extent(placement)],
        vec![],
        vec![],
        PersistedPhysicalRecoveryOperation::None,
    )
    .unwrap()
}

/// The record-less canonical member carrying `projection`, occupying the one
/// LSN `lsn` as a singleton group identified by `identity`.
pub(crate) fn member(
    projection: &PersistedPhysicalRecoveryProjection,
    lsn: u64,
    identity: u8,
    fate: RecoveryOperationFate,
) -> PhysicalRedoMemberInput {
    let encoded = projection.encode();
    let mut redo = Vec::new();
    redo.extend_from_slice(&(CANONICAL_REDO_V3_DOMAIN.len() as u64).to_le_bytes());
    redo.extend_from_slice(CANONICAL_REDO_V3_DOMAIN);
    redo.extend_from_slice(&0_u64.to_le_bytes());
    redo.extend_from_slice(&(encoded.len() as u64).to_le_bytes());
    redo.extend_from_slice(&encoded);
    PhysicalRedoMemberInput::new(
        WalLsnRange::new(LogSequenceNumber::new(lsn), LogSequenceNumber::new(lsn + 1)).unwrap(),
        [identity; 32],
        fate,
        &redo,
    )
}
