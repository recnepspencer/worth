//! Checkpoint-anchored ordering of exact, semantics-admitted root effects.
//! This is a planning lineage, not release custody: every released edge still
//! requires its own selected/addressed controls, C.9 fate, and Store rejoin.

use std::collections::BTreeSet;

use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    DurableFreeSpaceManifestHeader, DurablePhysicalRootManifest,
    PersistedPhysicalRecoveryOperation, PersistedRecordIdentity, PhysicalInventoryTranscriptV1,
    PhysicalRecordFormatDeclaration,
};
use worth_store_wal::WalLsnRange;

use super::{VerifiedOrdinaryRootStep, VerifiedReleasedV3InventoryTransition};
use crate::{AdmittedRootStepMemberView, PhysicalRedoGroupBinding, RecoveryOperationFate};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderedRootHistoryDenial {
    Source,
    Effect,
    WalOrder,
    Bound,
    Incomplete,
}

#[derive(Debug, Clone)]
pub struct VerifiedReleasedRootEdge {
    transition: VerifiedReleasedV3InventoryTransition,
    operation: [u8; 32],
    group: PhysicalRedoGroupBinding,
    fate: RecoveryOperationFate,
    lsn: WalLsnRange,
    redo_sha256: [u8; 32],
    descriptor_record: PersistedRecordIdentity,
    descriptor_frame_sha256: [u8; 32],
    candidate_root_generation: u64,
    source_root_frame_sha256: [u8; 32],
    source_free_space_frame_sha256: [u8; 32],
    result_root_frame_sha256: [u8; 32],
}

impl VerifiedReleasedRootEdge {
    pub fn transition(&self) -> &VerifiedReleasedV3InventoryTransition {
        &self.transition
    }
    pub const fn operation(&self) -> [u8; 32] {
        self.operation
    }
    pub const fn group(&self) -> PhysicalRedoGroupBinding {
        self.group
    }
    pub const fn fate(&self) -> RecoveryOperationFate {
        self.fate
    }
    pub const fn lsn(&self) -> WalLsnRange {
        self.lsn
    }
    pub const fn redo_sha256(&self) -> [u8; 32] {
        self.redo_sha256
    }
    pub const fn descriptor_record(&self) -> PersistedRecordIdentity {
        self.descriptor_record
    }
    pub const fn descriptor_frame_sha256(&self) -> [u8; 32] {
        self.descriptor_frame_sha256
    }
    pub const fn candidate_root_generation(&self) -> u64 {
        self.candidate_root_generation
    }
    pub const fn result_root_frame_sha256(&self) -> [u8; 32] {
        self.result_root_frame_sha256
    }
    pub const fn source_root_frame_sha256(&self) -> [u8; 32] {
        self.source_root_frame_sha256
    }
    pub const fn source_free_space_frame_sha256(&self) -> [u8; 32] {
        self.source_free_space_frame_sha256
    }
}

#[derive(Debug, Clone)]
pub enum VerifiedOrderedRootEdge {
    Ordinary(VerifiedOrdinaryRootStep),
    Released(VerifiedReleasedRootEdge),
}

#[derive(Debug, Clone)]
pub struct VerifiedOrderedRootHistory {
    checkpoint_root_frame_sha256: [u8; 32],
    selected_root_frame_sha256: [u8; 32],
    selected_free_space_frame_sha256: [u8; 32],
    selected_topology: PhysicalInventoryTranscriptV1,
    edges: Box<[VerifiedOrderedRootEdge]>,
    peak_scratch_bytes: u64,
}

impl VerifiedOrderedRootHistory {
    pub fn owned_heap_bytes(&self) -> Option<u64> {
        let edges = u64::try_from(self.edges.len())
            .ok()?
            .checked_mul(u64::try_from(std::mem::size_of::<VerifiedOrderedRootEdge>()).ok()?)?;
        self.edges.iter().try_fold(edges, |sum, edge| match edge {
            VerifiedOrderedRootEdge::Ordinary(_) => Some(sum),
            VerifiedOrderedRootEdge::Released(released) => {
                sum.checked_add(released.transition.owned_heap_bytes()?)
            }
        })
    }

    pub const fn checkpoint_root_frame_sha256(&self) -> [u8; 32] {
        self.checkpoint_root_frame_sha256
    }
    pub const fn selected_root_frame_sha256(&self) -> [u8; 32] {
        self.selected_root_frame_sha256
    }
    pub const fn selected_free_space_frame_sha256(&self) -> [u8; 32] {
        self.selected_free_space_frame_sha256
    }
    pub const fn selected_topology(&self) -> PhysicalInventoryTranscriptV1 {
        self.selected_topology
    }
    pub fn edges(&self) -> &[VerifiedOrderedRootEdge] {
        &self.edges
    }
    pub const fn peak_scratch_bytes(&self) -> u64 {
        self.peak_scratch_bytes
    }
}

