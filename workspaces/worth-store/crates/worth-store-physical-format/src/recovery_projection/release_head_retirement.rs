//! The WAL claim that one terminal release head left the selected head tree.
//! This is a terminal head retirement: a WAL member of its own, with no data
//! record. It is unrelated to a release intent retired by a checkpoint.

use crate::{
    BlobReclaimSourceBasisV1, PersistedRecordIdentity, PhysicalRecordFormatDeclaration,
    ReleaseCustodyHeadBlockReferenceV1, ReleaseCustodyHeadEntryV1, ReleaseCustodyHeadMutationV1,
    ReleaseCustodyHeadNodeWriteV1, ReleaseCustodyHeadPathNodeV1, ReleaseCustodyHeadTransitionV1,
    ReleasedGenerationReclaimBasisV1,
};

use super::{release_head_tree_claim, PhysicalRecoveryProjectionDenial};

/// A canonical WAL claim that removes exactly one terminal head entry. The
/// source path is still untrusted until C8 joins each referenced frame to
/// selected media, and the bound session declaration is a claim until its
/// record is re-read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersistedTerminalReleaseHeadRetirementV1 {
    source_root_generation: u64,
    declaration_record: PersistedRecordIdentity,
    declaration_frame_sha256: [u8; 32],
    tree_identity: u64,
    source_basis: ReleasedGenerationReclaimBasisV1,
    source_root: ReleaseCustodyHeadBlockReferenceV1,
    source_next_block: u64,
    source_path: Box<[ReleaseCustodyHeadPathNodeV1]>,
    expected_prior: ReleaseCustodyHeadEntryV1,
    result_root: Option<ReleaseCustodyHeadBlockReferenceV1>,
    result_next_block: u64,
    node_writes: Box<[ReleaseCustodyHeadNodeWriteV1]>,
}

