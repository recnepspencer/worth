use crate::{
    BlobReclaimSourceBasisV1, PhysicalRecordFormatDeclaration, ReleaseCustodyHeadBlockReferenceV1,
    ReleaseCustodyHeadMutationV1, ReleaseCustodyHeadNodeWriteV1, ReleaseCustodyHeadPathNodeV1,
    ReleaseCustodyHeadTransitionLimitsV1, ReleaseCustodyHeadTransitionV1,
    ReleasedGenerationReclaimBasisV1,
};

use super::PhysicalRecoveryProjectionDenial;

/// A canonical WAL claim for one keyed release-head upsert. The source path is
/// still untrusted until C8 joins each referenced frame to selected media.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersistedReleaseCustodyHeadEffectV1 {
    tree_identity: u64,
    source_basis: ReleasedGenerationReclaimBasisV1,
    source_root: Option<ReleaseCustodyHeadBlockReferenceV1>,
    source_next_block: u64,
    source_path: Box<[ReleaseCustodyHeadPathNodeV1]>,
    mutation: ReleaseCustodyHeadMutationV1,
    result_root: ReleaseCustodyHeadBlockReferenceV1,
    result_next_block: u64,
    node_writes: Box<[ReleaseCustodyHeadNodeWriteV1]>,
}

impl PersistedReleaseCustodyHeadEffectV1 {
    /// Additional heap needed while recomputing this exact COW path. The
    /// already-retained effect is measured separately by `owned_heap_bytes`.
    pub fn verification_additional_peak_bytes(
        &self,
        format: PhysicalRecordFormatDeclaration,
    ) -> Option<u64> {
        ReleaseCustodyHeadTransitionV1::verification_additional_peak_bytes(
            &self.source_path,
            self.node_writes.len(),
            format,
        )
    }

    pub fn owned_heap_bytes(&self) -> Option<u64> {
        let path = u64::try_from(self.source_path.len()).ok()?.checked_mul(
            u64::try_from(std::mem::size_of::<ReleaseCustodyHeadPathNodeV1>()).ok()?,
        )?;
        let writes = u64::try_from(self.node_writes.len()).ok()?.checked_mul(
            u64::try_from(std::mem::size_of::<ReleaseCustodyHeadNodeWriteV1>()).ok()?,
        )?;
        let bytes = self
            .source_path
            .iter()
            .try_fold(path.checked_add(writes)?, |bytes, node| {
                bytes.checked_add(node.owned_heap_bytes()?)
            })?;
        self.node_writes.iter().try_fold(bytes, |sum, write| {
            sum.checked_add(write.owned_heap_bytes()?)
        })
    }

    /// The producer supplies its pre-WAL plan; C9 independently recomputes it
    /// from the persisted path and exact proposed mutation.
    #[allow(clippy::too_many_arguments)]
    pub fn new_upsert(
        tree_identity: u64,
        source_root_generation: u64,
        source_basis: ReleasedGenerationReclaimBasisV1,
        source_path: Vec<ReleaseCustodyHeadPathNodeV1>,
        planned: ReleaseCustodyHeadTransitionV1,
        format: PhysicalRecordFormatDeclaration,
        limits: ReleaseCustodyHeadTransitionLimitsV1,
    ) -> Result<Self, PhysicalRecoveryProjectionDenial> {
        let ReleaseCustodyHeadMutationV1::Upsert { next, .. } = planned.mutation() else {
            return Err(PhysicalRecoveryProjectionDenial::Malformed);
        };
        let Some(result_generation) = source_root_generation.checked_add(1) else {
            return Err(PhysicalRecoveryProjectionDenial::Malformed);
        };
        if tree_identity == 0
            || next.key().object() != source_basis.object()
            || next.key().generation() != source_basis.generation()
            || next.source_basis_digest()
                != BlobReclaimSourceBasisV1::ReleasedGeneration(source_basis)
                    .digest(source_basis.publication().store())
            || next.source_root_generation() != source_root_generation
        {
            return Err(PhysicalRecoveryProjectionDenial::Malformed);
        }
        let result_root = planned
            .result_root()
            .ok_or(PhysicalRecoveryProjectionDenial::Malformed)?;
        ReleaseCustodyHeadTransitionV1::verify_exact(
            planned.source_root(),
            planned.source_next_block(),
            &source_path,
            planned.mutation(),
            result_generation,
            tree_identity,
            format,
            limits,
            Some(result_root),
            planned.result_next_block(),
            planned.writes(),
        )
        .map_err(|_| PhysicalRecoveryProjectionDenial::Malformed)?;
        let (source_root, source_next_block, mutation, _, result_next_block, node_writes, _) =
            planned.into_parts();
        Ok(Self {
            tree_identity,
            source_basis,
            source_root,
            source_next_block,
            source_path: source_path.into_boxed_slice(),
            mutation,
            result_root,
            result_next_block,
            node_writes: node_writes.into_boxed_slice(),
        })
    }

