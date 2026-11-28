use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use crate::contracts::history::data::coordinates::{BranchId, CommitId};
use crate::contracts::identity::data::primitives::LineageId;
use crate::contracts::identity::data::record_ids::{EntityId, RelationId};
use crate::contracts::lineage::data::events::LineageEventRecord;
use crate::contracts::lineage::data::graph::LineageNode;
use crate::lineage::tiered::ColdNamingSource;

use super::name_indexes::BranchNameIndexes;
use super::naming_basis::{AuthoritativeBranchNamingState, NamingBasisUnavailable};
use super::naming_rebuild::{rebuild_all_branch_naming_states, BranchNaming};
use super::recovery_token::FullLineageRecoveryToken;
use super::resolution_indexes::LineageResolutionIndexes;

crate::lineage::clone_fences::assert_not_clone!(LineageState);
crate::lineage::clone_fences::assert_not_clone!(super::name_indexes::BranchNameIndexes);
crate::lineage::clone_fences::assert_not_clone!(super::group_states::BranchGroupStates);

/// The lineage subsystem's authoritative contents (`!Clone`; use `fork_detached`).
#[derive(Debug)]
pub struct LineageState {
    pub(super) state_id: super::recovery_token::LineageStateId,
    pub nodes: BTreeMap<crate::contracts::identity::data::primitives::LineageId, LineageNode>,
    pub(super) relation_lineages: BTreeMap<RelationId, LineageId>,
    pub(super) relation_lineage_holders: BTreeMap<LineageId, RelationId>,
    pub(super) branch_forks: BTreeMap<BranchId, (BranchId, CommitId)>,
    pub(super) fork_children: BTreeMap<BranchId, BTreeMap<CommitId, BTreeSet<BranchId>>>,
    pub(super) branch_naming_states: BTreeMap<BranchId, BranchNaming>,
    pub(super) name_indexes: BranchNameIndexes,
    pub(super) naming_rebuilt_event_frontier: Option<u64>,
    pub(super) published_events: Vec<PublishedLineageEvent>,
    pub(super) published_event_positions: BTreeMap<u64, usize>,
    pub(super) event_commits_by_branch: BTreeMap<BranchId, BTreeSet<CommitId>>,
    pub(super) published_group_lineage: Vec<super::group_states::PublishedGroupLineage>,
    pub(super) group_states: super::group_states::BranchGroupStates,
    pub(super) cold_naming: Option<Arc<dyn ColdNamingSource>>,
    pub(super) branch_generations: BTreeMap<BranchId, u64>,
    pub(super) next_generation: u64,
    pub(super) full_history: Option<FullLineageRecoveryToken>,
    resolution_indexes: LineageResolutionIndexes,
    pub(crate) identity_allocator: super::LineageIdentityAllocator,
}

/// One published lineage event, held by shared ownership so a reader can carry
/// it out of the subsystem lock without copying it.
#[derive(Debug, Clone)]
pub(super) struct PublishedLineageEvent {
    pub(super) event: Arc<LineageEventRecord>,
    pub(super) publication_commit_id: CommitId,
}

pub use super::published_batch::ValidatedLineageEventBatch;

impl LineageState {
    pub(super) fn empty() -> Self {
        let state_id = super::recovery_token::next_state_id();
        Self {
            state_id,
            nodes: BTreeMap::new(),
            relation_lineages: BTreeMap::new(),
            relation_lineage_holders: BTreeMap::new(),
            branch_forks: BTreeMap::new(),
            fork_children: BTreeMap::new(),
            branch_naming_states: BTreeMap::new(),
            name_indexes: BranchNameIndexes::default(),
            naming_rebuilt_event_frontier: None,
            published_events: Vec::new(),
            published_event_positions: BTreeMap::new(),
            event_commits_by_branch: BTreeMap::new(),
            published_group_lineage: Vec::new(),
            group_states: Default::default(),
            cold_naming: None,
            branch_generations: BTreeMap::new(),
            next_generation: 1,
            full_history: Some(FullLineageRecoveryToken::fresh(state_id)),
            resolution_indexes: LineageResolutionIndexes::default(),
            identity_allocator: super::LineageIdentityAllocator::new(),
        }
    }

    pub(super) fn bump_branch_generation(&mut self, branch: &BranchId) {
        let generation = self.next_generation;
        self.next_generation = self.next_generation.saturating_add(1);
        self.branch_generations.insert(branch.clone(), generation);
    }

    fn record_event(&mut self, event: LineageEventRecord, publication_commit_id: CommitId) {
        self.record_shared_event(Arc::new(event), publication_commit_id, true);
    }

