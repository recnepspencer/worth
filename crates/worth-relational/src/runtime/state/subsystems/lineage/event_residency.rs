//! One counted, damage-isolated event cell is shared by all its node-range routes.
mod member_export;
use super::event_page::{self, ReadmittedLineageEventPage};
use crate::durability::data::*;
use crate::durability::medium::{
    RelationalDurableColdSource, RelationalDurableMemberCustody, RelationalDurableMemberLocator,
};
use crate::history::data::{BranchId, CommitId, RelationalCommitReceipt};
use crate::identity::data::LineageId;
use crate::storage::data::{
    RelationalStorageUnavailable, RelationalStorageUnavailableCause as Cause,
};
use crate::storage::residency::RelationalResidencyDemandFailure as Failure;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex,
};

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
    source: Arc<dyn RelationalDurableColdSource>,
    pages: BTreeMap<RelationalDurableMemberLocator, Arc<EventPageCell>>,
    routes: BTreeMap<RelationalLineageEventPageRoute, Vec<Arc<EventPageCell>>>,
    commit_bindings: BTreeMap<CommitId, RelationalCommitReceipt>,
    _placement: RelationalDurableMemberCustody,
}
impl RuntimeLineageEventResidency {
    pub(super) fn new(
        directory: &RelationalCheckpointDirectory,
    ) -> Result<Self, RelationalStorageUnavailable> {
        let members = directory
            .manifest
            .lineage_events
            .iter()
            .map(|page| page.member)
            .collect::<Vec<_>>();
        let placement = directory.source.retain_members(&members).map_err(|cause| {
            RelationalStorageUnavailable {
                subject: crate::storage::data::RelationalStorageSubject::Lineage {
                    first: LineageId(
                        directory
                            .manifest
                            .lineage_events
                            .iter()
                            .map(|page| page.first_node)
                            .min()
                            .unwrap_or(0),
                    ),
                    last: LineageId(
                        directory
                            .manifest
                            .lineage_events
                            .iter()
                            .map(|page| page.last_node)
                            .max()
                            .unwrap_or(0),
                    ),
                },
                cause,
            }
        })?;
        let mut pages = BTreeMap::new();
        let mut routes = BTreeMap::<_, Vec<_>>::new();
        for descriptor in &directory.manifest.lineage_events {
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
            source: directory.source.clone(),
            pages,
            routes,
            commit_bindings: directory
                .manifest
                .commits
                .iter()
                .map(|entry| (entry.commit.commit_id, entry.commit.clone()))
                .collect(),
            _placement: placement,
        })
    }
    pub(super) fn member_subjects(
        &self,
    ) -> impl Iterator<
        Item = (
            RelationalDurableMemberLocator,
            crate::storage::data::RelationalStorageSubject,
        ),
    > + '_ {
        self.pages.values().map(|page| {
            (
                page.descriptor.member,
                crate::storage::data::RelationalStorageSubject::Lineage {
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
        crate::execution::check_request(execution).map_err(Failure::Request)?;
        let ranges = nodes.map(|nodes| {
            nodes
                .iter()
                .filter(|node| node.0 != 0)
                .map(|node| (node.0 - 1) / LINEAGE_NODE_RANGE_WIDTH)
                .collect::<BTreeSet<_>>()
        });
        let mut selected = BTreeMap::new();
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
        selected
            .into_values()
            .map(|cell| cell.demand(execution, self.source.as_ref(), &self.commit_bindings))
            .collect()
    }
}
impl EventPageCell {
    fn demand(
        self: &Arc<Self>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        source: &dyn RelationalDurableColdSource,
        bindings: &BTreeMap<CommitId, RelationalCommitReceipt>,
    ) -> Result<ResidentLineageEventPage, Failure> {
        crate::execution::check_request(execution).map_err(Failure::Request)?;
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        if matches!(*state, EventPageState::Cold) {
            let loaded = if self.descriptor.member.length > LINEAGE_EVENT_PAGE_BYTES {
                Err(Cause::LimitExceeded {
                    length: self.descriptor.member.length,
                    limit: LINEAGE_EVENT_PAGE_BYTES,
                })
            } else {
                source.read_member(self.descriptor.member, LINEAGE_EVENT_PAGE_BYTES)
            }
            .map_err(|cause| event_page::failure(&self.descriptor, cause))
            .and_then(|bytes| event_page::readmit(execution, &bytes, &self.descriptor))
            .and_then(|page| {
                if page.rows.iter().any(|row| {
                    !bindings.contains_key(&row.publication_commit_id)
                        || bindings.get(&row.event.commit().commit_id) != Some(row.event.commit())
                }) {
                    Err(event_page::failure(&self.descriptor, Cause::Damaged))
                } else {
                    Ok(page)
                }
            });
            match loaded {
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
            EventPageState::Cold => unreachable!("successful event demand installs its page"),
        }
    }
}
impl EventPageCell {
    fn storage_failure(&self, cause: Cause) -> RelationalStorageUnavailable {
        let Failure::StorageUnavailable(failure) = event_page::failure(&self.descriptor, cause)
        else {
            unreachable!()
        };
        failure
    }
    fn resident_lease(
        self: &Arc<Self>,
        page: Arc<ReadmittedLineageEventPage>,
    ) -> Result<ResidentLineageEventPage, RelationalStorageUnavailable> {
        self.leases
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |count| {
                count.checked_add(1)
            })
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
pub(crate) struct ResidentLineageEventPage {
    cell: Arc<EventPageCell>,
    page: Arc<ReadmittedLineageEventPage>,
}
impl ResidentLineageEventPage {
    pub(crate) fn rows(&self) -> &[RelationalCheckpointLineageEventRow] {
        self.page.rows.elements()
    }
    pub(crate) fn member(&self) -> RelationalDurableMemberLocator {
        self.cell.descriptor.member
    }
    pub(crate) fn binding_failure(&self) -> RelationalStorageUnavailable {
        self.cell.storage_failure(Cause::Damaged)
    }
}
impl Drop for ResidentLineageEventPage {
    fn drop(&mut self) {
        let previous = self.cell.leases.fetch_sub(1, Ordering::AcqRel);
        debug_assert!(previous > 0);
    }
}