pub struct OrderedRootHistoryBuilder {
    checkpoint_root_frame_sha256: [u8; 32],
    current_root_sha256: [u8; 32],
    current_free_space_sha256: [u8; 32],
    current_topology: PhysicalInventoryTranscriptV1,
    current_generation: u64,
    previous_lsn_end: u64,
    operations: BTreeSet<[u8; 32]>,
    edges: Vec<VerifiedOrderedRootEdge>,
    retained_projected_bytes: u64,
    peak_effect_scratch_bytes: u64,
    peak_scratch_bytes: u64,
    maximum_edges: u64,
    maximum_scratch_bytes: u64,
}

impl OrderedRootHistoryBuilder {
    pub fn begin(
        checkpoint_root: &DurablePhysicalRootManifest,
        checkpoint_free: &DurableFreeSpaceManifestHeader,
        checkpoint_root_frame_sha256: [u8; 32],
        checkpoint_wal_cutoff: u64,
        checkpoint_topology: PhysicalInventoryTranscriptV1,
        format: PhysicalRecordFormatDeclaration,
        maximum_edges: u64,
        maximum_scratch_bytes: u64,
    ) -> Result<Self, OrderedRootHistoryDenial> {
        let root_sha256: [u8; 32] = Sha256::digest(checkpoint_root.encode(format)).into();
        if root_sha256 != checkpoint_root_frame_sha256
            || !checkpoint_topology.matches_headers(checkpoint_root, checkpoint_free, format)
            || maximum_edges == 0
            || maximum_scratch_bytes == 0
        {
            return Err(OrderedRootHistoryDenial::Source);
        }
        Ok(Self {
            checkpoint_root_frame_sha256,
            current_root_sha256: root_sha256,
            current_free_space_sha256: Sha256::digest(checkpoint_free.encode(format)).into(),
            current_topology: checkpoint_topology,
            current_generation: checkpoint_root.generation(),
            previous_lsn_end: checkpoint_wal_cutoff,
            operations: BTreeSet::new(),
            edges: Vec::new(),
            retained_projected_bytes: 0,
            peak_effect_scratch_bytes: 0,
            peak_scratch_bytes: 0,
            maximum_edges,
            maximum_scratch_bytes,
        })
    }

    pub fn advance_ordinary(
        &mut self,
        member: AdmittedRootStepMemberView<'_>,
        step: VerifiedOrdinaryRootStep,
        result_root: &DurablePhysicalRootManifest,
        result_free: &DurableFreeSpaceManifestHeader,
        format: PhysicalRecordFormatDeclaration,
    ) -> Result<(), OrderedRootHistoryDenial> {
        let result = step.result_topology();
        if step.source_topology() != self.current_topology
            || step.operation() != member.operation()
            || step.group() != member.group()
            || step.fate() != member.fate()
            || step.redo_sha256() != member.canonical_redo_sha256()
            || step.lsn_range() != Some(member.lsn_range())
            || !result.matches_headers(result_root, result_free, format)
        {
            return Err(OrderedRootHistoryDenial::Effect);
        }
        self.check_edge(member, result_root, step.scratch_bytes(), 0)?;
        self.edges.push(VerifiedOrderedRootEdge::Ordinary(step));
        self.advance_state(member, result_root, result_free, result, format);
        Ok(())
    }

    pub fn advance_released(
        &mut self,
        member: AdmittedRootStepMemberView<'_>,
        transition: VerifiedReleasedV3InventoryTransition,
        result_root: &DurablePhysicalRootManifest,
        result_free: &DurableFreeSpaceManifestHeader,
        format: PhysicalRecordFormatDeclaration,
    ) -> Result<&VerifiedReleasedRootEdge, OrderedRootHistoryDenial> {
        let PersistedPhysicalRecoveryOperation::RecordsDropped { binding, .. } =
            member.materialization().operation()
        else {
            return Err(OrderedRootHistoryDenial::Effect);
        };
        let result = transition.result_topology();
        if transition.source_topology() != self.current_topology
            || binding.candidate_root_generation() != result_root.generation()
            || !transition
                .projected()
                .iter()
                .any(|route| route.record() == binding.record())
            || !result.matches_headers(result_root, result_free, format)
            || member.fate() == RecoveryOperationFate::ProvenNoEffect
        {
            return Err(OrderedRootHistoryDenial::Effect);
        }
        let retained = (transition.projected().len() as u64)
            .checked_mul(std::mem::size_of::<
                worth_store_physical_format::CurrentPhysicalRecordPlacement,
            >() as u64)
            .ok_or(OrderedRootHistoryDenial::Bound)?;
        self.check_edge(member, result_root, transition.scratch_bytes(), retained)?;
        self.edges.push(VerifiedOrderedRootEdge::Released(
            VerifiedReleasedRootEdge {
                transition,
                operation: member.operation(),
                group: member.group(),
                fate: member.fate(),
                lsn: member.lsn_range(),
                redo_sha256: member.canonical_redo_sha256(),
                descriptor_record: binding.record(),
                descriptor_frame_sha256: binding.record_payload_sha256(),
                candidate_root_generation: result_root.generation(),
                source_root_frame_sha256: self.current_root_sha256,
                source_free_space_frame_sha256: self.current_free_space_sha256,
                result_root_frame_sha256: Sha256::digest(result_root.encode(format)).into(),
            },
        ));
        self.advance_state(member, result_root, result_free, result, format);
        match self.edges.last() {
            Some(VerifiedOrderedRootEdge::Released(edge)) => Ok(edge),
            _ => unreachable!("just appended a released edge"),
        }
    }