    pub(super) fn record_shared_event(
        &mut self,
        event: Arc<LineageEventRecord>,
        publication_commit_id: CommitId,
        apply_naming: bool,
    ) {
        let event_position = self.published_events.len();
        assert!(
            /* Proof: Live installation takes ValidatedLineageEventBatch after validate_reserved_lineage_events checks the exact batch from the unique owner allocator. install_recovered_event_batch rejects existing and non-increasing ids first; admit_readmitted_events skips ids already installed under the same exclusive state borrow. */
            self.published_event_positions
                .insert(event.event_id(), event_position)
                .is_none(),
            "validated lineage event id must remain unique until installation"
        );
        if let Some(relation_id) = event.relation_id()
            && let Some(&lineage_id) = event.targets().first()
        {
            self.record_relation_lineage(relation_id, lineage_id);
        }
        self.resolution_indexes.append_event(event_position, &event);
        if apply_naming {
            let branch = event.branch_id().clone();
            self.bump_branch_generation(&branch);
            let slot = self
                .branch_naming_states
                .entry(branch)
                .or_insert_with(|| Ok(AuthoritativeBranchNamingState::over(None, None, None)));
            if let Ok(state) = slot
                && let Err(cause) = state.apply_event(
                    &event,
                    publication_commit_id,
                    &self.nodes,
                    &self.relation_lineage_holders,
                )
            {
                *slot = Err(NamingBasisUnavailable::Storage(cause));
            }
        }
        self.event_commits_by_branch
            .entry(event.branch_id().clone())
            .or_default()
            .insert(publication_commit_id);
        self.published_events.push(PublishedLineageEvent {
            event,
            publication_commit_id,
        });
    }

    pub(crate) fn install_validated_event_batch(
        &mut self,
        batch: ValidatedLineageEventBatch,
        publication_commit_id: CommitId,
    ) {
        let from = self.published_events.len();
        let from_group = self.published_group_lineage.len();
        for event in batch.events {
            self.record_event(event, publication_commit_id);
        }
        if let Some((branch, transitions)) = batch.groups {
            self.record_group_lineage(&branch, publication_commit_id, transitions);
        }
        self.ingest_installed_name_events(from, from_group);
    }

    pub(crate) fn install_recovered_event_batch(
        &mut self,
        events: &[LineageEventRecord],
        publication_commit_id: CommitId,
    ) -> Result<(), String> {
        let mut previous_event_id = None;
        for event in events {
            if previous_event_id.is_some_and(|previous| event.event_id() <= previous) {
                return Err(
                    "recovered lineage event ids must advance within their durable batch"
                        .to_owned(),
                );
            }
            if self
                .published_event_positions
                .contains_key(&event.event_id())
            {
                return Err(format!(
                    "recovered lineage event id {} was already installed",
                    event.event_id()
                ));
            }
            previous_event_id = Some(event.event_id());
        }
        let next_event_id = previous_event_id
            .map(|event_id| {
                event_id
                    .checked_add(1)
                    .ok_or_else(|| "recovered lineage event exhausted the allocator".to_owned())
            })
            .transpose()?;
        let from = self.published_events.len();
        let from_group = self.published_group_lineage.len();
        for event in events.iter().cloned() {
            self.record_event(event, publication_commit_id);
        }
        self.ingest_installed_name_events(from, from_group);
        self.identity_allocator.advance_to(None, next_event_id);
        Ok(())
    }

