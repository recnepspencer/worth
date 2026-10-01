mod plan;
mod verification_budget;

use crate::PhysicalRecordFormatDeclaration;

use super::{
    ReleaseCustodyHeadBlockReferenceV1, ReleaseCustodyHeadDenial, ReleaseCustodyHeadEntryV1,
    ReleaseCustodyHeadKeyV1,
};

/// Raw source bytes are not authority until joined to the selected root by the caller.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleaseCustodyHeadPathNodeV1 {
    reference: ReleaseCustodyHeadBlockReferenceV1,
    frame: Vec<u8>,
}

impl ReleaseCustodyHeadPathNodeV1 {
    pub fn new(reference: ReleaseCustodyHeadBlockReferenceV1, frame: Vec<u8>) -> Self {
        Self { reference, frame }
    }
    pub const fn reference(&self) -> ReleaseCustodyHeadBlockReferenceV1 {
        self.reference
    }
    pub fn frame(&self) -> &[u8] {
        &self.frame
    }
    pub fn owned_heap_bytes(&self) -> Option<u64> {
        u64::try_from(self.frame.capacity()).ok()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleaseCustodyHeadNodeWriteV1 {
    reference: ReleaseCustodyHeadBlockReferenceV1,
    frame: Vec<u8>,
}

impl ReleaseCustodyHeadNodeWriteV1 {
    pub fn new(reference: ReleaseCustodyHeadBlockReferenceV1, frame: Vec<u8>) -> Self {
        Self { reference, frame }
    }
    pub const fn reference(&self) -> ReleaseCustodyHeadBlockReferenceV1 {
        self.reference
    }
    pub fn frame(&self) -> &[u8] {
        &self.frame
    }
    pub fn owned_heap_bytes(&self) -> Option<u64> {
        u64::try_from(self.frame.capacity()).ok()
    }
}

#[cfg(test)]
mod retained_storage_tests {
    use super::*;

    #[test]
    fn head_node_counts_reserved_frame_capacity() {
        let mut frame = Vec::with_capacity(32);
        frame.extend_from_slice(&[1, 2, 3]);
        let reference = ReleaseCustodyHeadBlockReferenceV1::new(
            1,
            1,
            0,
            ReleaseCustodyHeadKeyV1::new([1; 16], 1).unwrap(),
            ReleaseCustodyHeadKeyV1::new([1; 16], 1).unwrap(),
            [1; 32],
        );
        let node = ReleaseCustodyHeadPathNodeV1::new(reference.unwrap(), frame);
        assert_eq!(node.owned_heap_bytes(), Some(32));
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReleaseCustodyHeadMutationV1 {
    Upsert {
        expected_prior: Option<ReleaseCustodyHeadEntryV1>,
        next: ReleaseCustodyHeadEntryV1,
    },
    RetireTerminal {
        expected_prior: ReleaseCustodyHeadEntryV1,
    },
}

impl ReleaseCustodyHeadMutationV1 {
    pub const fn key(self) -> ReleaseCustodyHeadKeyV1 {
        match self {
            Self::Upsert { next, .. } => next.key(),
            Self::RetireTerminal { expected_prior } => expected_prior.key(),
        }
    }
    pub const fn expected_prior(self) -> Option<ReleaseCustodyHeadEntryV1> {
        match self {
            Self::Upsert { expected_prior, .. } => expected_prior,
            Self::RetireTerminal { expected_prior } => Some(expected_prior),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReleaseCustodyHeadTransitionLimitsV1 {
    max_path_nodes: u16,
    max_new_blocks: u16,
    max_total_frame_bytes: u64,
}

impl ReleaseCustodyHeadTransitionLimitsV1 {
    pub fn new(
        max_path_nodes: u16,
        max_new_blocks: u16,
        max_total_frame_bytes: u64,
    ) -> Option<Self> {
        (max_path_nodes > 0 && max_new_blocks > 0 && max_total_frame_bytes > 0).then_some(Self {
            max_path_nodes,
            max_new_blocks,
            max_total_frame_bytes,
        })
    }
    pub const fn max_path_nodes(self) -> u16 {
        self.max_path_nodes
    }
    pub const fn max_new_blocks(self) -> u16 {
        self.max_new_blocks
    }
    pub const fn max_total_frame_bytes(self) -> u64 {
        self.max_total_frame_bytes
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleaseCustodyHeadTransitionV1 {
    source_root: Option<ReleaseCustodyHeadBlockReferenceV1>,
    source_next_block: u64,
    mutation: ReleaseCustodyHeadMutationV1,
    result_root: Option<ReleaseCustodyHeadBlockReferenceV1>,
    result_next_block: u64,
    writes: Vec<ReleaseCustodyHeadNodeWriteV1>,
    peak_frame_bytes: u64,
}

impl ReleaseCustodyHeadTransitionV1 {
    pub(crate) fn verification_additional_peak_bytes(
        source_path: &[ReleaseCustodyHeadPathNodeV1],
        claimed_new_blocks: usize,
        format: PhysicalRecordFormatDeclaration,
    ) -> Option<u64> {
        verification_budget::additional_peak_bytes(source_path, claimed_new_blocks, format)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn plan(
        source_root: Option<ReleaseCustodyHeadBlockReferenceV1>,
        source_next_block: u64,
        source_path: &[ReleaseCustodyHeadPathNodeV1],
        mutation: ReleaseCustodyHeadMutationV1,
        result_generation: u64,
        tree_identity: u64,
        format: PhysicalRecordFormatDeclaration,
        limits: ReleaseCustodyHeadTransitionLimitsV1,
    ) -> Result<Self, ReleaseCustodyHeadDenial> {
        plan::plan(
            source_root,
            source_next_block,
            source_path,
            mutation,
            result_generation,
            tree_identity,
            format,
            limits,
        )
    }

    /// Recompute from the exact WAL-carried source path; the caller separately
    /// proves that path belongs to selected media.
    #[allow(clippy::too_many_arguments)]
    pub fn verify_exact(
        source_root: Option<ReleaseCustodyHeadBlockReferenceV1>,
        source_next_block: u64,
        source_path: &[ReleaseCustodyHeadPathNodeV1],
        mutation: ReleaseCustodyHeadMutationV1,
        result_generation: u64,
        tree_identity: u64,
        format: PhysicalRecordFormatDeclaration,
        limits: ReleaseCustodyHeadTransitionLimitsV1,
        claimed_result_root: Option<ReleaseCustodyHeadBlockReferenceV1>,
        claimed_result_next_block: u64,
        claimed_writes: &[ReleaseCustodyHeadNodeWriteV1],
    ) -> Result<Self, ReleaseCustodyHeadDenial> {
        let planned = Self::plan(
            source_root,
            source_next_block,
            source_path,
            mutation,
            result_generation,
            tree_identity,
            format,
            limits,
        )?;
        if planned.result_root != claimed_result_root
            || planned.result_next_block != claimed_result_next_block
            || planned.writes != claimed_writes
        {
            return Err(ReleaseCustodyHeadDenial::Mutation);
        }
        Ok(planned)
    }

    pub const fn source_root(&self) -> Option<ReleaseCustodyHeadBlockReferenceV1> {
        self.source_root
    }
    pub const fn source_next_block(&self) -> u64 {
        self.source_next_block
    }
    pub const fn mutation(&self) -> ReleaseCustodyHeadMutationV1 {
        self.mutation
    }
    pub const fn result_root(&self) -> Option<ReleaseCustodyHeadBlockReferenceV1> {
        self.result_root
    }
    pub const fn result_next_block(&self) -> u64 {
        self.result_next_block
    }
    pub fn writes(&self) -> &[ReleaseCustodyHeadNodeWriteV1] {
        &self.writes
    }
    pub const fn peak_frame_bytes(&self) -> u64 {
        self.peak_frame_bytes
    }
    pub fn into_parts(
        self,
    ) -> (
        Option<ReleaseCustodyHeadBlockReferenceV1>,
        u64,
        ReleaseCustodyHeadMutationV1,
        Option<ReleaseCustodyHeadBlockReferenceV1>,
        u64,
        Vec<ReleaseCustodyHeadNodeWriteV1>,
        u64,
    ) {
        (
            self.source_root,
            self.source_next_block,
            self.mutation,
            self.result_root,
            self.result_next_block,
            self.writes,
            self.peak_frame_bytes,
        )
    }
}