    fn check_edge(
        &mut self,
        member: AdmittedRootStepMemberView<'_>,
        result_root: &DurablePhysicalRootManifest,
        effect_scratch: u64,
        retained_projected_increment: u64,
    ) -> Result<(), OrderedRootHistoryDenial> {
        if self.current_generation.checked_add(1) != Some(result_root.generation())
            || member.materialization().source_root_generation() != self.current_generation
        {
            return Err(OrderedRootHistoryDenial::Source);
        }
        if member.lsn_range().start().get() < self.previous_lsn_end
            || self.operations.contains(&member.operation())
        {
            return Err(OrderedRootHistoryDenial::WalOrder);
        }
        let count = self
            .edges
            .len()
            .checked_add(1)
            .ok_or(OrderedRootHistoryDenial::Bound)?;
        let roster_bytes = (count as u64)
            .checked_mul(
                (std::mem::size_of::<VerifiedOrderedRootEdge>()
                    + std::mem::size_of::<[u8; 32]>()
                    + 8 * std::mem::size_of::<usize>()) as u64,
            )
            .ok_or(OrderedRootHistoryDenial::Bound)?;
        let effect_peak = self.peak_effect_scratch_bytes.max(effect_scratch);
        let retained_projected = self
            .retained_projected_bytes
            .checked_add(retained_projected_increment)
            .ok_or(OrderedRootHistoryDenial::Bound)?;
        let peak = effect_peak
            .checked_add(roster_bytes)
            .and_then(|bytes| bytes.checked_add(retained_projected))
            .ok_or(OrderedRootHistoryDenial::Bound)?;
        if count as u64 > self.maximum_edges || peak > self.maximum_scratch_bytes {
            return Err(OrderedRootHistoryDenial::Bound);
        }
        self.edges
            .try_reserve(1)
            .map_err(|_| OrderedRootHistoryDenial::Bound)?;
        self.retained_projected_bytes = retained_projected;
        self.peak_effect_scratch_bytes = effect_peak;
        self.peak_scratch_bytes = peak;
        Ok(())
    }

    fn advance_state(
        &mut self,
        member: AdmittedRootStepMemberView<'_>,
        result_root: &DurablePhysicalRootManifest,
        result_free: &DurableFreeSpaceManifestHeader,
        result: PhysicalInventoryTranscriptV1,
        format: PhysicalRecordFormatDeclaration,
    ) {
        self.operations.insert(member.operation());
        self.previous_lsn_end = member.lsn_range().end_exclusive().get();
        self.current_generation = result_root.generation();
        self.current_root_sha256 = Sha256::digest(result_root.encode(format)).into();
        self.current_free_space_sha256 = Sha256::digest(result_free.encode(format)).into();
        self.current_topology = result;
    }

    pub fn finish(
        self,
        selected_root: &DurablePhysicalRootManifest,
        selected_free: &DurableFreeSpaceManifestHeader,
        format: PhysicalRecordFormatDeclaration,
    ) -> Result<VerifiedOrderedRootHistory, OrderedRootHistoryDenial> {
        let selected_sha256: [u8; 32] = Sha256::digest(selected_root.encode(format)).into();
        let selected_free_sha256: [u8; 32] = Sha256::digest(selected_free.encode(format)).into();
        if self.edges.is_empty()
            || self.current_generation != selected_root.generation()
            || self.current_root_sha256 != selected_sha256
            || self.current_free_space_sha256 != selected_free_sha256
            || !self
                .current_topology
                .matches_headers(selected_root, selected_free, format)
        {
            return Err(OrderedRootHistoryDenial::Incomplete);
        }
        Ok(VerifiedOrderedRootHistory {
            checkpoint_root_frame_sha256: self.checkpoint_root_frame_sha256,
            selected_root_frame_sha256: selected_sha256,
            selected_free_space_frame_sha256: selected_free_sha256,
            selected_topology: self.current_topology,
            edges: self.edges.into_boxed_slice(),
            peak_scratch_bytes: self.peak_scratch_bytes,
        })
    }
}
