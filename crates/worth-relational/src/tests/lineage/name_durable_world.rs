//! A real multi-commit naming lifecycle for the durable persistent-name log
//! courts. Every commit runs Relational posture finalization; groups run the
//! real group naming authority over members that are genuinely born names.
//!
//! The history exercises `Create`, `Split`, `Merge` (kept and minted),
//! `Retire`, in-commit regeneration, cross-commit entity and relation
//! `Restore`, split-child birth restore, and every group transition kind.

use std::collections::{BTreeMap, BTreeSet};
use worth_relational_lineage::internal_port::lineage::index::batch::PersistentNameCommittedBatchFixturePort as _;

use worth_foundational::facade::AggregateIdentity;

use crate::aggregate_identity::{
    AggregateContinuityRule, AggregateGroupLineageEvent, AggregateGroupMembers,
    AggregateGroupNamingAuthority, AggregateGroupScheme, AggregateGroupSchemeId,
    AggregateGroupTransition, AggregateMemberRole, AggregatePriorGroup,
};
use crate::facade::history::CommitId;
use crate::facade::lineage::{
    encode_commit_frame, encode_durable_persistent_name_log, encode_log_header,
    rebuild_name_index_from_durable_log, LineageEventKind, LineageSubject,
    PersistentNameBranchIndex, PersistentNameCommittedBatch, PersistentNameDurabilityError,
    PersistentNameSelection,
};
use crate::graph_member::NativeMemberKey;
use crate::identity::data::{EntityId, PartitionId};

use super::name_index_fixtures::{all, bounded, exact, main_branch, side};
use super::name_world::{Keep, LineageWorld, Name, Subject};

/// Group naming over real member names, one transition per call.
struct NamedGroups {
    scheme: AggregateGroupScheme,
    prior: Vec<AggregatePriorGroup>,
    retired: BTreeSet<AggregateIdentity>,
    carriers: BTreeMap<Name, u64>,
}

impl NamedGroups {
    fn new() -> Self {
        let id = AggregateGroupSchemeId::new("cad.loop.v1").expect("valid scheme id");
        Self {
            scheme: AggregateGroupScheme::new(id, AggregateContinuityRule::Anchor),
            prior: Vec::new(),
            retired: BTreeSet::new(),
            carriers: BTreeMap::new(),
        }
    }

    fn step(&mut self, groups: &[&[Name]]) -> AggregateGroupTransition {
        let mut proposed = Vec::with_capacity(groups.len());
        for group in groups {
            let mut members = AggregateGroupMembers::new();
            for name in *group {
                let next = self.carriers.len() as u64 + 1;
                let slot = *self.carriers.entry(*name).or_insert(next);
                let carrier = NativeMemberKey::Entity(EntityId::new(PartitionId::main(), slot, 1));
                members
                    .insert(*name, carrier, AggregateMemberRole::Ordinary)
                    .expect("distinct members");
            }
            proposed.push(members);
        }
        let transition = AggregateGroupNamingAuthority::transition(
            &self.scheme,
            &self.prior,
            &self.retired,
            proposed,
        )
        .expect("group transition names the proposed partition");
        for assignment in &transition.assignments {
            self.retired.remove(&assignment.identity);
        }
        self.retired.extend(transition.retired.iter().copied());
        self.prior = transition
            .assignments
            .iter()
            .map(|assignment| AggregatePriorGroup {
                identity: assignment.identity,
                anchor: assignment.anchor,
                sources: assignment.sources.clone(),
                members: assignment.members.clone(),
            })
            .collect();
        transition
    }
}

/// The committed naming log, each batch with the retention floor in force
/// after it, and the names the courts tamper with or select by.
pub(super) struct DurableNamingHistory {
    pub(super) commits: Vec<(PersistentNameCommittedBatch, Option<CommitId>)>,
    pub(super) split_child: Name,
    pub(super) merged: Name,
    pub(super) carriers: Vec<Name>,
}

impl DurableNamingHistory {
    pub(super) fn header(&self) -> Vec<u8> {
        encode_log_header(&main_branch()).expect("header encodes")
    }

    pub(super) fn frames(&self) -> Vec<Vec<u8>> {
        self.commits
            .iter()
            .map(|(batch, floor)| encode_commit_frame(batch, *floor).expect("frame encodes"))
            .collect()
    }

    /// The header followed by the first `boundary` frames.
    pub(super) fn persisted_through(&self, boundary: usize) -> Vec<u8> {
        let mut bytes = self.header();
        for frame in &self.frames()[..boundary] {
            bytes.extend_from_slice(frame);
        }
        bytes
    }

    /// The live index after the first `boundary` commits, applied one batch
    /// at a time exactly as commit publication would.
    pub(super) fn live_index(&self, boundary: usize) -> PersistentNameBranchIndex {
        let mut index = PersistentNameBranchIndex::new(main_branch(), None);
        for (batch, floor) in &self.commits[..boundary] {
            index.apply(batch).expect("committed batch ingests");
            if let Some(floor) = floor {
                index.advance_retention_floor(*floor);
            }
        }
        index
    }

    /// Rebuilds from the log with `mutate` applied to its committed batches.
    pub(super) fn rebuild_mutated(
        &self,
        mutate: impl FnOnce(&mut Vec<(PersistentNameCommittedBatch, Option<CommitId>)>),
    ) -> Result<PersistentNameBranchIndex, PersistentNameDurabilityError> {
        let mut commits = self.commits.clone();
        mutate(&mut commits);
        let bytes = encode_durable_persistent_name_log(
            &main_branch(),
            commits.iter().map(|(batch, floor)| (batch, *floor)),
        )
        .expect("the mutated log still encodes");
        rebuild_name_index_from_durable_log(&bytes)
    }

