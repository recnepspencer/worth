//! Checked lineage from a completed V3 result through later ordinary roots.
//! An addressed root is not authority merely because its frame is valid: every
//! successor must be the exact effect of a semantics-admitted C.9 WAL member.

use std::collections::BTreeSet;

use sha2::{Digest, Sha256};

use worth_store_physical_format::{
    DurableFreeSpaceManifestHeader, DurablePhysicalRootManifest,
    PersistedPhysicalRecoveryOperation, PhysicalInventoryTranscriptV1,
    PhysicalRecordFormatDeclaration,
};
use worth_store_wal::WalLsnRange;

use super::{
    VerifiedHistoricalReleaseRootPrefix, VerifiedOrdinaryRootStep,
    VerifiedReleasedV3InventoryTransition,
};
use crate::{AdmittedRootStepMemberView, PhysicalRedoGroupBinding, RecoveryOperationFate};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HistoricalReleaseRootChainDenial {
    InvalidFirstDrop,
    InvalidSuccessor,
    InvalidWalOrder,
    IncompleteChain,
    BoundExceeded,
}

/// Private-field proof of a complete, consecutive post-V3 root lineage.
/// Store still independently rereads the inventories and WAL before seal.
#[derive(Debug, Clone)]
pub struct VerifiedHistoricalReleaseRootChain {
    descriptor_operation: [u8; 32],
    source_root_generation: u64,
    source_root_frame_sha256: [u8; 32],
    first_result_generation: u64,
    first_result_root_frame_sha256: [u8; 32],
    selected_root_frame_sha256: [u8; 32],
    selected_free_frame_sha256: [u8; 32],
    source: PhysicalInventoryTranscriptV1,
    first_result: PhysicalInventoryTranscriptV1,
    first_transition: VerifiedReleasedV3InventoryTransition,
    selected: PhysicalInventoryTranscriptV1,
    first_lsn: WalLsnRange,
    first_group: PhysicalRedoGroupBinding,
    first_fate: RecoveryOperationFate,
    first_redo_sha256: [u8; 32],
    ordinary: Box<[VerifiedOrdinaryRootStep]>,
    checkpoint_prefix: Option<VerifiedHistoricalReleaseRootPrefix>,
    checkpoint_root_frame_sha256: Option<[u8; 32]>,
    scratch_bytes: u64,
}

/// Streaming construction retains checked edge tokens, not every root's
/// route/free inventory. The caller releases each addressed inventory after
/// the edge is admitted and charges the peak observation separately.
pub struct HistoricalReleaseRootChainBuilder {
    descriptor_operation: [u8; 32],
    source_root_generation: u64,
    source_root_frame_sha256: [u8; 32],
    first_result_generation: u64,
    first_result_root_frame_sha256: [u8; 32],
    source: PhysicalInventoryTranscriptV1,
    first_result: PhysicalInventoryTranscriptV1,
    first_transition: VerifiedReleasedV3InventoryTransition,
    previous: PhysicalInventoryTranscriptV1,
    generation: u64,
    first_lsn: WalLsnRange,
    first_group: PhysicalRedoGroupBinding,
    first_fate: RecoveryOperationFate,
    first_redo_sha256: [u8; 32],
    previous_lsn: WalLsnRange,
    operations: BTreeSet<[u8; 32]>,
    ordinary: Vec<VerifiedOrdinaryRootStep>,
    peak_scratch_bytes: u64,
    maximum_steps: u64,
    maximum_scratch_bytes: u64,
}