impl PersistedTerminalReleaseHeadRetirementV1 {
    /// The producer supplies its pre-WAL plan; the retained claim is admitted
    /// only when recomputing it from the carried path yields that exact plan.
    /// `source_basis` must name the object key being retired, and the session
    /// declaration identity is the non-reissue fact later readers re-verify.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        source_root_generation: u64,
        declaration_record: PersistedRecordIdentity,
        declaration_frame_sha256: [u8; 32],
        tree_identity: u64,
        source_basis: ReleasedGenerationReclaimBasisV1,
        source_path: Vec<ReleaseCustodyHeadPathNodeV1>,
        planned: ReleaseCustodyHeadTransitionV1,
        format: PhysicalRecordFormatDeclaration,
    ) -> Result<Self, PhysicalRecoveryProjectionDenial> {
        let ReleaseCustodyHeadMutationV1::RetireTerminal { expected_prior } = planned.mutation()
        else {
            return Err(PhysicalRecoveryProjectionDenial::Malformed);
        };
        let source_root = planned
            .source_root()
            .ok_or(PhysicalRecoveryProjectionDenial::Malformed)?;
        if expected_prior.key().object() != source_basis.object()
            || expected_prior.key().generation() != source_basis.generation()
            || expected_prior.source_basis_digest()
                != BlobReclaimSourceBasisV1::ReleasedGeneration(source_basis)
                    .digest(source_basis.publication().store())
            || expected_prior.source_root_generation() >= source_root_generation
            || declaration_frame_sha256 == [0; 32]
            || declaration_record == source_basis.publication_record()
        {
            return Err(PhysicalRecoveryProjectionDenial::Malformed);
        }
        let (_, source_next_block, _, result_root, result_next_block, node_writes, _) =
            planned.into_parts();
        let retirement = Self {
            source_root_generation,
            declaration_record,
            declaration_frame_sha256,
            tree_identity,
            source_basis,
            source_root,
            source_next_block,
            source_path: source_path.into_boxed_slice(),
            expected_prior,
            result_root,
            result_next_block,
            node_writes: node_writes.into_boxed_slice(),
        };
        retirement.verify_exact(format)?;
        Ok(retirement)
    }

    /// Recompute the copy-on-write removal from the carried path. The result
    /// tree is the source tree minus exactly the expected terminal entry.
    pub fn verify_exact(
        &self,
        format: PhysicalRecordFormatDeclaration,
    ) -> Result<(), PhysicalRecoveryProjectionDenial> {
        let result_generation = self
            .source_root_generation
            .checked_add(1)
            .ok_or(PhysicalRecoveryProjectionDenial::Malformed)?;
        let limits =
            release_head_tree_claim::exact_limits(&self.source_path, self.new_blocks(), format)?;
        ReleaseCustodyHeadTransitionV1::verify_exact(
            Some(self.source_root),
            self.source_next_block,
            &self.source_path,
            self.mutation(),
            result_generation,
            self.tree_identity,
            format,
            limits,
            self.result_root,
            self.result_next_block,
            &self.node_writes,
        )
        .map_err(|_| PhysicalRecoveryProjectionDenial::Malformed)?;
        Ok(())
    }

    /// Additional heap needed while recomputing this exact removal. The
    /// already-retained claim is measured separately by `owned_heap_bytes`.
    pub fn verification_additional_peak_bytes(
        &self,
        format: PhysicalRecordFormatDeclaration,
    ) -> Option<u64> {
        ReleaseCustodyHeadTransitionV1::verification_additional_peak_bytes(
            &self.source_path,
            self.new_blocks(),
            format,
        )
    }

    /// Retiring the last head writes no block; the planner still reserves one.
    pub(super) fn reserved_new_blocks(node_writes: &[ReleaseCustodyHeadNodeWriteV1]) -> usize {
        node_writes.len().max(1)
    }

    fn new_blocks(&self) -> usize {
        Self::reserved_new_blocks(&self.node_writes)
    }

    pub const fn source_root_generation(&self) -> u64 {
        self.source_root_generation
    }
    pub const fn declaration_record(&self) -> PersistedRecordIdentity {
        self.declaration_record
    }
    pub const fn declaration_frame_sha256(&self) -> [u8; 32] {
        self.declaration_frame_sha256
    }
    pub const fn tree_identity(&self) -> u64 {
        self.tree_identity
    }
    pub const fn source_basis(&self) -> ReleasedGenerationReclaimBasisV1 {
        self.source_basis
    }
    pub const fn source_root(&self) -> ReleaseCustodyHeadBlockReferenceV1 {
        self.source_root
    }
    pub const fn source_next_block(&self) -> u64 {
        self.source_next_block
    }
    pub fn source_path(&self) -> &[ReleaseCustodyHeadPathNodeV1] {
        &self.source_path
    }
    pub const fn expected_prior(&self) -> ReleaseCustodyHeadEntryV1 {
        self.expected_prior
    }
    pub const fn mutation(&self) -> ReleaseCustodyHeadMutationV1 {
        ReleaseCustodyHeadMutationV1::RetireTerminal {
            expected_prior: self.expected_prior,
        }
    }
    /// Absent exactly when the retired head was the last entry in the tree.
    pub const fn result_root(&self) -> Option<ReleaseCustodyHeadBlockReferenceV1> {
        self.result_root
    }
    pub const fn result_next_block(&self) -> u64 {
        self.result_next_block
    }
    pub fn node_writes(&self) -> &[ReleaseCustodyHeadNodeWriteV1] {
        &self.node_writes
    }

    pub fn entry_count(&self) -> Option<u64> {
        release_head_tree_claim::entry_count(&self.source_path, &self.node_writes)
    }
    pub fn framed_bytes(&self) -> Option<u64> {
        release_head_tree_claim::framed_bytes(&self.source_path, &self.node_writes)
    }
    pub fn owned_heap_bytes(&self) -> Option<u64> {
        release_head_tree_claim::owned_heap_bytes(&self.source_path, &self.node_writes)
    }
}
