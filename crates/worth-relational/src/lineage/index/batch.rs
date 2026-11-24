//! One commit's canonical naming input to the derived name index.

use std::collections::{BTreeMap, BTreeSet};

use crate::aggregate_identity::group_lineage::AggregateGroupLineageEvent;
use crate::aggregate_identity::transition::group::transition_result::AggregateGroupTransition;
use crate::history::data::CommitId;
use crate::identity::data::{EntityId, LineageId, RelationId};
#[cfg(test)]
use crate::lineage::data::PreparedLineageFinalization;
use crate::lineage::data::{LineageEventRecord, PersistentNameIdentity, PersistentNameKey};

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

    /// Adds one Phase 2 finalization: its event batch, the entity lineage
    /// nodes it installed, and the relation lineages it reserved. Commit
    /// publication does not yet maintain a runtime index, so only the
    /// finalization courts call this today.
    #[cfg(test)]
    pub(crate) fn with_prepared_lineage(self, prepared: &PreparedLineageFinalization) -> Self {
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

#[cfg(test)]
use worth_relational_contracts::internal_port::lineage::data::artifacts::finalization_artifact::PreparedLineageFinalizationFixturePort as _;