    pub const fn tree_identity(&self) -> u64 {
        self.tree_identity
    }
    pub const fn source_basis(&self) -> ReleasedGenerationReclaimBasisV1 {
        self.source_basis
    }
    pub const fn source_root(&self) -> Option<ReleaseCustodyHeadBlockReferenceV1> {
        self.source_root
    }
    pub const fn source_next_block(&self) -> u64 {
        self.source_next_block
    }
    pub fn source_path(&self) -> &[ReleaseCustodyHeadPathNodeV1] {
        &self.source_path
    }
    pub const fn mutation(&self) -> ReleaseCustodyHeadMutationV1 {
        self.mutation
    }
    pub const fn result_root(&self) -> ReleaseCustodyHeadBlockReferenceV1 {
        self.result_root
    }
    pub const fn result_next_block(&self) -> u64 {
        self.result_next_block
    }
    pub fn node_writes(&self) -> &[ReleaseCustodyHeadNodeWriteV1] {
        &self.node_writes
    }

    pub fn entry_count(&self) -> Option<u64> {
        u64::try_from(self.source_path.len())
            .ok()?
            .checked_add(u64::try_from(self.node_writes.len()).ok()?)
    }

    pub fn framed_bytes(&self) -> Option<u64> {
        let paths = self.source_path.iter().try_fold(0_u64, |sum, node| {
            sum.checked_add(node.frame().len() as u64)
        })?;
        let writes = self.node_writes.iter().try_fold(0_u64, |sum, write| {
            sum.checked_add(write.frame().len() as u64)
        })?;
        paths.checked_add(writes)
    }

    pub fn verify_exact(
        &self,
        source_root_generation: u64,
        format: PhysicalRecordFormatDeclaration,
    ) -> Result<(), PhysicalRecoveryProjectionDenial> {
        let result_generation = source_root_generation
            .checked_add(1)
            .ok_or(PhysicalRecoveryProjectionDenial::Malformed)?;
        let limits = ReleaseCustodyHeadTransitionLimitsV1::new(
            u16::try_from(self.source_path.len().max(1))
                .map_err(|_| PhysicalRecoveryProjectionDenial::EntryLimit)?,
            u16::try_from(self.node_writes.len())
                .map_err(|_| PhysicalRecoveryProjectionDenial::EntryLimit)?,
            self.source_path
                .iter()
                .try_fold(0_u64, |sum, node| {
                    sum.checked_add(node.frame().len() as u64)
                })
                .and_then(|path_bytes| {
                    (self.node_writes.len() as u64)
                        .checked_mul(u64::from(format.page_size().bytes()))
                        .and_then(|maximum_writes| path_bytes.checked_add(maximum_writes))
                })
                .ok_or(PhysicalRecoveryProjectionDenial::EntryLimit)?,
        )
        .ok_or(PhysicalRecoveryProjectionDenial::EntryLimit)?;
        ReleaseCustodyHeadTransitionV1::verify_exact(
            self.source_root,
            self.source_next_block,
            &self.source_path,
            self.mutation,
            result_generation,
            self.tree_identity,
            format,
            limits,
            Some(self.result_root),
            self.result_next_block,
            &self.node_writes,
        )
        .map_err(|_| PhysicalRecoveryProjectionDenial::Malformed)?;
        Ok(())
    }
}
