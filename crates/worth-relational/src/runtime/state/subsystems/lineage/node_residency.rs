//! A node page has one shared load, sticky storage damage and counted readers.
mod member_export;
use super::node_page::{self, ReadmittedLineageNodePage};
use crate::durability::data::{RelationalCheckpointDirectory, RelationalCheckpointPage};
use crate::durability::medium::RelationalDurableColdSource;
use crate::identity::data::LineageId;
use crate::lineage::data::LineageNode;
use crate::storage::data::{
    RelationalStorageUnavailable, RelationalStorageUnavailableCause as Cause,
};
use crate::storage::residency::RelationalResidencyDemandFailure as Failure;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex,
};

#[derive(Debug)]
enum NodePageState {
    Cold,
    Resident(Arc<ReadmittedLineageNodePage>),
    Damaged(RelationalStorageUnavailable),
}

#[derive(Debug)]
struct NodePageCell {
    descriptor: RelationalCheckpointPage,
    state: Mutex<NodePageState>,
    leases: AtomicUsize,
}

#[derive(Debug)]
pub(super) struct RuntimeLineageNodeResidency {
    source: Arc<dyn RelationalDurableColdSource>,
    pages: Vec<Arc<NodePageCell>>,
    _placement: crate::durability::medium::RelationalDurableMemberCustody,
}

impl RuntimeLineageNodeResidency {
    pub(super) fn new(
        directory: &RelationalCheckpointDirectory,
    ) -> Result<Self, RelationalStorageUnavailable> {
        let members = directory
            .manifest
            .lineage
            .iter()
            .map(|page| page.member)
            .collect::<Vec<_>>();
        let placement = directory.source.retain_members(&members).map_err(|cause| {
            RelationalStorageUnavailable {
                subject: crate::storage::data::RelationalStorageSubject::Lineage {
                    first: LineageId(
                        directory
                            .manifest
                            .lineage
                            .first()
                            .map_or(0, |page| page.first),
                    ),
                    last: LineageId(
                        directory
                            .manifest
                            .lineage
                            .last()
                            .map_or(0, |page| page.last),
                    ),
                },
                cause,
            }
        })?;
        Ok(Self {
            _placement: placement,
            source: directory.source.clone(),
            pages: directory
                .manifest
                .lineage
                .iter()
                .map(|descriptor| {
                    Arc::new(NodePageCell {
                        descriptor: *descriptor,
                        state: Mutex::new(NodePageState::Cold),
                        leases: AtomicUsize::new(0),
                    })
                })
                .collect(),
        })
    }

    pub(super) fn covers(&self, id: LineageId) -> bool {
        self.page(id).is_some()
    }

    fn page(&self, id: LineageId) -> Option<&Arc<NodePageCell>> {
        let range = RelationalCheckpointPage::range_of(id.0);
        let index = self
            .pages
            .partition_point(|page| page.descriptor.range() < range);
        self.pages[index..]
            .iter()
            .take_while(|page| page.descriptor.range() == range)
            .find(|page| page.descriptor.contains(id.0))
    }

    pub(super) fn demand(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        id: LineageId,
    ) -> Result<Option<ResidentLineageNode>, Failure> {
        let Some(cell) = self.page(id) else {
            return Ok(None);
        };
        let page = cell.demand(execution, self.source.as_ref())?;
        let Ok(slot) = page
            .page
            .nodes
            .binary_search_by_key(&id, |node| node.lineage_id())
        else {
            return Ok(None);
        };
        Ok(Some(ResidentLineageNode {
            node: page.page.nodes[slot].clone(),
            _page: Some(page),
        }))
    }
}

impl NodePageCell {
    fn demand(
        self: &Arc<Self>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        source: &dyn RelationalDurableColdSource,
    ) -> Result<NodePageLease, Failure> {
        crate::execution::check_request(execution).map_err(Failure::Request)?;
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        if matches!(*state, NodePageState::Cold) {
            const PAGE_BYTE_LIMIT: u64 = 1 << 20;
            let result = if self.descriptor.member.length > PAGE_BYTE_LIMIT {
                Err(Cause::LimitExceeded {
                    length: self.descriptor.member.length,
                    limit: PAGE_BYTE_LIMIT,
                })
            } else {
                source.read_member(self.descriptor.member, PAGE_BYTE_LIMIT)
            }
            .map_err(|cause| node_page::failure(&self.descriptor, cause))
            .and_then(|bytes| node_page::readmit(execution, &bytes, &self.descriptor));
            match result {
                Ok(page) => *state = NodePageState::Resident(Arc::new(page)),
                Err(Failure::StorageUnavailable(failure)) => {
                    if failure.cause.is_proven_damage() {
                        *state = NodePageState::Damaged(failure);
                    }
                    return Err(failure.into());
                }
                Err(stop) => return Err(stop),
            }
        }
        match &*state {
            NodePageState::Resident(page) => {
                self.leases
                    .fetch_update(Ordering::AcqRel, Ordering::Acquire, |count| {
                        count.checked_add(1)
                    })
                    .map_err(|_| node_page::failure(&self.descriptor, Cause::MemoryExhausted))?;
                Ok(NodePageLease {
                    cell: self.clone(),
                    page: page.clone(),
                })
            }
            NodePageState::Damaged(failure) => Err((*failure).into()),
            NodePageState::Cold => {
                unreachable!("a successful page load installs resident contents")
            }
        }
    }
}

#[derive(Debug)]
struct NodePageLease {
    cell: Arc<NodePageCell>,
    page: Arc<ReadmittedLineageNodePage>,
}
impl Drop for NodePageLease {
    fn drop(&mut self) {
        let previous = self.cell.leases.fetch_sub(1, Ordering::AcqRel);
        debug_assert!(previous > 0);
    }
}

/// One selected immutable node, with its page retained while it can be read.
#[derive(Debug)]
pub(crate) struct ResidentLineageNode {
    node: LineageNode,
    _page: Option<NodePageLease>,
}
impl ResidentLineageNode {
    pub(super) fn published(node: LineageNode) -> Self {
        Self { node, _page: None }
    }
    pub(crate) fn node(&self) -> &LineageNode {
        &self.node
    }
    pub(crate) fn reject_binding(&self) -> Failure {
        if let Some(page) = &self._page {
            let failure = node_page::failure(&page.cell.descriptor, Cause::Damaged);
            if let Failure::StorageUnavailable(failure) = failure {
                *page
                    .cell
                    .state
                    .lock()
                    .unwrap_or_else(|error| error.into_inner()) = NodePageState::Damaged(failure);
                return failure.into();
            }
            unreachable!("a node binding refusal is a storage failure");
        }
        crate::storage::data::RelationalStorageUnavailable {
            subject: crate::storage::data::RelationalStorageSubject::Lineage {
                first: self.node.lineage_id(),
                last: self.node.lineage_id(),
            },
            cause: Cause::Damaged,
        }
        .into()
    }
    pub(super) fn page_nodes(&self) -> &[LineageNode] {
        self._page
            .as_ref()
            .map_or(std::slice::from_ref(&self.node), |page| {
                page.page.nodes.elements()
            })
    }
}
