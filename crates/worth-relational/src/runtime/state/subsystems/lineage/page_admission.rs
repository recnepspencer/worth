//! Bind cold page placement and admit each verified page into its runtime once.
use super::{event_residency, node_residency, LineageSubsystem, ResidentLineageEventPage};
use crate::durability::data::RelationalCheckpointDirectory;
use crate::history::data::BranchId;
use crate::identity::data::LineageId;
use crate::storage::residency::RelationalResidencyDemandFailure as Failure;
use std::{collections::BTreeSet, sync::Arc};

impl LineageSubsystem {
    pub(crate) fn install_cold_pages(
        &self,
        directory: &RelationalCheckpointDirectory,
    ) -> Result<(), crate::storage::data::RelationalStorageUnavailable> {
        let nodes = node_residency::RuntimeLineageNodeResidency::new(directory)?;
        let events = event_residency::RuntimeLineageEventResidency::new(directory)?;
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
        crate::execution::check_request(execution).map_err(Failure::Request)?;
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
}
