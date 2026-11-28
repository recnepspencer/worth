//! A node page has one shared load, sticky storage damage and counted readers.
use crate::contracts::storage::data::storage_unavailable::{
    RelationalStorageUnavailable, RelationalStorageUnavailableCause as Cause,
};
pub mod member_export;
use super::node_page::{self, ReadmittedLineageNodePage};
use crate::contracts::identity::data::primitives::LineageId;
use crate::contracts::lineage::data::graph::LineageNode;
use std::sync::Arc;
use super::cold_page::{ColdPageCell, ColdPageDescriptor, ColdPageLease};
use super::source_retirement::LineagePageSource;
use worth_relational_storage::internal_port::storage::data::member_frame::MemberKind;
use worth_relational_storage::internal_port::durability::medium::cold_source::RelationalDurableColdSource;
use worth_relational_storage::internal_port::storage::data::checkpoint_page::RelationalCheckpointPage;
use worth_relational_storage::internal_port::storage::residency::demand_failure::RelationalResidencyDemandFailure as Failure;

type NodePageCell = ColdPageCell<RelationalCheckpointPage, ReadmittedLineageNodePage>;
type NodePageLease = ColdPageLease<RelationalCheckpointPage, ReadmittedLineageNodePage>;

impl ColdPageDescriptor for RelationalCheckpointPage {
    const BYTE_LIMIT: u64 = 1 << 20;
    const KIND: MemberKind = MemberKind::Lineage;

    fn member(&self) -> worth_relational_storage::internal_port::durability::medium::cold_source::RelationalDurableMemberLocator {
        self.member
    }

    fn storage_failure(&self, cause: Cause) -> RelationalStorageUnavailable {
        node_page::storage_failure(self, cause)
    }
}

#[derive(Debug)]
pub(super) struct RuntimeLineageNodeResidency {
    pub(super) custody: LineagePageSource,
    pages: Vec<Arc<NodePageCell>>,
}

impl RuntimeLineageNodeResidency {
    pub(super) fn new(
        page_descriptors: &[RelationalCheckpointPage],
        source: &Arc<dyn RelationalDurableColdSource>,
    ) -> Result<Self, RelationalStorageUnavailable> {
        let members = page_descriptors
            .iter()
            .map(|page| page.member)
            .collect::<Vec<_>>();
        let custody = LineagePageSource::retain(source, &members)
                .map_err(|cause| RelationalStorageUnavailable {
                    subject: crate::contracts::storage::data::storage_unavailable::RelationalStorageSubject::Lineage {
                        first: LineageId(page_descriptors.first().map_or(0, |page| page.first)),
                        last: LineageId(page_descriptors.last().map_or(0, |page| page.last)),
                    },
                    cause,
                })?;
        Ok(Self {
            custody,
            pages: page_descriptors
                .iter()
                .map(|descriptor| {
                    Arc::new(NodePageCell::new(*descriptor))
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
        let source = self
            .custody.source()
            .map_err(|cause| node_page::failure(&cell.descriptor, cause))?;
        let page = cell.demand(execution, source.as_ref(), |bytes, descriptor| {
            node_page::readmit(execution, bytes, descriptor)
        })?;
        let Ok(slot) = page
            .page()
            .nodes
            .binary_search_by_key(&id, |node| node.lineage_id())
        else {
            return Ok(None);
        };
        Ok(Some(ResidentLineageNode {
            node: page.page().nodes[slot].clone(),
            _page: Some(page),
        }))
    }
}

/// One selected immutable node, with its page retained while it can be read.
#[derive(Debug)]
pub struct ResidentLineageNode {
    node: LineageNode,
    _page: Option<NodePageLease>,
}
impl ResidentLineageNode {
    pub(super) fn published(node: LineageNode) -> Self {
        Self { node, _page: None }
    }
    pub fn node(&self) -> &LineageNode {
        &self.node
    }
    pub fn reject_binding(&self) -> Failure {
        if let Some(page) = &self._page {
            return page.cell().reject_binding().into();
        }
        crate::contracts::storage::data::storage_unavailable::RelationalStorageUnavailable {
            subject: crate::contracts::storage::data::storage_unavailable::RelationalStorageSubject::Lineage {
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
                page.page().nodes.elements()
            })
    }
}