impl HistoricalReleaseRootChainBuilder {
    #[allow(clippy::too_many_arguments)]
    pub fn begin(
        first_member: AdmittedRootStepMemberView<'_>,
        first: VerifiedReleasedV3InventoryTransition,
        source_root: &DurablePhysicalRootManifest,
        source_free: &DurableFreeSpaceManifestHeader,
        first_result_root: &DurablePhysicalRootManifest,
        first_result_free: &DurableFreeSpaceManifestHeader,
        format: PhysicalRecordFormatDeclaration,
        maximum_steps: u64,
        maximum_scratch_bytes: u64,
    ) -> Result<Self, HistoricalReleaseRootChainDenial> {
        use HistoricalReleaseRootChainDenial as Denial;
        let PersistedPhysicalRecoveryOperation::RecordsDropped { binding, .. } =
            first_member.materialization().operation()
        else {
            return Err(Denial::InvalidFirstDrop);
        };
        if first_member.fate() == RecoveryOperationFate::ProvenNoEffect
            || first_member.materialization().source_root_generation() != source_root.generation()
            || binding.candidate_root_generation() != first_result_root.generation()
            || source_root.generation().checked_add(1) != Some(first_result_root.generation())
            || !first
                .source_topology()
                .matches_headers(source_root, source_free, format)
            || !first.result_topology().matches_headers(
                first_result_root,
                first_result_free,
                format,
            )
            || !first
                .projected()
                .iter()
                .any(|route| route.record() == binding.record())
            || first.scratch_bytes() > maximum_scratch_bytes
        {
            return Err(Denial::InvalidFirstDrop);
        }
        let source_topology = first.source_topology();
        let first_result_topology = first.result_topology();
        let first_scratch_bytes = first.scratch_bytes();
        Ok(Self {
            descriptor_operation: first_member.operation(),
            source_root_generation: source_root.generation(),
            source_root_frame_sha256: Sha256::digest(source_root.encode(format)).into(),
            first_result_generation: first_result_root.generation(),
            first_result_root_frame_sha256: Sha256::digest(first_result_root.encode(format)).into(),
            source: source_topology,
            first_result: first_result_topology,
            first_transition: first,
            previous: first_result_topology,
            generation: first_result_root.generation(),
            first_lsn: first_member.lsn_range(),
            first_group: first_member.group(),
            first_fate: first_member.fate(),
            first_redo_sha256: first_member.canonical_redo_sha256(),
            previous_lsn: first_member.lsn_range(),
            operations: BTreeSet::from([first_member.operation()]),
            ordinary: Vec::new(),
            peak_scratch_bytes: first_scratch_bytes,
            maximum_steps,
            maximum_scratch_bytes,
        })
    }

    pub fn advance(
        &mut self,
        member: AdmittedRootStepMemberView<'_>,
        transition: VerifiedOrdinaryRootStep,
        result_root: &DurablePhysicalRootManifest,
        result_free: &DurableFreeSpaceManifestHeader,
        format: PhysicalRecordFormatDeclaration,
    ) -> Result<(), HistoricalReleaseRootChainDenial> {
        use HistoricalReleaseRootChainDenial as Denial;
        if self.ordinary.len() as u64 >= self.maximum_steps {
            return Err(Denial::BoundExceeded);
        }
        if self.generation.checked_add(1) != Some(result_root.generation())
            || transition.source_topology() != self.previous
            || !transition
                .result_topology()
                .matches_headers(result_root, result_free, format)
            || transition.operation() != member.operation()
            || transition.group() != member.group()
            || transition.fate() != member.fate()
            || transition.redo_sha256() != member.canonical_redo_sha256()
            || transition.lsn_range() != Some(member.lsn_range())
            || self.operations.contains(&member.operation())
        {
            return Err(Denial::InvalidSuccessor);
        }
        if self.previous_lsn.end_exclusive() > member.lsn_range().start() {
            return Err(Denial::InvalidWalOrder);
        }
        let roster_bytes = (self.ordinary.len() as u64 + 1)
            .checked_mul(
                (std::mem::size_of::<VerifiedOrdinaryRootStep>()
                    + std::mem::size_of::<[u8; 32]>()
                    + 8 * std::mem::size_of::<usize>()) as u64,
            )
            .ok_or(Denial::BoundExceeded)?;
        let peak = self
            .peak_scratch_bytes
            .max(transition.scratch_bytes())
            .checked_add(roster_bytes)
            .ok_or(Denial::BoundExceeded)?;
        if peak > self.maximum_scratch_bytes {
            return Err(Denial::BoundExceeded);
        }
        self.ordinary
            .try_reserve(1)
            .map_err(|_| Denial::BoundExceeded)?;
        self.operations.insert(member.operation());
        self.ordinary.push(transition);
        self.previous = transition.result_topology();
        self.generation = result_root.generation();
        self.previous_lsn = member.lsn_range();
        self.peak_scratch_bytes = self.peak_scratch_bytes.max(transition.scratch_bytes());
        Ok(())
    }

