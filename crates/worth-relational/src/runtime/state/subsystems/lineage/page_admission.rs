//! Bind cold page placement and admit each verified page into its runtime once.
use super::{event_residency, node_residency, LineageSubsystem, ResidentLineageEventPage};
use crate::durability::data::lineage_event_pages::RelationalCheckpointLineageEventPage;
use crate::durability::medium::RelationalDurableColdSource;
use crate::history::data::RelationalCommitReceipt;
use crate::history::data::{BranchId, CommitId};
use crate::identity::data::LineageId;
use crate::lineage::data::LineageEventRecord;
use crate::storage::data::{
    checkpoint_page::RelationalCheckpointPage, RelationalStorageUnavailable,
};
use crate::storage::residency::RelationalResidencyDemandFailure as Failure;
use std::{collections::BTreeSet, sync::Arc};

/// The exact decoded page routes and source custody prepared for installation.
pub(crate) struct PreparedLineageColdPages {
    nodes: node_residency::RuntimeLineageNodeResidency,
    events: event_residency::RuntimeLineageEventResidency,
}

impl PreparedLineageColdPages {
    pub(crate) fn prepare(
        nodes: &[RelationalCheckpointPage],
        events: &[RelationalCheckpointLineageEventPage],
        source: &Arc<dyn RelationalDurableColdSource>,
        commits: impl Iterator<Item = RelationalCommitReceipt>,
    ) -> Result<Self, RelationalStorageUnavailable> {
        let nodes = node_residency::RuntimeLineageNodeResidency::new(nodes, source)?;
        let events = event_residency::RuntimeLineageEventResidency::new(events, source, commits)?;
        Ok(Self { nodes, events })
    }
}

impl LineageSubsystem {
    pub(crate) fn install_cold_pages<'a, Source>(
        &self,
        source: &'a Source,
    ) -> Result<(), RelationalStorageUnavailable>
    where
        &'a Source: TryInto<PreparedLineageColdPages, Error = RelationalStorageUnavailable>,
    {
        let PreparedLineageColdPages { nodes, events } = source.try_into()?;
        *self.node_residency.write() = Some(Arc::new(nodes));
        *self.event_residency.write() = Some(Arc::new(events));
        self.event_admissions.write().clear();
        Ok(())
    }
    pub(crate) fn demand_event_pages(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        nodes: Option<&BTreeSet<LineageId>>,
        branches: Option<&BTreeSet<BranchId>>,
    ) -> Result<Vec<ResidentLineageEventPage>, Failure> {
        crate::execution::request_custody::check_request(execution).map_err(Failure::Request)?;
        let Some(residency) = self.event_residency.read().clone() else {
            return Ok(Vec::new());
        };
        let pages = residency.demand(execution, nodes, branches)?;
        let mut admitted = self.event_admissions.write();
        let mut state = self.state.write();
        for page in &pages {
            if admitted.contains(&page.member()) {
                continue;
            }
            state
                .admit_readmitted_events(
                    page.rows()
                        .iter()
                        .map(|row| (row.event.clone(), row.publication_commit_id)),
                )
                .map_err(|cause| {
                    let mut failure = page.binding_failure();
                    failure.cause = cause;
                    Failure::StorageUnavailable(failure)
                })?;
            admitted.insert(page.member());
        }
        Ok(pages)
    }
    pub(crate) fn capture_resident_event_pages(
        &self,
    ) -> Result<
        (Vec<ResidentLineageEventPage>, bool),
        crate::storage::data::RelationalStorageUnavailable,
    > {
        self.event_residency
            .read()
            .as_ref()
            .map_or(Ok((Vec::new(), true)), |residency| {
                residency.capture_resident()
            })
    }

    pub(crate) fn event_member_sources(
        &self,
    ) -> std::collections::BTreeMap<
        crate::durability::medium::RelationalDurableMemberLocator,
        (
            crate::storage::data::RelationalStorageSubject,
            Arc<dyn crate::durability::medium::RelationalDurableColdSource>,
        ),
    > {
        let Some(residency) = self.event_residency.read().clone() else {
            return Default::default();
        };
        let source =
            residency.clone() as Arc<dyn crate::durability::medium::RelationalDurableColdSource>;
        residency
            .member_subjects()
            .map(|(member, subject)| (member, (subject, source.clone())))
            .collect()
    }
    pub(crate) fn node_export_sources(
        &self,
    ) -> std::collections::BTreeMap<
        crate::durability::medium::RelationalDurableMemberLocator,
        (
            crate::storage::data::RelationalStorageSubject,
            Arc<dyn crate::durability::medium::RelationalDurableColdSource>,
        ),
    > {
        let Some(pages) = self.node_residency.read().clone() else {
            return Default::default();
        };
        pages.export_sources()
    }
    pub(crate) fn has_paged_events(&self) -> bool {
        self.event_residency.read().is_some()
    }

    /// Whether lineage nodes are served from cold pages, so the derived
    /// entity index holds only the epochs demanded so far.
    pub(crate) fn has_paged_nodes(&self) -> bool {
        self.node_residency.read().is_some()
    }
}

impl super::LineageState {
    /// Installs readmitted event-page rows, verifying rows already present
    /// and feeding in-order rows to the derived naming state and name index.
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
        let prior_max_event_id = self
            .published_event_positions
            .last_key_value()
            .map(|(&id, _)| id);
        let mut in_order_positions = Vec::new();
        for (event, publication_commit_id) in rows {
            if !self
                .published_event_positions
                .contains_key(&event.event_id())
            {
                let in_order = self
                    .naming_rebuilt_event_frontier
                    .is_none_or(|frontier| event.event_id() > frontier)
                    && prior_max_event_id.is_none_or(|max_id| event.event_id() > max_id);
                if in_order {
                    in_order_positions.push(self.published_events.len());
                }
                self.record_shared_event(event, publication_commit_id, in_order);
            }
        }
        self.ingest_readmitted_name_events(&in_order_positions);
        Ok(())
    }
}
