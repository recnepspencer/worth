//! Dense meaning tables retain every original open and historical occurrence.
use super::WorldCheckpointCoverage;
use crate::durability::{
    AdmittedWorldDisposition, AdmittedWorldOccurrence, AdmittedWorldOpenAttempt,
    DecodedWorldRecord, DurableWorldHead, WorldDurableHomeBinding, WorldIdentityFrontiers,
    WorldReadmissionDenial as Denial, WorldRecord,
};
use std::{cmp::Ordering, sync::Arc};
use worth_execution::ExecutionOrderedMap;
use worth_foundational::facade::{
    DurableCompositeCommitIdentity, DurableCompositePublicationAttemptIdentity,
    DurableProductBranchIdentity, DurableProductBranchIncarnation,
    DurableProductReferenceGeneration,
};

#[derive(Clone, Copy, Debug)]
pub(crate) struct WorldOwnerRecordLocation {
    pub(crate) position: u64,
    pub(crate) digest: [u8; 32],
}
/// Comparing this immutable key borrows the actual funded branch text.
#[derive(Clone, Debug)]
pub(crate) struct WorldBranchImageKey {
    record: Arc<DecodedWorldRecord>,
    pub(crate) position: u64,
    pub(crate) digest: [u8; 32],
}
impl WorldBranchImageKey {
    pub(crate) fn new(record: Arc<DecodedWorldRecord>, position: u64, digest: [u8; 32]) -> Self {
        Self {
            record,
            position,
            digest,
        }
    }
    pub(crate) fn record(&self) -> &Arc<DecodedWorldRecord> {
        &self.record
    }
    pub(crate) fn branch(&self) -> &DurableProductBranchIdentity {
        branch_fields(&self.record).0
    }
}
impl PartialEq for WorldBranchImageKey {
    fn eq(&self, other: &Self) -> bool {
        self.branch() == other.branch()
    }
}
impl Eq for WorldBranchImageKey {}
impl PartialOrd for WorldBranchImageKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for WorldBranchImageKey {
    fn cmp(&self, other: &Self) -> Ordering {
        self.branch().cmp(other.branch())
    }
}

#[derive(Clone, Debug)]
pub(crate) struct WorldImageHead {
    record: Arc<DecodedWorldRecord>,
    generation: DurableProductReferenceGeneration,
}
impl WorldImageHead {
    pub(crate) fn branch(record: Arc<DecodedWorldRecord>) -> Self {
        let WorldRecord::Branch { head, .. } = &record.record else {
            unreachable!("branch origin")
        };
        let generation = head.generation;
        Self { record, generation }
    }
    pub(crate) fn occurrence(record: Arc<DecodedWorldRecord>) -> Result<Self, Denial> {
        let WorldRecord::OpenAttempt(open) = &record.record else {
            unreachable!("occurrence head")
        };
        let next = open
            .expected
            .generation
            .get()
            .checked_add(1)
            .ok_or(Denial::BranchLineage)?;
        Ok(Self {
            record,
            generation: DurableProductReferenceGeneration::from_recorded(next),
        })
    }
    pub(in crate::durability) fn program_record(&self) -> &DecodedWorldRecord {
        &self.record
    }
    pub(crate) fn fields(
        &self,
    ) -> (
        &DurableProductBranchIdentity,
        DurableProductBranchIncarnation,
        DurableCompositeCommitIdentity,
        DurableProductReferenceGeneration,
    ) {
        let (branch, incarnation, commit) = branch_fields(&self.record);
        (branch, incarnation, commit, self.generation)
    }
    pub(crate) fn matches(&self, expected: &DurableWorldHead) -> bool {
        let (branch, incarnation, commit) = branch_fields(&self.record);
        branch == &expected.branch
            && incarnation == expected.incarnation
            && commit == expected.commit
            && self.generation == expected.generation
    }
    pub(crate) fn represents_open(
        &self,
        expected: &crate::durability::WorldOpenAttemptRecord,
    ) -> bool {
        matches!(&self.record.record, WorldRecord::OpenAttempt(open) if open == expected)
    }
}
fn branch_fields(
    record: &DecodedWorldRecord,
) -> (
    &DurableProductBranchIdentity,
    DurableProductBranchIncarnation,
    DurableCompositeCommitIdentity,
) {
    match &record.record {
        WorldRecord::Branch { head, .. } => (&head.branch, head.incarnation, head.commit),
        WorldRecord::OpenAttempt(open) => (
            &open.expected.branch,
            open.expected.incarnation,
            open.commit,
        ),
        _ => unreachable!("only a branch origin or occurrence owns a head"),
    }
}

pub(crate) struct WorldCheckpointImage {
    pub(crate) binding: WorldDurableHomeBinding,
    pub(crate) coverage: WorldCheckpointCoverage,
    pub(crate) owner: Option<WorldOwnerRecordLocation>,
    pub(crate) frontiers: WorldIdentityFrontiers,
    pub(crate) branches: ExecutionOrderedMap<WorldBranchImageKey, ()>,
    pub(crate) occurrences: ExecutionOrderedMap<u64, AdmittedWorldOccurrence>,
    pub(crate) history: ExecutionOrderedMap<DurableCompositeCommitIdentity, WorldImageHead>,
    pub(crate) heads: ExecutionOrderedMap<WorldBranchImageKey, WorldImageHead>,
    pub(crate) opens:
        ExecutionOrderedMap<DurableCompositePublicationAttemptIdentity, AdmittedWorldOpenAttempt>,
    pub(crate) dispositions:
        ExecutionOrderedMap<DurableCompositePublicationAttemptIdentity, AdmittedWorldDisposition>,
}
impl WorldCheckpointImage {
    pub(crate) fn empty(binding: &WorldDurableHomeBinding) -> Self {
        Self {
            binding: binding.clone(),
            coverage: Default::default(),
            owner: None,
            frontiers: Default::default(),
            branches: Default::default(),
            occurrences: Default::default(),
            history: Default::default(),
            heads: Default::default(),
            opens: Default::default(),
            dispositions: Default::default(),
        }
    }
    pub(crate) fn branch_origins(&self) -> impl ExactSizeIterator<Item = &WorldBranchImageKey> {
        self.branches.keys()
    }
    pub(crate) fn first_branch(&self) -> Option<&WorldBranchImageKey> {
        self.branches.entry_at(0).map(|(key, ())| key)
    }
}
impl std::fmt::Debug for WorldCheckpointImage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorldCheckpointImage")
            .field("coverage", &self.coverage)
            .field("branches", &self.branches.len())
            .field("opens", &self.opens.len())
            .field("occurrences", &self.occurrences.len())
            .field("dispositions", &self.dispositions.len())
            .finish_non_exhaustive()
    }
}