    pub fn finish(
        self,
        selected_root: &DurablePhysicalRootManifest,
        selected_free: &DurableFreeSpaceManifestHeader,
        format: PhysicalRecordFormatDeclaration,
    ) -> Result<VerifiedHistoricalReleaseRootChain, HistoricalReleaseRootChainDenial> {
        use HistoricalReleaseRootChainDenial as Denial;
        if self.generation != selected_root.generation()
            || !self
                .previous
                .matches_headers(selected_root, selected_free, format)
        {
            return Err(Denial::IncompleteChain);
        }
        let roster_bytes = (self.ordinary.len() as u64)
            .checked_mul(
                (std::mem::size_of::<VerifiedOrdinaryRootStep>()
                    + std::mem::size_of::<[u8; 32]>()
                    + 8 * std::mem::size_of::<usize>()) as u64,
            )
            .ok_or(Denial::BoundExceeded)?;
        let scratch_bytes = self
            .peak_scratch_bytes
            .checked_add(roster_bytes)
            .ok_or(Denial::BoundExceeded)?;
        if scratch_bytes > self.maximum_scratch_bytes {
            return Err(Denial::BoundExceeded);
        }
        Ok(VerifiedHistoricalReleaseRootChain {
            descriptor_operation: self.descriptor_operation,
            source_root_generation: self.source_root_generation,
            source_root_frame_sha256: self.source_root_frame_sha256,
            first_result_generation: self.first_result_generation,
            first_result_root_frame_sha256: self.first_result_root_frame_sha256,
            selected_root_frame_sha256: Sha256::digest(selected_root.encode(format)).into(),
            selected_free_frame_sha256: Sha256::digest(selected_free.encode(format)).into(),
            source: self.source,
            first_result: self.first_result,
            first_transition: self.first_transition,
            selected: self.previous,
            first_lsn: self.first_lsn,
            first_group: self.first_group,
            first_fate: self.first_fate,
            first_redo_sha256: self.first_redo_sha256,
            ordinary: self.ordinary.into_boxed_slice(),
            checkpoint_prefix: None,
            checkpoint_root_frame_sha256: None,
            scratch_bytes,
        })
    }
}

impl VerifiedHistoricalReleaseRootChain {
    pub fn owned_heap_bytes(&self) -> Option<u64> {
        let ordinary = u64::try_from(self.ordinary.len())
            .ok()?
            .checked_mul(u64::try_from(std::mem::size_of::<VerifiedOrdinaryRootStep>()).ok()?)?;
        ordinary
            .checked_add(self.first_transition.owned_heap_bytes()?)?
            .checked_add(self.checkpoint_prefix.as_ref().map_or(
                Some(0),
                VerifiedHistoricalReleaseRootPrefix::owned_heap_bytes,
            )?)
    }

    pub fn bind_checkpoint_prefix(
        mut self,
        prefix: VerifiedHistoricalReleaseRootPrefix,
        selected_checkpoint_source_sha256: [u8; 32],
        maximum_scratch_bytes: u64,
    ) -> Result<Self, HistoricalReleaseRootChainDenial> {
        use HistoricalReleaseRootChainDenial as Denial;
        if self.checkpoint_root_frame_sha256.is_some()
            || prefix.checkpoint_root_frame_sha256() != selected_checkpoint_source_sha256
            || prefix.result_topology() != self.source
            || prefix
                .checkpoint_generation()
                .checked_add(prefix.steps().len() as u64)
                != Some(self.source_root_generation)
            || prefix.last_lsn().end_exclusive() > self.first_lsn.start()
        {
            return Err(Denial::IncompleteChain);
        }
        let mut operations = BTreeSet::from([self.descriptor_operation]);
        operations.extend(self.ordinary.iter().map(|step| step.operation()));
        if prefix
            .steps()
            .iter()
            .any(|step| !operations.insert(step.operation()))
        {
            return Err(Denial::InvalidWalOrder);
        }
        let scratch = self
            .scratch_bytes
            .max(prefix.scratch_bytes())
            .checked_add(
                (prefix.steps().len() as u64)
                    .checked_mul(std::mem::size_of::<VerifiedOrdinaryRootStep>() as u64)
                    .ok_or(Denial::BoundExceeded)?,
            )
            .ok_or(Denial::BoundExceeded)?;
        if scratch > maximum_scratch_bytes {
            return Err(Denial::BoundExceeded);
        }
        self.scratch_bytes = scratch;
        self.checkpoint_root_frame_sha256 = Some(selected_checkpoint_source_sha256);
        self.checkpoint_prefix = Some(prefix);
        Ok(self)
    }

