//! What tracked name reads rested on, and whether a later commit stales them.

use std::collections::{BTreeMap, BTreeSet};

use worth_relational::facade::history::BranchId;
use worth_relational::facade::identity::LineageId;
use worth_relational::facade::lineage::{
    LineageEventKind, LineageEventRecord, LineageRecordTarget, PersistentNameResolutionFootprint,
};
use worth_relational::facade::transactions::CommitResult;

/// The dependency footprint of every tracked name read one reader made.
///
/// Per branch it holds the live records the answers rested on (keyed by
/// their lineage), the lineage each ended name on a walked path held until
/// it ended, and the digests of held names that were still unknown.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct NameReadDependencies {
    branches: BTreeMap<BranchId, BranchNameDependencies>,
}

mod groups;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct BranchNameDependencies {
    live_records: BTreeMap<LineageId, LineageRecordTarget>,
    ended_lineages: BTreeSet<LineageId>,
    awaited_births: BTreeSet<[u8; 32]>,
    /// Groups an answer rested on: a held group and its live successors.
    groups: BTreeSet<[u8; 32]>,
    /// `(scheme, member)` group lookups and the group each one answered.
    group_members: BTreeMap<(String, [u8; 32]), Option<[u8; 32]>>,
}

impl BranchNameDependencies {
    fn is_empty(&self) -> bool {
        self.live_records.is_empty()
            && self.ended_lineages.is_empty()
            && self.awaited_births.is_empty()
            && self.groups.is_empty()
            && self.group_members.is_empty()
    }

    fn rests_on(&self, lineage: &LineageId) -> bool {
        self.live_records.contains_key(lineage) || self.ended_lineages.contains(lineage)
    }
}

/// Whether a later commit changed what recorded name reads rested on.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NameReadCurrentness {
    /// No later lineage event touched a recorded dependency.
    Clean,
    /// The recorded reads must be re-evaluated.
    Stale(NameReadStaleness),
}

/// The recorded dependencies a commit touched, in ascending order.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct NameReadStaleness {
    /// Lineages a read rested on that the commit ended, replaced, split,
    /// merged away or restored from.
    pub lineages: Vec<LineageId>,
    /// Digests of held names a read found unknown that the commit birthed.
    pub births: Vec<[u8; 32]>,
    /// Groups a read rested on, or answered for a member, that the commit
    /// retired, revived, split, merged or re-membered.
    pub groups: Vec<[u8; 32]>,
}

impl NameReadDependencies {
    pub fn new() -> Self {
        Self::default()
    }

    /// Whether no read recorded any dependency.
    pub fn is_empty(&self) -> bool {
        self.branches.values().all(BranchNameDependencies::is_empty)
    }

    /// The live records reads on `branch` rested on, in lineage order.
    pub fn live_records_on(&self, branch: &BranchId) -> Vec<LineageRecordTarget> {
        self.branches
            .get(branch)
            .map(|deps| deps.live_records.values().copied().collect())
            .unwrap_or_default()
    }

    /// The lineages of ended names reads on `branch` walked through,
    /// ascending; only a `Restore` from one of them changes those answers.
    pub fn ended_lineages_on(&self, branch: &BranchId) -> Vec<LineageId> {
        self.branches
            .get(branch)
            .map(|deps| deps.ended_lineages.iter().copied().collect())
            .unwrap_or_default()
    }

    /// Whether a read on `branch` found a held name not yet born.
    pub fn awaits_births_on(&self, branch: &BranchId) -> bool {
        self.branches
            .get(branch)
            .is_some_and(|deps| !deps.awaited_births.is_empty())
    }

    pub(crate) fn record_live_record(
        &mut self,
        branch: &BranchId,
        lineage: LineageId,
        record: LineageRecordTarget,
    ) {
        self.branch_mut(branch).live_records.insert(lineage, record);
    }

    pub(crate) fn record_footprint(
        &mut self,
        branch: &BranchId,
        footprint: &PersistentNameResolutionFootprint,
    ) {
        let deps = self.branch_mut(branch);
        for target in &footprint.live {
            if let Some((lineage, record)) = live_record(target) {
                deps.live_records.insert(lineage, record);
            }
        }
        deps.ended_lineages
            .extend(footprint.ended_lineages.iter().copied());
    }

    pub(crate) fn record_awaited_birth(&mut self, branch: &BranchId, digest: [u8; 32]) {
        self.branch_mut(branch).awaited_births.insert(digest);
    }

    /// Whether `commit` stales any recorded read.
    ///
    /// Complexity: O(commit lineage events * log dependencies).
    pub fn currentness_after_commit(&self, commit: &CommitResult) -> NameReadCurrentness {
        let events = self.currentness_after_lineage_events(commit.envelope().lineage_events());
        let groups = self.currentness_after_group_lineage(
            &commit.envelope().branch_context,
            commit.group_lineage(),
        );
        events.join(groups)
    }

    /// Whether one commit's canonical lineage events stale any recorded read.
    /// Events on branches no read touched never stale anything.
    ///
    /// Complexity: O(events * log dependencies).
    pub fn currentness_after_lineage_events(
        &self,
        events: &[LineageEventRecord],
    ) -> NameReadCurrentness {
        let mut stale_lineages = BTreeSet::new();
        let mut births = BTreeSet::new();
        for event in events {
            let Some(deps) = self.branches.get(event.branch_id()) else {
                continue;
            };
            match event.kind() {
                LineageEventKind::Create => {
                    if let Some(birth) = event.name_birth()
                        && deps.awaited_births.contains(birth.identity().as_bytes())
                    {
                        births.insert(*birth.identity().as_bytes());
                    }
                }
                LineageEventKind::Retire
                | LineageEventKind::Replace
                | LineageEventKind::Split
                | LineageEventKind::Merge
                | LineageEventKind::Restore => {
                    let ending = event
                        .sources()
                        .iter()
                        .filter(|source| !event.targets().contains(source))
                        .filter(|source| deps.rests_on(source));
                    stale_lineages.extend(ending.copied());
                }
            }
        }
        if stale_lineages.is_empty() && births.is_empty() {
            return NameReadCurrentness::Clean;
        }
        NameReadCurrentness::Stale(NameReadStaleness {
            lineages: stale_lineages.into_iter().collect(),
            births: births.into_iter().collect(),
            groups: Vec::new(),
        })
    }

    fn branch_mut(&mut self, branch: &BranchId) -> &mut BranchNameDependencies {
        self.branches.entry(branch.clone()).or_default()
    }
}

fn live_record(
    target: &worth_relational::facade::lineage::PersistentNameTarget,
) -> Option<(LineageId, LineageRecordTarget)> {
    use worth_relational::facade::lineage::PersistentNameTarget as Target;
    match target {
        Target::Entity {
            entity, lineage, ..
        } => Some((*lineage, LineageRecordTarget::Entity(*entity))),
        Target::Relation {
            relation, lineage, ..
        } => Some((*lineage, LineageRecordTarget::Relation(*relation))),
        Target::Group { .. } => None,
    }
}
