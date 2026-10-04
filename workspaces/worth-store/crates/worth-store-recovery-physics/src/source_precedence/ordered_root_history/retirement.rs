//! A root edge authorized by a checkpoint-covered retirement intent instead of
//! an admitted WAL member. An extent release writes its durable intent, then a
//! covering checkpoint, then the free-range root whose exact bytes the intent
//! names. The checkpoint retires that WAL, so no C.9 member exists for the
//! edge; the retained intent frame is its only authority.
//!
//! One decision owns the choice between a WAL member and such an intent.
//! Recovery planning and the Store rejoin both call it with their own sampled
//! evidence, so they cannot disagree about which basis an edge has.

use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    DurableFreeSpaceManifestHeader, DurablePhysicalRootManifest, PhysicalInventoryTranscriptV1,
    PhysicalRecordFormatDeclaration,
};
use worth_store_wal::WalLsnRange;

use super::VerifiedOrderedRootEdge;
use super::{OrderedRootHistoryBuilder, OrderedRootHistoryDenial as Denial};
use crate::source_precedence::{ordinary_root_step::transcript, ReleasedInventoryView};

/// One durable retirement-release intent sampled from retained WAL. This is
/// data, not authority: only [`decide_ordered_root_step_basis`] can turn it
/// into an edge basis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RetirementReleaseIntent {
    lsn: WalLsnRange,
    source_generation: u64,
    candidate_generation: u64,
    candidate_root_sha256: [u8; 32],
}

impl RetirementReleaseIntent {
    pub const fn new(
        lsn: WalLsnRange,
        source_generation: u64,
        candidate_generation: u64,
        candidate_root_sha256: [u8; 32],
    ) -> Self {
        Self {
            lsn,
            source_generation,
            candidate_generation,
            candidate_root_sha256,
        }
    }
    pub const fn lsn(self) -> WalLsnRange {
        self.lsn
    }
    pub const fn source_generation(self) -> u64 {
        self.source_generation
    }
    pub const fn candidate_generation(self) -> u64 {
        self.candidate_generation
    }
    pub const fn candidate_root_sha256(self) -> [u8; 32] {
        self.candidate_root_sha256
    }
    fn same_release(self, other: Self) -> bool {
        self.source_generation == other.source_generation
            && self.candidate_generation == other.candidate_generation
            && self.candidate_root_sha256 == other.candidate_root_sha256
    }
}

/// A retirement intent the shared decision admitted: its WAL lies at or below
/// the selected checkpoint's cutoff and it authorizes one edge of the
/// history's leading run of retirement edges.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CheckpointRetiredReleaseIntent {
    intent: RetirementReleaseIntent,
    checkpoint_cutoff: u64,
}

impl CheckpointRetiredReleaseIntent {
    pub const fn intent(self) -> RetirementReleaseIntent {
        self.intent
    }
    pub const fn checkpoint_cutoff(self) -> u64 {
        self.checkpoint_cutoff
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderedRootStepBasis {
    WalMember,
    RetirementIntent(CheckpointRetiredReleaseIntent),
}

/// Decides what authorizes the root edge leaving `source_generation`.
///
/// A retirement intent counts only when its WAL is at or below the checkpoint
/// cutoff and every earlier edge is itself a retirement edge: covered releases
/// publish before any replayed member. Above the cutoff the release WAL is
/// still live, so a missing member stays a denial. A member and a covered
/// intent for the same source root are ambiguous and deny.
pub fn decide_ordered_root_step_basis(
    source_generation: u64,
    retirement_prefix: bool,
    checkpoint_cutoff: u64,
    member_at_source: bool,
    intents: &[RetirementReleaseIntent],
) -> Result<OrderedRootStepBasis, Denial> {
    let mut covered: Option<RetirementReleaseIntent> = None;
    for intent in intents
        .iter()
        .filter(|intent| intent.source_generation == source_generation)
    {
        if intent.lsn.end_exclusive().get() > checkpoint_cutoff {
            continue;
        }
        match covered {
            None => covered = Some(*intent),
            Some(first) if first.same_release(*intent) => {}
            Some(_) => return Err(Denial::Source),
        }
    }
    match (member_at_source, covered) {
        (true, None) => Ok(OrderedRootStepBasis::WalMember),
        (true, Some(_)) => Err(Denial::Effect),
        (false, Some(intent))
            if retirement_prefix
                && source_generation.checked_add(1) == Some(intent.candidate_generation) =>
        {
            Ok(OrderedRootStepBasis::RetirementIntent(
                CheckpointRetiredReleaseIntent {
                    intent,
                    checkpoint_cutoff,
                },
            ))
        }
        (false, _) => Err(Denial::Source),
    }
}

/// The free-range root publication of one checkpoint-covered release. Routes
/// and segment membership are unchanged; the result root is byte-for-byte the
/// candidate the intent named, which also pins its free-space header.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VerifiedRetirementRootEdge {
    basis: CheckpointRetiredReleaseIntent,
    source: PhysicalInventoryTranscriptV1,
    result: PhysicalInventoryTranscriptV1,
}

impl VerifiedRetirementRootEdge {
    pub fn admit(
        source: ReleasedInventoryView<'_>,
        result: ReleasedInventoryView<'_>,
        basis: CheckpointRetiredReleaseIntent,
        format: PhysicalRecordFormatDeclaration,
        maximum_entries: u64,
    ) -> Result<Self, Denial> {
        let intent = basis.intent;
        let result_sha256: [u8; 32] = Sha256::digest(result.root.encode(format)).into();
        if source.root.generation() != intent.source_generation
            || result.root.generation() != intent.candidate_generation
            || result.free.generation() != intent.candidate_generation
            || result_sha256 != intent.candidate_root_sha256
        {
            return Err(Denial::Source);
        }
        if source.routes != result.routes || source.segments != result.segments {
            return Err(Denial::Effect);
        }
        Ok(Self {
            basis,
            source: transcript(source, format, maximum_entries).map_err(|_| Denial::Bound)?,
            result: transcript(result, format, maximum_entries).map_err(|_| Denial::Bound)?,
        })
    }

