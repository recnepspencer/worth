//! One counted, damage-isolated event cell is shared by all its node-range routes.
use crate::contracts::storage::data::storage_unavailable::{
    RelationalStorageSubject, RelationalStorageUnavailable,
    RelationalStorageUnavailableCause as Cause, RelationalStorageUnavailableCauseOwnerPort as _,
};
pub mod member_export;
use super::event_page::{self, ReadmittedLineageEventPage};
use crate::contracts::history::data::commit_receipt::RelationalCommitReceipt;
use crate::contracts::history::data::coordinates::{BranchId, CommitId};
use crate::contracts::identity::data::primitives::LineageId;
use crate::durability::data::lineage_event_pages::*;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex,
};
use worth_relational_storage::internal_port::durability::medium::cold_source::{
    RelationalDurableColdSource, RelationalDurableMemberLocator,
};
use worth_relational_storage::internal_port::durability::medium::member_custody::{
    native_source_slot::NativeSourceMemberState, RelationalDurableMemberCustody,
};
use worth_relational_storage::internal_port::storage::residency::demand_failure::RelationalResidencyDemandFailure as Failure;

#[derive(Debug)]
enum EventPageState {
    Cold,
    Resident(Arc<ReadmittedLineageEventPage>),
    Damaged(RelationalStorageUnavailable),
}
#[derive(Debug)]
struct EventPageCell {
    descriptor: RelationalCheckpointLineageEventPage,
    state: Mutex<EventPageState>,
    leases: AtomicUsize,
}
#[derive(Debug)]
pub(super) struct RuntimeLineageEventResidency {
    pub(super) custody: Mutex<NativeSourceMemberState>,
    pages: BTreeMap<RelationalDurableMemberLocator, Arc<EventPageCell>>,
    routes: BTreeMap<RelationalLineageEventPageRoute, Vec<Arc<EventPageCell>>>,
    commit_bindings: LineageCommitBinding,
}
/// The checkpoint's indexed receipt for a commit, read lazily from its paged
/// commit index so admission never materializes every commit.
#[derive(Clone)]
pub struct LineageCommitBinding(
    pub Arc<dyn Fn(CommitId) -> Result<Option<RelationalCommitReceipt>, Cause> + Send + Sync>,
);
impl std::fmt::Debug for LineageCommitBinding {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("LineageCommitBinding")
    }
}
impl RuntimeLineageEventResidency {
    pub(super) fn new(
        page_descriptors: &[RelationalCheckpointLineageEventPage],
        source: &Arc<dyn RelationalDurableColdSource>,
        commits: LineageCommitBinding,
    ) -> Result<Self, RelationalStorageUnavailable> {
        let members = page_descriptors
            .iter()
            .map(|page| page.member)
            .collect::<Vec<_>>();
        let placement =
            source
                .retain_members(&members)
                .map_err(|cause| RelationalStorageUnavailable {
                    subject: RelationalStorageSubject::Lineage {
                        first: LineageId(
                            page_descriptors
                                .iter()
                                .map(|page| page.first_node)
                                .min()
                                .unwrap_or(0),
                        ),
                        last: LineageId(
                            page_descriptors
                                .iter()
                                .map(|page| page.last_node)
                                .max()
                                .unwrap_or(0),
                        ),
                    },
                    cause,
                })?;
        let mut pages = BTreeMap::new();
        let mut routes = BTreeMap::<_, Vec<_>>::new();
        for descriptor in page_descriptors {
            let cell = Arc::new(EventPageCell {
                descriptor: descriptor.clone(),
                state: Mutex::new(EventPageState::Cold),
                leases: AtomicUsize::new(0),
            });
            for route in &descriptor.routes {
                routes.entry(route.clone()).or_default().push(cell.clone());
            }
            pages.insert(descriptor.member, cell);
        }
        Ok(Self {
            custody: Mutex::new(NativeSourceMemberState::new(source.clone(), placement)),
            pages,
            routes,
            commit_bindings: commits,
        })
    }
    pub(super) fn member_subjects(
        &self,
    ) -> impl Iterator<Item = (RelationalDurableMemberLocator, RelationalStorageSubject)> + '_ {
        self.pages.values().map(|page| {
            (
                page.descriptor.member,
                RelationalStorageSubject::Lineage {
                    first: LineageId(page.descriptor.first_node),
                    last: LineageId(page.descriptor.last_node),
                },
            )
        })
    }
    pub(super) fn demand(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        nodes: Option<&BTreeSet<LineageId>>,
        branches: Option<&BTreeSet<BranchId>>,
    ) -> Result<Vec<ResidentLineageEventPage>, Failure> {
        crate::contracts::execution::request_custody::check_request(execution)
            .map_err(Failure::Request)?;
        let ranges = nodes.map(|nodes| {
            nodes
                .iter()
                .filter(|node| node.0 != 0)
                .map(|node| (node.0 - 1) / LINEAGE_NODE_RANGE_WIDTH)
                .collect::<BTreeSet<_>>()
        });
        let mut selected = BTreeMap::new();
        if ranges.is_none() && branches.is_none() {
            for (member, page) in &self.pages {
                selected.insert(*member, page);
            }
        } else {
            for (route, pages) in &self.routes {
                if ranges
                    .as_ref()
                    .is_some_and(|ranges| !ranges.contains(&route.node_range))
                    || branches.is_some_and(|branches| !branches.contains(&route.branch_id))
                {
                    continue;
                }
                for page in pages {
                    selected.entry(page.descriptor.member).or_insert(page);
                }
            }
        }
        selected
            .into_values()
            .map(|cell| {
                let source = self.source().map_err(|cause| cell.storage_failure(cause))?;
                cell.demand(execution, source.as_ref(), &self.commit_bindings)
            })
            .collect()
    }
}
impl EventPageCell {
    fn load_verified(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        source: &dyn RelationalDurableColdSource,
        bindings: &LineageCommitBinding,
    ) -> Result<ReadmittedLineageEventPage, Failure> {
        let bytes = if self.descriptor.member.length > LINEAGE_EVENT_PAGE_BYTES {
            Err(Cause::LimitExceeded {
                length: self.descriptor.member.length,
                limit: LINEAGE_EVENT_PAGE_BYTES,
            })
        } else {
            source.read_member(self.descriptor.member, LINEAGE_EVENT_PAGE_BYTES)
        }
        .map_err(|cause| event_page::failure(&self.descriptor, cause))?;
        let page = event_page::readmit(execution, &bytes, &self.descriptor)?;
        for row in &page.rows {
            let ok = (bindings.0)(row.publication_commit_id)
                .and_then(|pub_c| {
                    Ok(pub_c.is_some()
                        && (bindings.0)(row.event.commit().commit_id)?.as_ref()
                            == Some(row.event.commit()))
                })
                .map_err(|cause| event_page::failure(&self.descriptor, cause))?;
            if !ok {
                return Err(event_page::failure(&self.descriptor, Cause::Damaged));
            }
        }
        Ok(page)
    }

