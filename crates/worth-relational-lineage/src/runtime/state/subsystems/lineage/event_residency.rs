//! One counted, damage-isolated event cell is shared by all its node-range routes.
use crate::contracts::storage::data::storage_unavailable::{
    RelationalStorageSubject, RelationalStorageUnavailable,
    RelationalStorageUnavailableCause as Cause,
};
pub mod member_export;
use super::event_page::{self, ReadmittedLineageEventPage};
use crate::contracts::history::data::commit_receipt::RelationalCommitReceipt;
use crate::contracts::history::data::coordinates::{BranchId, CommitId};
use crate::contracts::identity::data::primitives::LineageId;
use crate::durability::data::lineage_event_pages::*;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;
use super::cold_page::{ColdPageCell, ColdPageDescriptor, ColdPageLease};
use super::source_retirement::LineagePageSource;
use worth_relational_storage::internal_port::storage::data::member_frame::MemberKind;
use worth_relational_storage::internal_port::durability::medium::cold_source::{
    RelationalDurableColdSource, RelationalDurableMemberLocator,
};
use worth_relational_storage::internal_port::durability::medium::member_custody::RelationalDurableMemberCustody;
use worth_relational_storage::internal_port::storage::residency::demand_failure::RelationalResidencyDemandFailure as Failure;

type EventPageCell = ColdPageCell<RelationalCheckpointLineageEventPage, ReadmittedLineageEventPage>;
type EventPageLease = ColdPageLease<RelationalCheckpointLineageEventPage, ReadmittedLineageEventPage>;

impl ColdPageDescriptor for RelationalCheckpointLineageEventPage {
    const BYTE_LIMIT: u64 = LINEAGE_EVENT_PAGE_BYTES;
    const KIND: MemberKind = MemberKind::LineageEvents;

    fn member(&self) -> worth_relational_storage::internal_port::durability::medium::cold_source::RelationalDurableMemberLocator {
        self.member
    }

    fn storage_failure(&self, cause: Cause) -> RelationalStorageUnavailable {
        event_page::storage_failure(self, cause)
    }
}

#[derive(Debug)]
pub(super) struct RuntimeLineageEventResidency {
    pub(super) custody: LineagePageSource,
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
        let custody = LineagePageSource::retain(source, &members)
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
            let cell = Arc::new(EventPageCell::new(descriptor.clone()));
            for route in &descriptor.routes {
                routes.entry(route.clone()).or_default().push(cell.clone());
            }
            pages.insert(descriptor.member, cell);
        }
        Ok(Self {
            custody,
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
                let source = self.custody.source().map_err(|cause| cell.descriptor.storage_failure(cause))?;
                cell.demand(execution, source.as_ref(), |bytes, descriptor| {
                    readmit_bound_events(execution, bytes, descriptor, &self.commit_bindings)
                }).map(ResidentLineageEventPage)
            })
            .collect()
    }
}
fn readmit_bound_events(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    bytes: &[u8],
    descriptor: &RelationalCheckpointLineageEventPage,
    bindings: &LineageCommitBinding,
) -> Result<ReadmittedLineageEventPage, Failure> {
        let page = event_page::readmit(execution, bytes, descriptor)?;
        for row in &page.rows {
            let ok = (bindings.0)(row.publication_commit_id)
                .and_then(|pub_c| {
                    Ok(pub_c.is_some()
                        && (bindings.0)(row.event.commit().commit_id)?.as_ref()
                            == Some(row.event.commit()))
                })
                .map_err(|cause| event_page::failure(descriptor, cause))?;
            if !ok {
                return Err(event_page::failure(descriptor, Cause::Damaged));
            }
        }
        Ok(page)
}
impl RuntimeLineageEventResidency {
    pub(super) fn capture_resident(
        &self,
    ) -> Result<(Vec<ResidentLineageEventPage>, bool), RelationalStorageUnavailable> {
        let mut leases = Vec::new();
        let mut complete = true;
        for page in self.pages.values() {
            if let Some(lease) = page.capture_resident()? {
                leases.push(ResidentLineageEventPage(lease));
            } else {
                complete = false;
            }
        }
        Ok((leases, complete))
    }
}
#[derive(Debug)]
pub struct ResidentLineageEventPage(EventPageLease);
impl ResidentLineageEventPage {
    pub fn rows(&self) -> &[RelationalCheckpointLineageEventRow] {
        self.0.page().rows.elements()
    }
    pub(crate) fn member(&self) -> RelationalDurableMemberLocator {
        self.0.cell().descriptor.member
    }
    pub fn binding_failure(&self) -> RelationalStorageUnavailable {
        self.0.cell().descriptor.storage_failure(Cause::Damaged)
    }
}
