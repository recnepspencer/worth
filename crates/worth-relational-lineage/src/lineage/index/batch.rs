//! One commit's canonical naming input to the derived name index.

use std::collections::{BTreeMap, BTreeSet};

use crate::contracts::aggregate_identity::group_lineage::AggregateGroupLineageEvent;
use crate::contracts::aggregate_identity::transition::group::transition_result::AggregateGroupTransition;
use crate::contracts::history::data::coordinates::CommitId;
use crate::contracts::identity::data::primitives::LineageId;
use crate::contracts::identity::data::record_ids::{EntityId, RelationId};
use crate::contracts::lineage::data::artifacts::finalization_artifact::PreparedLineageFinalization;
use crate::contracts::lineage::data::events::LineageEventRecord;
use crate::contracts::lineage::data::name_resolution::PersistentNameKey;
use crate::contracts::lineage::data::naming::PersistentNameIdentity;

/// Everything one committed transaction contributes to persistent naming:
/// its canonical entity/relation lineage events with the records their
/// created lineages bind to, its group lineage events, and declared carrier
/// sets. The index derives all of its state from an ordered log of these.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersistentNameCommittedBatch {
    pub(crate) commit: CommitId,
    pub(crate) lineage_events: Vec<LineageEventRecord>,
    pub(crate) entity_holders: BTreeMap<LineageId, EntityId>,
    pub(crate) relation_holders: BTreeMap<LineageId, RelationId>,
    pub(crate) group_events: BTreeSet<AggregateGroupLineageEvent>,
    pub(crate) carriers: BTreeMap<PersistentNameKey, BTreeSet<PersistentNameIdentity>>,
}

impl PersistentNameCommittedBatch {
    pub fn new(commit: CommitId) -> Self {
        Self {
            commit,
            lineage_events: Vec::new(),
            entity_holders: BTreeMap::new(),
            relation_holders: BTreeMap::new(),
            group_events: BTreeSet::new(),
            carriers: BTreeMap::new(),
        }
    }

    pub const fn commit(&self) -> CommitId {
        self.commit
    }

    /// Adds canonical committed lineage events and the records that their
    /// created lineages bind to.
    pub fn with_lineage_events(
        mut self,
        events: impl IntoIterator<Item = LineageEventRecord>,
        entity_holders: impl IntoIterator<Item = (LineageId, EntityId)>,
        relation_holders: impl IntoIterator<Item = (LineageId, RelationId)>,
    ) -> Self {
        self.lineage_events.extend(events);
        self.entity_holders.extend(entity_holders);
        self.relation_holders.extend(relation_holders);
        self
    }

    /// Adds canonical group lineage events.
    pub fn with_group_events(
        mut self,
        events: impl IntoIterator<Item = AggregateGroupLineageEvent>,
    ) -> Self {
        self.group_events.extend(events);
        self
    }

    /// Adds one group naming transition: its lineage events, and each
    /// next-state group's member names as that group's carriers.
    pub fn with_group_transition(mut self, transition: &AggregateGroupTransition) -> Self {
        self.group_events.extend(transition.lineage.iter().copied());
        for assignment in &transition.assignments {
            let members = assignment.members.iter().map(|(name, _)| *name).collect();
            self.carriers
                .insert(PersistentNameKey::Group(assignment.identity), members);
        }
        self
    }

    /// Declares the carriers `name` is bounded by from this commit on.
    pub fn with_carriers(
        mut self,
        name: PersistentNameKey,
        carriers: impl IntoIterator<Item = PersistentNameIdentity>,
    ) -> Self {
        self.carriers.insert(name, carriers.into_iter().collect());
        self
    }
}

use crate::contracts::lineage::data::artifacts::finalization_artifact::PreparedLineageFinalizationFixturePort as _;

#[doc(hidden)]
pub trait PersistentNameCommittedBatchFixturePort: Sized {
    fn with_prepared_lineage_for_test(self, prepared: &PreparedLineageFinalization) -> Self;
    fn carriers_for_test(&self) -> &BTreeMap<PersistentNameKey, BTreeSet<PersistentNameIdentity>>;
    fn carriers_mut_for_test(
        &mut self,
    ) -> &mut BTreeMap<PersistentNameKey, BTreeSet<PersistentNameIdentity>>;
    fn entity_holders_for_test(&self) -> &BTreeMap<LineageId, EntityId>;
    fn entity_holders_mut_for_test(&mut self) -> &mut BTreeMap<LineageId, EntityId>;
    fn group_events_for_test(&self) -> &BTreeSet<AggregateGroupLineageEvent>;
    fn lineage_events_for_test(&self) -> &Vec<LineageEventRecord>;
    fn lineage_events_mut_for_test(&mut self) -> &mut Vec<LineageEventRecord>;
    fn relation_holders_for_test(&self) -> &BTreeMap<LineageId, RelationId>;
    fn relation_holders_mut_for_test(&mut self) -> &mut BTreeMap<LineageId, RelationId>;
}

impl PersistentNameCommittedBatchFixturePort for PersistentNameCommittedBatch {
    fn with_prepared_lineage_for_test(self, prepared: &PreparedLineageFinalization) -> Self {
        let events = prepared.artifact().event_batch().events().iter().cloned();
        let entities = prepared
            .new_nodes_for_test()
            .iter()
            .map(|node| (node.lineage_id(), node.entity_id()));
        let relations = prepared
            .relation_lineages_for_test()
            .iter()
            .map(|(relation, lineage)| (*lineage, *relation));
        self.with_lineage_events(events, entities, relations)
    }

    fn carriers_for_test(&self) -> &BTreeMap<PersistentNameKey, BTreeSet<PersistentNameIdentity>> {
        &self.carriers
    }

    fn carriers_mut_for_test(
        &mut self,
    ) -> &mut BTreeMap<PersistentNameKey, BTreeSet<PersistentNameIdentity>> {
        &mut self.carriers
    }

    fn entity_holders_for_test(&self) -> &BTreeMap<LineageId, EntityId> {
        &self.entity_holders
    }

    fn entity_holders_mut_for_test(&mut self) -> &mut BTreeMap<LineageId, EntityId> {
        &mut self.entity_holders
    }

    fn group_events_for_test(&self) -> &BTreeSet<AggregateGroupLineageEvent> {
        &self.group_events
    }

    fn lineage_events_for_test(&self) -> &Vec<LineageEventRecord> {
        &self.lineage_events
    }

    fn lineage_events_mut_for_test(&mut self) -> &mut Vec<LineageEventRecord> {
        &mut self.lineage_events
    }

    fn relation_holders_for_test(&self) -> &BTreeMap<LineageId, RelationId> {
        &self.relation_holders
    }

    fn relation_holders_mut_for_test(&mut self) -> &mut BTreeMap<LineageId, RelationId> {
        &mut self.relation_holders
    }
}
