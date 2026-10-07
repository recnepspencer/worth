//! Checkpoint-source to first V3-source lineage, built from complete ordinary
//! C.9 root steps. This closes the otherwise orphan addressed-source gap.

use std::collections::BTreeSet;

use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    DurableFreeSpaceManifestHeader, DurablePhysicalRootManifest, PhysicalInventoryTranscriptV1,
    PhysicalRecordFormatDeclaration,
};
use worth_store_wal::WalLsnRange;

use super::{HistoricalReleaseRootChainDenial as Denial, VerifiedOrdinaryRootStep};
use crate::AdmittedRootStepMemberView;

#[derive(Debug, Clone)]
pub struct VerifiedHistoricalReleaseRootPrefix {
    checkpoint_root_frame_sha256: [u8; 32],
    checkpoint_generation: u64,
    first: PhysicalInventoryTranscriptV1,
    result: PhysicalInventoryTranscriptV1,
    first_lsn: WalLsnRange,
    last_lsn: WalLsnRange,
    steps: Box<[VerifiedOrdinaryRootStep]>,
    scratch_bytes: u64,
}

pub struct HistoricalReleaseRootPrefixBuilder {
    checkpoint_root_frame_sha256: [u8; 32],
    checkpoint_generation: u64,
    checkpoint_free: DurableFreeSpaceManifestHeader,
    checkpoint_root: DurablePhysicalRootManifest,
    first: Option<PhysicalInventoryTranscriptV1>,
    previous: Option<PhysicalInventoryTranscriptV1>,
    generation: u64,
    first_lsn: Option<WalLsnRange>,
    last_lsn: Option<WalLsnRange>,
    operations: BTreeSet<[u8; 32]>,
    steps: Vec<VerifiedOrdinaryRootStep>,
    peak_scratch_bytes: u64,
    maximum_steps: u64,
    maximum_scratch_bytes: u64,
}

impl HistoricalReleaseRootPrefixBuilder {
    pub fn begin(
        checkpoint_root: &DurablePhysicalRootManifest,
        checkpoint_free: &DurableFreeSpaceManifestHeader,
        checkpoint_root_frame_sha256: [u8; 32],
        format: PhysicalRecordFormatDeclaration,
        maximum_steps: u64,
        maximum_scratch_bytes: u64,
    ) -> Result<Self, Denial> {
        if maximum_steps == 0
            || <[u8; 32]>::from(Sha256::digest(checkpoint_root.encode(format)))
                != checkpoint_root_frame_sha256
        {
            return Err(Denial::InvalidSuccessor);
        }
        Ok(Self {
            checkpoint_root_frame_sha256,
            checkpoint_generation: checkpoint_root.generation(),
            checkpoint_free: checkpoint_free.clone(),
            checkpoint_root: checkpoint_root.clone(),
            first: None,
            previous: None,
            generation: checkpoint_root.generation(),
            first_lsn: None,
            last_lsn: None,
            operations: BTreeSet::new(),
            steps: Vec::new(),
            peak_scratch_bytes: 0,
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
    ) -> Result<(), Denial> {
        if self.steps.len() as u64 >= self.maximum_steps {
            return Err(Denial::BoundExceeded);
        }
        let source_matches = self.previous.map_or_else(
            || {
                transition.source_topology().matches_headers(
                    &self.checkpoint_root,
                    &self.checkpoint_free,
                    format,
                )
            },
            |previous| transition.source_topology() == previous,
        );
        if !source_matches
            || self.generation.checked_add(1) != Some(result_root.generation())
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
        if self
            .last_lsn
            .is_some_and(|previous| previous.end_exclusive() > member.lsn_range().start())
        {
            return Err(Denial::InvalidWalOrder);
        }
        let roster_bytes = (self.steps.len() as u64 + 1)
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
        self.steps
            .try_reserve(1)
            .map_err(|_| Denial::BoundExceeded)?;
        self.first.get_or_insert(transition.source_topology());
        self.first_lsn.get_or_insert(member.lsn_range());
        self.last_lsn = Some(member.lsn_range());
        self.previous = Some(transition.result_topology());
        self.generation = result_root.generation();
        self.peak_scratch_bytes = self.peak_scratch_bytes.max(transition.scratch_bytes());
        self.operations.insert(member.operation());
        self.steps.push(transition);
        Ok(())
    }

    pub fn finish(
        self,
        first_v3_source_root: &DurablePhysicalRootManifest,
        first_v3_source_free: &DurableFreeSpaceManifestHeader,
        format: PhysicalRecordFormatDeclaration,
    ) -> Result<VerifiedHistoricalReleaseRootPrefix, Denial> {
        if self.generation != first_v3_source_root.generation()
            || !self.previous.is_some_and(|previous| {
                previous.matches_headers(first_v3_source_root, first_v3_source_free, format)
            })
        {
            return Err(Denial::IncompleteChain);
        }
        let roster_bytes = (self.steps.len() as u64)
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
        Ok(VerifiedHistoricalReleaseRootPrefix {
            checkpoint_root_frame_sha256: self.checkpoint_root_frame_sha256,
            checkpoint_generation: self.checkpoint_generation,
            first: self.first.ok_or(Denial::IncompleteChain)?,
            result: self.previous.ok_or(Denial::IncompleteChain)?,
            first_lsn: self.first_lsn.ok_or(Denial::IncompleteChain)?,
            last_lsn: self.last_lsn.ok_or(Denial::IncompleteChain)?,
            steps: self.steps.into_boxed_slice(),
            scratch_bytes,
        })
    }
}

impl VerifiedHistoricalReleaseRootPrefix {
    pub fn owned_heap_bytes(&self) -> Option<u64> {
        u64::try_from(self.steps.len())
            .ok()?
            .checked_mul(u64::try_from(std::mem::size_of::<VerifiedOrdinaryRootStep>()).ok()?)
    }

    pub const fn checkpoint_root_frame_sha256(&self) -> [u8; 32] {
        self.checkpoint_root_frame_sha256
    }
    pub const fn checkpoint_generation(&self) -> u64 {
        self.checkpoint_generation
    }
    pub const fn first_topology(&self) -> PhysicalInventoryTranscriptV1 {
        self.first
    }
    pub const fn result_topology(&self) -> PhysicalInventoryTranscriptV1 {
        self.result
    }
    pub const fn first_lsn(&self) -> WalLsnRange {
        self.first_lsn
    }
    pub const fn last_lsn(&self) -> WalLsnRange {
        self.last_lsn
    }
    pub fn steps(&self) -> &[VerifiedOrdinaryRootStep] {
        &self.steps
    }
    pub const fn scratch_bytes(&self) -> u64 {
        self.scratch_bytes
    }
}