    pub(super) fn selections(&self) -> Vec<PersistentNameSelection> {
        let mut selections = vec![exact(), all(), side("left"), side("absent")];
        selections.extend(self.carriers.iter().map(|carrier| bounded(&[*carrier])));
        selections.push(bounded(&self.carriers));
        selections
    }
}

pub(super) fn durable_naming_history() -> DurableNamingHistory {
    let mut world = LineageWorld::new();
    let mut groups = NamedGroups::new();
    let mut commits = vec![(world.log()[0].clone(), None)];

    let face = world.create(Subject::Entity, "face");
    let seam = world.create(Subject::Relation, "seam");
    let cutter = world.create(Subject::Entity, "cutter");
    let (a, b) = (
        world.create(Subject::Entity, "a"),
        world.create(Subject::Entity, "b"),
    );
    let parent = world.create(Subject::Entity, "parent");
    let batch = world.commit();
    let [face, seam, cutter, a, b, parent] =
        [face, seam, cutter, a, b, parent].map(|holder| world.name_of(&holder));
    let mut members = [face, cutter, a, b];
    members.sort_unstable();
    let [m0, m1, m2, m3] = members;
    commits.push((batch.with_group_transition(&groups.step(&[&members])), None));

    let pieces = world.split(face, &["left", "right"]);
    world.bind_carriers(&pieces[0], &[cutter]);
    let seams = world.split(seam, &["top", "bottom"]);
    world.split(parent, &["left", "right"]);
    let batch = world.commit();
    let (left, right) = (world.name_of(&pieces[0]), world.name_of(&pieces[1]));
    let (top, bottom) = (world.name_of(&seams[0]), world.name_of(&seams[1]));
    commits.push((
        batch.with_group_transition(&groups.step(&[&[m1], &[m2, m3]])),
        None,
    ));

    world.retire(right);
    world.merge(&[top, bottom], Keep::Recreated);
    world.merge(&[a, b], Keep::InPlace);
    let parent_children = world
        .live_names()
        .into_iter()
        .filter(|(name, _)| world.birth_parts[name].parent_identities() == [parent])
        .map(|(name, _)| name)
        .collect::<Vec<_>>();
    for child in &parent_children {
        world.retire(*child);
    }
    let batch = world.commit();
    commits.push((
        batch.with_group_transition(&groups.step(&[&[m0], &[m1, m2, m3]])),
        None,
    ));

    let merged_holder = world.merge(&[left, cutter], Keep::MintNew);
    world.regenerate(a);
    world.restore(parent, false);
    let batch = world.commit();
    let merged = world.name_of(&merged_holder);
    commits.push((batch, None));

    let retired_children = world.retired_children_of(parent);
    assert_eq!(retired_children.len(), 2, "both parent pieces retired");
    world.resplit_replaying_retired(parent, &retired_children);
    world.retire(merged);
    world.restore(bottom, false);
    let batch = world.commit();
    let transition = groups.step(&[&[m0, m1, m2, m3]]);
    commits.push((batch.with_group_transition(&transition), Some(CommitId(2))));

    let history = DurableNamingHistory {
        commits,
        split_child: left,
        merged,
        carriers: vec![cutter, m0, m3],
    };
    assert_lifecycle_coverage(&history);
    history
}

/// The history must carry every event shape the durable log persists.
fn assert_lifecycle_coverage(history: &DurableNamingHistory) {
    let events: Vec<_> = history
        .commits
        .iter()
        .flat_map(|(batch, _)| batch.lineage_events_for_test().iter())
        .collect();
    let has = |predicate: &dyn Fn(&&crate::facade::lineage::LineageEventRecord) -> bool| {
        events.iter().any(predicate)
    };
    assert!(has(
        &|e| e.kind() == LineageEventKind::Create && e.name_birth().is_some()
    ));
    assert!(has(
        &|e| e.kind() == LineageEventKind::Create && e.kept_name().is_some()
    ));
    assert!(has(&|e| e.kind() == LineageEventKind::Split));
    assert!(has(
        &|e| e.kind() == LineageEventKind::Merge && e.kept_name().is_some()
    ));
    assert!(has(
        &|e| e.kind() == LineageEventKind::Merge && e.kept_name().is_none()
    ));
    assert!(has(&|e| e.kind() == LineageEventKind::Retire));
    for subject in [LineageSubject::Entity, LineageSubject::Relation] {
        assert!(has(
            &|e| e.kind() == LineageEventKind::Restore && e.subject() == subject
        ));
    }
    let restores_in_split_commit = history.commits.iter().any(|(batch, _)| {
        let kinds: Vec<_> = batch
            .lineage_events_for_test()
            .iter()
            .map(|e| e.kind())
            .collect();
        kinds.contains(&LineageEventKind::Split) && kinds.contains(&LineageEventKind::Restore)
    });
    assert!(restores_in_split_commit, "split-child birth restore");
    let group_kinds: BTreeSet<&str> = history
        .commits
        .iter()
        .flat_map(|(batch, _)| batch.group_events_for_test().iter())
        .map(|event| match event {
            AggregateGroupLineageEvent::Born { .. } => "born",
            AggregateGroupLineageEvent::Revived { .. } => "revived",
            AggregateGroupLineageEvent::SplitFrom { .. } => "split",
            AggregateGroupLineageEvent::MergedInto { .. } => "merged",
            AggregateGroupLineageEvent::Retired { .. } => "retired",
        })
        .collect();
    assert_eq!(
        group_kinds,
        BTreeSet::from(["born", "merged", "retired", "revived", "split"]),
        "every group transition kind"
    );
}