    fn demand(
        self: &Arc<Self>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        source: &dyn RelationalDurableColdSource,
        bindings: &LineageCommitBinding,
    ) -> Result<ResidentLineageEventPage, Failure> {
        crate::contracts::execution::request_custody::check_request(execution)
            .map_err(Failure::Request)?;
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        if matches!(*state, EventPageState::Cold) {
            match self.load_verified(execution, source, bindings) {
                Ok(page) => *state = EventPageState::Resident(Arc::new(page)),
                Err(Failure::StorageUnavailable(failure)) => {
                    if failure.cause.is_proven_damage() {
                        *state = EventPageState::Damaged(failure);
                    }
                    return Err(failure.into());
                }
                Err(stop) => return Err(stop),
            }
        }
        match &*state {
            EventPageState::Resident(page) => self.resident_lease(page.clone()).map_err(Into::into),
            EventPageState::Damaged(failure) => Err((*failure).into()),
            EventPageState::Cold => Err(event_page::failure(
                &self.descriptor,
                Cause::SourceUnavailable,
            )),
        }
    }
}
impl EventPageCell {
    fn storage_failure(&self, cause: Cause) -> RelationalStorageUnavailable {
        event_page::storage_failure(&self.descriptor, cause)
    }
    fn resident_lease(
        self: &Arc<Self>,
        page: Arc<ReadmittedLineageEventPage>,
    ) -> Result<ResidentLineageEventPage, RelationalStorageUnavailable> {
        {
            let update_counter = &self.leases;
            let mut count = update_counter.load(Ordering::Acquire);
            loop {
                let Some(updated_value) = count.checked_add(1) else {
                    break Err(count);
                };
                match update_counter.compare_exchange_weak(
                    count,
                    updated_value,
                    Ordering::AcqRel,
                    Ordering::Acquire,
                ) {
                    Ok(previous) => break Ok(previous),
                    Err(reloaded_value) => count = reloaded_value,
                }
            }
        }
        .map_err(|_| self.storage_failure(Cause::MemoryExhausted))?;
        Ok(ResidentLineageEventPage {
            cell: self.clone(),
            page,
        })
    }
    fn capture_resident(
        self: &Arc<Self>,
    ) -> Result<Option<ResidentLineageEventPage>, RelationalStorageUnavailable> {
        let state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        match &*state {
            EventPageState::Cold => Ok(None),
            EventPageState::Damaged(failure) => Err(*failure),
            EventPageState::Resident(page) => self.resident_lease(page.clone()).map(Some),
        }
    }
}
impl RuntimeLineageEventResidency {
    pub(super) fn capture_resident(
        &self,
    ) -> Result<(Vec<ResidentLineageEventPage>, bool), RelationalStorageUnavailable> {
        let mut leases = Vec::new();
        let mut complete = true;
        for page in self.pages.values() {
            if let Some(lease) = page.capture_resident()? {
                leases.push(lease);
            } else {
                complete = false;
            }
        }
        Ok((leases, complete))
    }
}
#[derive(Debug)]
pub struct ResidentLineageEventPage {
    cell: Arc<EventPageCell>,
    page: Arc<ReadmittedLineageEventPage>,
}
impl ResidentLineageEventPage {
    pub fn rows(&self) -> &[RelationalCheckpointLineageEventRow] {
        self.page.rows.elements()
    }
    pub(crate) fn member(&self) -> RelationalDurableMemberLocator {
        self.cell.descriptor.member
    }
    pub fn binding_failure(&self) -> RelationalStorageUnavailable {
        self.cell.storage_failure(Cause::Damaged)
    }
}
impl Drop for ResidentLineageEventPage {
    fn drop(&mut self) {
        let previous = self.cell.leases.fetch_sub(1, Ordering::AcqRel);
        debug_assert!(
            /* Proof: Each ResidentLineageEventPage constructor increments this page counter and owns a strong Arc; its non-Clone custody Drop decrements once. Loaded-page replacement cannot change the leased counter. */
            previous > 0
        );
    }
}
