//! Bind cold page placement and admit each verified page into its runtime once.
use super::{event_residency, node_residency, LineageSubsystem, ResidentLineageEventPage};
use crate::contracts::history::data::coordinates::{BranchId, CommitId};
use crate::contracts::identity::data::primitives::LineageId;
use crate::contracts::lineage::data::events::LineageEventRecord;
use crate::contracts::storage::data::storage_unavailable::RelationalStorageUnavailable;
use crate::durability::data::lineage_event_pages::RelationalCheckpointLineageEventPage;
use std::{collections::BTreeSet, sync::Arc};
use worth_relational_storage::internal_port::durability::medium::cold_source::RelationalDurableColdSource;
use worth_relational_storage::internal_port::storage::data::checkpoint_page::RelationalCheckpointPage;
use worth_relational_storage::internal_port::storage::residency::demand_failure::RelationalResidencyDemandFailure as Failure;

/// The exact decoded page routes and source custody prepared for installation.
pub struct PreparedLineageColdPages {
    nodes: node_residency::RuntimeLineageNodeResidency,
    events: event_residency::RuntimeLineageEventResidency,
}

impl PreparedLineageColdPages {
    pub fn prepare(
        nodes: &[RelationalCheckpointPage],
        events: &[RelationalCheckpointLineageEventPage],
        source: &Arc<dyn RelationalDurableColdSource>,
        commits: event_residency::LineageCommitBinding,
        may_contain_named_events: bool,
    ) -> Result<Self, RelationalStorageUnavailable> {
        let nodes = node_residency::RuntimeLineageNodeResidency::new(nodes, source)?;
        let events = event_residency::RuntimeLineageEventResidency::new(
            events,
            source,
            commits,
            may_contain_named_events,
        )?;
        Ok(Self { nodes, events })
    }
}

impl LineageSubsystem {
    pub fn install_cold_pages(&self, prepared: PreparedLineageColdPages) {
        let PreparedLineageColdPages { nodes, events } = prepared;
        *self.node_residency.write() = Some(Arc::new(nodes));
        *self.event_residency.write() = Some(Arc::new(events));
        self.event_admissions.write().clear();
        // Paged events are no longer all resident: no full rebuild may run.
        self.state.write().full_history = None;
    }

    /// Demands the event pages routed to `nodes` within `branches`. At least
    /// one filter is required: an unfiltered demand would fault the whole
    /// event history, which only [`Self::demand_all_event_pages`] may do.
    pub fn demand_event_pages(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        nodes: Option<&BTreeSet<LineageId>>,
        branches: Option<&BTreeSet<BranchId>>,
    ) -> Result<Vec<ResidentLineageEventPage>, Failure> {
        if nodes.is_none() && branches.is_none() {
            return Err(Failure::StorageUnavailable(
                crate::contracts::storage::data::storage_unavailable::RelationalStorageUnavailable {
                    subject: crate::contracts::storage::data::storage_unavailable::RelationalStorageSubject::Lineage {
                        first: LineageId(0),
                        last: LineageId(0),
                    },
                    cause: crate::contracts::storage::data::storage_unavailable::RelationalStorageUnavailableCause::LimitExceeded {
                        length: u64::MAX,
                        limit: 0,
                    },
                },
            ));
        }
        self.demand_routed_event_pages(execution, nodes, branches)
    }

    /// Materializes every event page: the memory checkpoint door's explicit
    /// whole-history copy, never a reopen or point read. It admits rows but
    /// rebuilds no derived naming state.
    pub fn demand_all_event_pages(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Vec<ResidentLineageEventPage>, Failure> {
        self.demand_routed_event_pages(execution, None, None)
    }

    fn demand_routed_event_pages(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        nodes: Option<&BTreeSet<LineageId>>,
        branches: Option<&BTreeSet<BranchId>>,
    ) -> Result<Vec<ResidentLineageEventPage>, Failure> {
        crate::contracts::execution::request_custody::check_request(execution)
            .map_err(Failure::Request)?;
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
    pub fn capture_resident_event_pages(
        &self,
    ) -> Result<
        (Vec<ResidentLineageEventPage>, bool),
        crate::contracts::storage::data::storage_unavailable::RelationalStorageUnavailable,
    > {
        self.event_residency
            .read()
            .as_ref()
            .map_or(Ok((Vec::new(), true)), |residency| {
                residency.capture_resident()
            })
    }

    pub fn event_member_sources(
        &self,
    ) -> std::collections::BTreeMap<
        worth_relational_storage::internal_port::durability::medium::cold_source::RelationalDurableMemberLocator,
        (
            crate::contracts::storage::data::storage_unavailable::RelationalStorageSubject,
            Arc<dyn worth_relational_storage::internal_port::durability::medium::cold_source::RelationalDurableColdSource>,
        ),
    >{
        let Some(residency) = self.event_residency.read().clone() else {
            return Default::default();
        };
        let source =
            residency.clone() as Arc<dyn worth_relational_storage::internal_port::durability::medium::cold_source::RelationalDurableColdSource>;
        residency
            .member_subjects()
            .map(|(member, subject)| (member, (subject, source.clone())))
            .collect()
    }
    pub fn node_export_sources(
        &self,
    ) -> std::collections::BTreeMap<
        worth_relational_storage::internal_port::durability::medium::cold_source::RelationalDurableMemberLocator,
        (
            crate::contracts::storage::data::storage_unavailable::RelationalStorageSubject,
            Arc<dyn worth_relational_storage::internal_port::durability::medium::cold_source::RelationalDurableColdSource>,
        ),
    >{
        let Some(pages) = self.node_residency.read().clone() else {
            return Default::default();
        };
        pages.export_sources()
    }
    pub fn has_paged_events(&self) -> bool {
        self.event_residency.read().is_some()
    }

    /// Whether lineage nodes are served from cold pages, so the derived
    /// entity index holds only the epochs demanded so far.
    pub fn has_paged_nodes(&self) -> bool {
        self.node_residency.read().is_some()
    }
}

impl super::LineageState {
    /// Installs readmitted event-page rows, verifying rows already present
    /// and feeding in-order rows to the derived naming state and name index.
    pub(super) fn admit_readmitted_events(
        &mut self,
        rows: impl Clone + Iterator<Item = (Arc<LineageEventRecord>, CommitId)>,
    ) -> Result<
        (),
        crate::contracts::storage::data::storage_unavailable::RelationalStorageUnavailableCause,
    > {
        use crate::contracts::storage::data::storage_unavailable::RelationalStorageUnavailableCause as Cause;
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