    /// Store replays the same predicate over its own reads of both roots and
    /// its own decision; any difference from the planned edge denies.
    pub fn recheck_actual_media(
        self,
        source: ReleasedInventoryView<'_>,
        result: ReleasedInventoryView<'_>,
        basis: CheckpointRetiredReleaseIntent,
        format: PhysicalRecordFormatDeclaration,
        maximum_entries: u64,
    ) -> Result<(), Denial> {
        if basis != self.basis {
            return Err(Denial::Source);
        }
        let checked = Self::admit(source, result, basis, format, maximum_entries)?;
        (checked == self).then_some(()).ok_or(Denial::Effect)
    }

    pub const fn basis(self) -> CheckpointRetiredReleaseIntent {
        self.basis
    }
    pub const fn source_topology(self) -> PhysicalInventoryTranscriptV1 {
        self.source
    }
    pub const fn result_topology(self) -> PhysicalInventoryTranscriptV1 {
        self.result
    }
}

impl VerifiedOrderedRootEdge {
    pub fn source_topology(&self) -> PhysicalInventoryTranscriptV1 {
        match self {
            Self::Ordinary(step) => step.source_topology(),
            Self::Released(edge) => edge.transition().source_topology(),
            Self::Retirement(edge) => edge.source_topology(),
        }
    }
    pub fn result_topology(&self) -> PhysicalInventoryTranscriptV1 {
        match self {
            Self::Ordinary(step) => step.result_topology(),
            Self::Released(edge) => edge.transition().result_topology(),
            Self::Retirement(edge) => edge.result_topology(),
        }
    }
    /// The edge's C.9 member operation; a retirement edge has none.
    pub fn operation(&self) -> Option<[u8; 32]> {
        match self {
            Self::Ordinary(step) => Some(step.operation()),
            Self::Released(edge) => Some(edge.operation()),
            Self::Retirement(_) => None,
        }
    }
    /// The edge's member WAL range; a retirement edge's WAL is checkpoint
    /// retired and orders before every member edge.
    pub fn member_lsn(&self) -> Option<WalLsnRange> {
        match self {
            Self::Ordinary(step) => step.lsn_range(),
            Self::Released(edge) => Some(edge.lsn()),
            Self::Retirement(_) => None,
        }
    }
}

/// Whether `edges` is empty or holds only retirement edges, so the next edge
/// may still be authorized by a checkpoint-covered intent.
pub fn is_retirement_prefix(edges: &[VerifiedOrderedRootEdge]) -> bool {
    edges
        .iter()
        .all(|edge| matches!(edge, VerifiedOrderedRootEdge::Retirement(_)))
}

impl OrderedRootHistoryBuilder {
    pub fn retirement_prefix(&self) -> bool {
        is_retirement_prefix(&self.edges)
    }

    pub fn advance_retirement(
        &mut self,
        edge: VerifiedRetirementRootEdge,
        result_root: &DurablePhysicalRootManifest,
        result_free: &DurableFreeSpaceManifestHeader,
        format: PhysicalRecordFormatDeclaration,
    ) -> Result<(), Denial> {
        let intent = edge.basis.intent;
        let result_sha256: [u8; 32] = Sha256::digest(result_root.encode(format)).into();
        // Retirement edges keep the WAL position at the cutoff and every member
        // edge moves it past, so cutoff equality also proves a retirement prefix.
        if edge.basis.checkpoint_cutoff != self.previous_lsn_end
            || intent.source_generation != self.current_generation
            || self.current_generation.checked_add(1) != Some(result_root.generation())
            || result_sha256 != intent.candidate_root_sha256
        {
            return Err(Denial::Source);
        }
        // Each covered release names the root its predecessor published, so
        // its intent follows that predecessor's intent in WAL order.
        if let Some(VerifiedOrderedRootEdge::Retirement(prior)) = self.edges.last() {
            if intent.lsn.start() < prior.basis.intent.lsn.end_exclusive() {
                return Err(Denial::WalOrder);
            }
        }
        if edge.source != self.current_topology
            || !edge
                .result
                .matches_headers(result_root, result_free, format)
        {
            return Err(Denial::Effect);
        }
        self.reserve_edge(0, 0)?;
        self.edges.push(VerifiedOrderedRootEdge::Retirement(edge));
        self.current_generation = result_root.generation();
        self.current_root_sha256 = result_sha256;
        self.current_free_space_sha256 = Sha256::digest(result_free.encode(format)).into();
        self.current_topology = edge.result;
        Ok(())
    }
}

#[cfg(test)]
#[path = "retirement_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "retirement_edge_tests.rs"]
mod edge_tests;
