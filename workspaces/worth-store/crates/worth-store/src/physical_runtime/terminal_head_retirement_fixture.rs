//! One record-less terminal head retirement member that retires the only
//! head of its tree. Store readers that do not yet handle the member must
//! deny it.

use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    BlobGenerationPublicationV1, BlobReclaimSourceBasisV1, PersistedPhysicalRecoveryOperation,
    PersistedPhysicalRecoveryProjection, PersistedPhysicalRecoveryRootState,
    PersistedRecordIdentity, PersistedTerminalReleaseHeadRetirementV1,
    PhysicalRecordFormatDeclaration, ReleaseCustodyHeadBlockV1, ReleaseCustodyHeadEntryV1,
    ReleaseCustodyHeadKeyV1, ReleaseCustodyHeadMutationV1, ReleaseCustodyHeadPathNodeV1,
    ReleaseCustodyHeadTransitionLimitsV1, ReleaseCustodyHeadTransitionV1,
    ReleasedGenerationReclaimBasisV1, CANONICAL_REDO_V3_DOMAIN,
};

const STORE: [u8; 16] = [7; 16];
const TREE: u64 = 6;
const SOURCE_GENERATION: u64 = 11;

fn record(ordinal: u64) -> PersistedRecordIdentity {
    PersistedRecordIdentity::new([1; 16], ordinal).unwrap()
}

pub(super) fn basis() -> ReleasedGenerationReclaimBasisV1 {
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

/// The terminal head of exactly the object key the source basis names.
fn terminal_head() -> ReleaseCustodyHeadEntryV1 {
    ReleaseCustodyHeadEntryV1::new(
        ReleaseCustodyHeadKeyV1::new(basis().object(), basis().generation()).unwrap(),
        record(9),
        [5; 32],
        record(7),
        [8; 32],
        record(8),
        [9; 32],
        BlobReclaimSourceBasisV1::ReleasedGeneration(basis()).digest(STORE),
        None,
        9,
        1,
        true,
    )
    .unwrap()
}

/// Retires the only head of tree 6 at source generation 11: the result has
/// no head root.
pub(super) fn retirement() -> PersistedTerminalReleaseHeadRetirementV1 {
    let format = PhysicalRecordFormatDeclaration::builder().admit().unwrap();
    let terminal_head = terminal_head();
    let leaf = ReleaseCustodyHeadBlockV1::leaf(TREE, 10, 1, vec![terminal_head], format).unwrap();
    let path = vec![ReleaseCustodyHeadPathNodeV1::new(
        leaf.reference(format),
        leaf.encode(format),
    )];
    let planned = ReleaseCustodyHeadTransitionV1::plan(
        Some(leaf.reference(format)),
        2,
        &path,
        ReleaseCustodyHeadMutationV1::RetireTerminal {
            expected_prior: terminal_head,
        },
        SOURCE_GENERATION + 1,
        TREE,
        format,
        ReleaseCustodyHeadTransitionLimitsV1::new(1, 1, 8 * 65_536).unwrap(),
    )
    .unwrap();
    PersistedTerminalReleaseHeadRetirementV1::new(
        SOURCE_GENERATION,
        record(2),
        [0x44; 32],
        TREE,
        basis(),
        path,
        planned,
        format,
    )
    .unwrap()
}

pub(super) fn projection() -> PersistedPhysicalRecoveryProjection {
    PersistedPhysicalRecoveryProjection::new_with_operation(
        SOURCE_GENERATION,
        PersistedPhysicalRecoveryRootState::new(4096, 1, 32, vec![], None, None).unwrap(),
        vec![],
        vec![],
        vec![],
        vec![],
        vec![],
        PersistedPhysicalRecoveryOperation::TerminalReleaseHeadRetired(retirement()),
    )
    .unwrap()
}

/// The canonical redo of the member: no data record, one projection.
pub(super) fn canonical_redo() -> Vec<u8> {
    let encoded = projection().encode();
    let mut redo = Vec::new();
    redo.extend_from_slice(&(CANONICAL_REDO_V3_DOMAIN.len() as u64).to_le_bytes());
    redo.extend_from_slice(CANONICAL_REDO_V3_DOMAIN);
    redo.extend_from_slice(&0_u64.to_le_bytes());
    redo.extend_from_slice(&(encoded.len() as u64).to_le_bytes());
    redo.extend_from_slice(&encoded);
    redo
}