    pub(crate) fn branch_events(
        &self,
        branch_id: &BranchId,
    ) -> impl Iterator<Item = &LineageEventRecord> + use<'_> {
        self.resolution_indexes
            .branch_event_positions(branch_id)
            .iter()
            .map(|position| self.published_events[*position].event.as_ref())
    }

    pub(crate) fn branch_event_positions_for_sources(
        &self,
        branch_id: &BranchId,
        lineage_ids: &BTreeSet<LineageId>,
    ) -> BTreeSet<usize> {
        self.resolution_indexes
            .branch_event_positions_for_sources(branch_id, lineage_ids)
    }

    pub(crate) fn record_node(&mut self, node: LineageNode) {
        if self.nodes.contains_key(&node.lineage_id()) {
            return;
        }
        self.resolution_indexes.record_node(&node);
        self.nodes.insert(node.lineage_id(), node);
    }

    pub(crate) fn branch_event_positions_for_lineages(
        &self,
        branch_ids: &BTreeSet<BranchId>,
        lineage_ids: &BTreeSet<LineageId>,
        sources_only: bool,
    ) -> (BTreeSet<usize>, usize) {
        self.resolution_indexes.branch_event_positions_for_lineages(
            branch_ids,
            lineage_ids,
            sources_only,
        )
    }

    pub(crate) fn indexed_lineages_for_entity(
        &self,
        entity_id: EntityId,
    ) -> (Vec<LineageId>, usize) {
        self.resolution_indexes.lineages_for_entity(entity_id)
    }

    pub(crate) fn indexed_lineages_are_exclusive_to_branch(
        &self,
        lineage_ids: &BTreeSet<LineageId>,
        branch_id: &BranchId,
        sources_only: bool,
    ) -> (bool, usize) {
        self.resolution_indexes.lineages_are_exclusive_to_branch(
            lineage_ids,
            branch_id,
            sources_only,
        )
    }

    pub(crate) fn event_publication_commit(&self, position: usize) -> Option<CommitId> {
        self.published_events
            .get(position)
            .map(|p| p.publication_commit_id)
    }

    pub(crate) fn event(&self, position: usize) -> Option<Arc<LineageEventRecord>> {
        self.published_events
            .get(position)
            .map(|p| Arc::clone(&p.event))
    }

    pub(crate) fn events(&self) -> impl DoubleEndedIterator<Item = &LineageEventRecord> {
        self.published_events.iter().map(|p| p.event.as_ref())
    }

    pub(crate) fn published_rows(&self) -> Vec<(Arc<LineageEventRecord>, CommitId)> {
        self.published_events
            .iter()
            .map(|r| (r.event.clone(), r.publication_commit_id))
            .collect()
    }

    pub(crate) fn shared_events(&self) -> Vec<Arc<LineageEventRecord>> {
        self.published_events
            .iter()
            .map(|p| Arc::clone(&p.event))
            .collect()
    }

    pub fn rebuild_derived_indexes(
        &mut self,
        lane: &crate::contracts::storage::substrate::maintenance::reconstruction::ReconstructionLane<'_>,
        full: &FullLineageRecoveryToken,
    ) {
        full.assert_owner(self.state_id);
        let relation_bindings: Vec<_> = self
            .published_events
            .iter()
            .filter_map(|p| Some((p.event.relation_id()?, *p.event.targets().first()?)))
            .collect();
        for (relation_id, lineage_id) in relation_bindings {
            self.record_relation_lineage(relation_id, lineage_id);
        }
        self.published_event_positions = self
            .published_events
            .iter()
            .enumerate()
            .map(|(position, published)| (published.event.event_id(), position))
            .collect();
        self.event_commits_by_branch.clear();
        for published in &self.published_events {
            self.event_commits_by_branch
                .entry(published.event.branch_id().clone())
                .or_default()
                .insert(published.publication_commit_id);
        }
        let nodes = self.nodes.values();
        let events = self.published_events.iter().map(|p| p.event.as_ref());
        self.resolution_indexes.rebuild(lane, nodes, events);
        self.rebuild_group_states(full);
        self.branch_naming_states = rebuild_all_branch_naming_states(
            full,
            &self.published_events,
            &self.nodes,
            &self.relation_lineage_holders,
            &self.branch_forks,
        );
        self.rebuild_name_indexes(full);
    }

    pub fn replace_events(
        &mut self,
        lane: &crate::contracts::storage::substrate::maintenance::reconstruction::ReconstructionLane<'_>,
        events: Vec<(LineageEventRecord, CommitId)>,
    ) {
        self.published_events = events
            .into_iter()
            .map(|(event, publication_commit_id)| PublishedLineageEvent {
                event: Arc::new(event),
                publication_commit_id,
            })
            .collect();
        self.cold_naming = None;
        let full = FullLineageRecoveryToken::full_replay(self.state_id);
        self.rebuild_derived_indexes(lane, &full);
        self.full_history = Some(full);
        self.naming_rebuilt_event_frontier = self
            .published_events
            .last()
            .map(|published| published.event.event_id());
    }

    pub(crate) fn drain_events(
        &mut self,
    ) -> impl Iterator<Item = (LineageEventRecord, CommitId)> + use<> {
        self.published_event_positions.clear();
        self.event_commits_by_branch.clear();
        self.branch_naming_states.clear();
        self.name_indexes.clear();
        std::mem::take(&mut self.published_events)
            .into_iter()
            .map(|p| (Arc::unwrap_or_clone(p.event), p.publication_commit_id))
    }
}

impl Default for LineageState {
    fn default() -> Self {
        Self::empty()
    }
}

mod whole_state_copy;
