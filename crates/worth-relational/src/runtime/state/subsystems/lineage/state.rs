use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use crate::history::data::{BranchId, CommitId};
use crate::identity::data::{EntityId, LineageId};
use crate::lineage::data::{LineageEventRecord, LineageNode};

use super::resolution_indexes::LineageResolutionIndexes;

/// The lineage subsystem's authoritative contents.
#[derive(Debug)]
pub(crate) struct LineageState {
    pub(crate) nodes: BTreeMap<crate::identity::data::LineageId, LineageNode>,
    published_events: Vec<PublishedLineageEvent>,
    published_event_positions: BTreeMap<u64, usize>,
    resolution_indexes: LineageResolutionIndexes,
    pub(crate) identity_allocator: crate::runtime::state::subsystems::LineageIdentityAllocator,
}

/// One published lineage event, held by shared ownership so a reader can carry
/// it out of the subsystem lock without copying it.
#[derive(Debug, Clone)]
struct PublishedLineageEvent {
    event: Arc<LineageEventRecord>,
    publication_commit_id: CommitId,
}

pub(crate) struct ValidatedLineageEventBatch {
    events: Vec<LineageEventRecord>,
}

impl ValidatedLineageEventBatch {
    pub(crate) fn from_reserved(events: Vec<LineageEventRecord>) -> Self {
        Self { events }
    }
}

impl LineageState {
    pub(super) fn empty() -> Self {
        Self {
            nodes: BTreeMap::new(),
            published_events: Vec::new(),
            published_event_positions: BTreeMap::new(),
            resolution_indexes: LineageResolutionIndexes::default(),
            identity_allocator: crate::runtime::state::subsystems::LineageIdentityAllocator::new(),
        }
    }

    fn record_event(&mut self, event: LineageEventRecord, publication_commit_id: CommitId) {
        self.record_shared_event(Arc::new(event), publication_commit_id);
    }

    fn record_shared_event(
        &mut self,
        event: Arc<LineageEventRecord>,
        publication_commit_id: CommitId,
    ) {
        let event_position = self.published_events.len();
        assert!(
            self.published_event_positions
                .insert(event.event_id(), event_position)
                .is_none(),
            "validated lineage event id must remain unique until installation"
        );
        self.resolution_indexes.append_event(event_position, &event);
        self.published_events.push(PublishedLineageEvent {
            event,
            publication_commit_id,
        });
    }

    pub(super) fn admit_readmitted_events(
        &mut self,
        rows: impl Clone + Iterator<Item = (Arc<LineageEventRecord>, CommitId)>,
    ) -> Result<(), crate::storage::data::RelationalStorageUnavailableCause> {
        use crate::storage::data::RelationalStorageUnavailableCause as Cause;
        let mut fresh = 0;
        for (event, publication_commit_id) in rows.clone() {
            if let Some(position) = self.published_event_positions.get(&event.event_id()) {
                let existing = &self.published_events[*position];
                if existing.event != event
                    || existing.publication_commit_id != publication_commit_id
                {
                    return Err(Cause::Damaged);
                }
            } else {
                fresh += 1;
            }
        }
        self.published_events
            .try_reserve(fresh)
            .map_err(|_| Cause::MemoryExhausted)?;
        for (event, publication_commit_id) in rows {
            if !self
                .published_event_positions
                .contains_key(&event.event_id())
            {
                self.record_shared_event(event, publication_commit_id);
            }
        }
        Ok(())
    }

    pub(crate) fn install_validated_event_batch(
        &mut self,
        batch: ValidatedLineageEventBatch,
        publication_commit_id: CommitId,
    ) {
        for event in batch.events {
            self.record_event(event, publication_commit_id);
        }
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
        for event in events.iter().cloned() {
            self.record_event(event, publication_commit_id);
        }
        self.identity_allocator.advance_to(None, next_event_id);
        Ok(())
    }

    #[cfg(test)]
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

    pub(crate) fn indexed_lineage_for_entity(
        &self,
        entity_id: EntityId,
    ) -> (Option<LineageId>, usize) {
        self.resolution_indexes.lineage_for_entity(entity_id)
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
            .map(|published| published.publication_commit_id)
    }

    pub(crate) fn event(&self, position: usize) -> Option<Arc<LineageEventRecord>> {
        self.published_events
            .get(position)
            .map(|published| Arc::clone(&published.event))
    }

    pub(crate) fn events(&self) -> impl DoubleEndedIterator<Item = &LineageEventRecord> {
        self.published_events
            .iter()
            .map(|published| published.event.as_ref())
    }

    pub(crate) fn published_rows(&self) -> Vec<(Arc<LineageEventRecord>, CommitId)> {
        self.published_events
            .iter()
            .map(|row| (row.event.clone(), row.publication_commit_id))
            .collect()
    }

    pub(crate) fn shared_events(&self) -> Vec<Arc<LineageEventRecord>> {
        self.published_events
            .iter()
            .map(|published| Arc::clone(&published.event))
            .collect()
    }

    pub(crate) fn rebuild_derived_indexes(&mut self) {
        self.published_event_positions = self
            .published_events
            .iter()
            .enumerate()
            .map(|(position, published)| (published.event.event_id(), position))
            .collect();
        let nodes = self.nodes.values();
        let events = self
            .published_events
            .iter()
            .map(|published| published.event.as_ref());
        self.resolution_indexes.rebuild(nodes, events);
    }

    pub(crate) fn replace_events(&mut self, events: Vec<(LineageEventRecord, CommitId)>) {
        self.published_events = events
            .into_iter()
            .map(|(event, publication_commit_id)| PublishedLineageEvent {
                event: Arc::new(event),
                publication_commit_id,
            })
            .collect();
        self.rebuild_derived_indexes();
    }

    pub(crate) fn drain_events(
        &mut self,
    ) -> impl Iterator<Item = (LineageEventRecord, CommitId)> + use<> {
        self.published_event_positions.clear();
        std::mem::take(&mut self.published_events)
            .into_iter()
            .map(|published| {
                (
                    Arc::try_unwrap(published.event)
                        .unwrap_or_else(|shared| shared.as_ref().clone()),
                    published.publication_commit_id,
                )
            })
    }
}

impl Default for LineageState {
    fn default() -> Self {
        Self::empty()
    }
}

impl Clone for LineageState {
    fn clone(&self) -> Self {
        Self {
            nodes: self.nodes.clone(),
            published_events: self.published_events.clone(),
            published_event_positions: self.published_event_positions.clone(),
            resolution_indexes: self.resolution_indexes.clone(),
            identity_allocator: self.identity_allocator.detached(),
        }
    }
}