    pub fn bind_direct_checkpoint(
        mut self,
        checkpoint_root: &DurablePhysicalRootManifest,
        checkpoint_free: &DurableFreeSpaceManifestHeader,
        selected_checkpoint_source_sha256: [u8; 32],
        format: PhysicalRecordFormatDeclaration,
    ) -> Result<Self, HistoricalReleaseRootChainDenial> {
        use HistoricalReleaseRootChainDenial as Denial;
        if self.checkpoint_root_frame_sha256.is_some()
            || checkpoint_root.generation() != self.source_root_generation
            || selected_checkpoint_source_sha256 != self.source_root_frame_sha256
            || !self
                .source
                .matches_headers(checkpoint_root, checkpoint_free, format)
        {
            return Err(Denial::IncompleteChain);
        }
        // An empty prefix is represented by the exact selected checkpoint
        // source root SHA and the source transcript itself.
        self.checkpoint_root_frame_sha256 = Some(selected_checkpoint_source_sha256);
        Ok(self)
    }

    pub const fn checkpoint_prefix(&self) -> Option<&VerifiedHistoricalReleaseRootPrefix> {
        self.checkpoint_prefix.as_ref()
    }
    pub const fn checkpoint_root_frame_sha256(&self) -> Option<[u8; 32]> {
        self.checkpoint_root_frame_sha256
    }

    pub const fn descriptor_operation(&self) -> [u8; 32] {
        self.descriptor_operation
    }
    pub const fn source_root_generation(&self) -> u64 {
        self.source_root_generation
    }
    pub const fn source_root_frame_sha256(&self) -> [u8; 32] {
        self.source_root_frame_sha256
    }
    pub const fn first_result_generation(&self) -> u64 {
        self.first_result_generation
    }
    pub const fn first_result_root_frame_sha256(&self) -> [u8; 32] {
        self.first_result_root_frame_sha256
    }
    pub const fn selected_root_frame_sha256(&self) -> [u8; 32] {
        self.selected_root_frame_sha256
    }
    pub const fn selected_free_frame_sha256(&self) -> [u8; 32] {
        self.selected_free_frame_sha256
    }
    pub const fn source_topology(&self) -> PhysicalInventoryTranscriptV1 {
        self.source
    }
    pub const fn first_result_topology(&self) -> PhysicalInventoryTranscriptV1 {
        self.first_result
    }
    pub fn first_transition(&self) -> &VerifiedReleasedV3InventoryTransition {
        &self.first_transition
    }
    pub const fn selected_topology(&self) -> PhysicalInventoryTranscriptV1 {
        self.selected
    }
    pub const fn first_lsn(&self) -> WalLsnRange {
        self.first_lsn
    }
    pub const fn first_group(&self) -> PhysicalRedoGroupBinding {
        self.first_group
    }
    pub const fn first_fate(&self) -> RecoveryOperationFate {
        self.first_fate
    }
    pub const fn first_redo_sha256(&self) -> [u8; 32] {
        self.first_redo_sha256
    }
    pub fn ordinary_steps(&self) -> &[VerifiedOrdinaryRootStep] {
        &self.ordinary
    }
    pub const fn scratch_bytes(&self) -> u64 {
        self.scratch_bytes
    }
}
