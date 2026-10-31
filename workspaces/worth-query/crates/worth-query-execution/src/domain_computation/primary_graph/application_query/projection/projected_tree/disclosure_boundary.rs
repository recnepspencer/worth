use super::super::retained_projection::{storage::DisclosedRows, OriginalProjection};
use super::WorthQueryApplicationProjectionNode;
use crate::domain_computation::primary_graph::application_query::disclosure::WorthQueryApplicationQueryGovernance;
use worth_relational::facade::identity::EntityId;

pub(in crate::domain_computation::primary_graph::application_query) struct WorthQueryApplicationWorkingProjectionTree
{
    rows: Vec<WorthQueryApplicationProjectionNode>,
}

pub(in crate::domain_computation::primary_graph::application_query) struct WorthQueryApplicationDisclosedProjectionTree
{
    rows: DisclosedRows,
}

#[derive(Clone, Copy)]
pub(in crate::domain_computation::primary_graph::application_query) struct WorthQueryApplicationDisclosedProjectionNode<
    'a,
> {
    node: &'a WorthQueryApplicationProjectionNode,
}

impl WorthQueryApplicationWorkingProjectionTree {
    pub(in crate::domain_computation::primary_graph::application_query) fn new(
        rows: Vec<WorthQueryApplicationProjectionNode>,
    ) -> Self {
        Self { rows }
    }

    pub(in crate::domain_computation::primary_graph::application_query) fn into_disclosed(
        mut self,
        governance: &WorthQueryApplicationQueryGovernance,
    ) -> WorthQueryApplicationDisclosedProjectionTree {
        strip_undisclosed_nodes(&mut self.rows, governance);

        WorthQueryApplicationDisclosedProjectionTree {
            rows: DisclosedRows::Owned(self.rows),
        }
    }
}

impl WorthQueryApplicationDisclosedProjectionTree {
    pub(in crate::domain_computation::primary_graph::application_query) fn raw_rows(
        &self,
    ) -> &[WorthQueryApplicationProjectionNode] {
        self.rows.rows()
    }

    pub(in crate::domain_computation::primary_graph::application_query) fn iter(
        &self,
    ) -> impl ExactSizeIterator<Item = WorthQueryApplicationDisclosedProjectionNode<'_>> {
        self.rows
            .rows()
            .iter()
            .map(WorthQueryApplicationDisclosedProjectionNode::new)
    }

    pub(in crate::domain_computation::primary_graph::application_query) fn at(
        &self,
        ordinal: usize,
    ) -> Option<WorthQueryApplicationDisclosedProjectionNode<'_>> {
        self.rows
            .rows()
            .get(ordinal)
            .map(WorthQueryApplicationDisclosedProjectionNode::new)
    }

    pub(in crate::domain_computation::primary_graph::application_query) fn only(
        &self,
    ) -> Option<WorthQueryApplicationDisclosedProjectionNode<'_>> {
        let [node] = self.rows.rows().as_slice() else {
            return None;
        };
        Some(WorthQueryApplicationDisclosedProjectionNode::new(node))
    }

    pub(in crate::domain_computation::primary_graph::application_query) fn take(&mut self) -> Self {
        Self {
            rows: std::mem::replace(&mut self.rows, DisclosedRows::Owned(Vec::new())),
        }
    }

    pub(in crate::domain_computation::primary_graph::application_query) fn clear(&mut self) {
        self.rows.clear();
    }

    pub(in crate::domain_computation::primary_graph::application_query) fn retain_projection(
        &mut self,
        disclosure: worth_execution::ExecutionArray<(
            std::sync::Arc<
                worth_query_declaration::facade::application_query::ApplicationQueryResultSlotKey,
            >,
            bool,
        )>,
        memory: Option<worth_execution::ExecutionMemoryReservation>,
    ) -> Option<std::sync::Arc<OriginalProjection>> {
        self.rows.retain(disclosure, memory)
    }
}

impl<'a> WorthQueryApplicationDisclosedProjectionNode<'a> {
    fn new(node: &'a WorthQueryApplicationProjectionNode) -> Self {
        Self { node }
    }

    pub(in crate::domain_computation::primary_graph::application_query) const fn entity_id(
        &self,
    ) -> EntityId {
        self.node.entity_id()
    }

    pub(in crate::domain_computation::primary_graph::application_query) fn field(
        &self,
        slot_type: &str,
    ) -> Option<&'a super::WorthQueryApplicationProjectedField> {
        self.node.field(slot_type)
    }

    pub(in crate::domain_computation::primary_graph::application_query) fn relation(
        &self,
        slot_type: &str,
    ) -> Option<&'a super::WorthQueryApplicationProjectedRelation> {
        self.node.relation(slot_type)
    }

    pub(in crate::domain_computation::primary_graph::application_query) fn child(
        &self,
        node: &'a WorthQueryApplicationProjectionNode,
    ) -> Self {
        Self::new(node)
    }
}

fn strip_undisclosed_nodes(
    nodes: &mut [WorthQueryApplicationProjectionNode],
    governance: &WorthQueryApplicationQueryGovernance,
) {
    for node in nodes {
        node.fields
            .retain(|field| governance.is_disclosed(field.slot_key.as_ref()));
        node.relations.retain_mut(|relation| {
            if !governance.is_disclosed(relation.slot_key.as_ref()) {
                return false;
            }
            strip_undisclosed_nodes(&mut relation.rows, governance);
            true
        });
    }
}

impl worth_execution::ChargedBytes for WorthQueryApplicationDisclosedProjectionTree {
    fn additional_charged_bytes(&self) -> u64 {
        let Self { rows } = self;
        let rows = rows.rows();
        rows.iter().fold(
            (rows.capacity() as u64)
                .saturating_mul(std::mem::size_of::<WorthQueryApplicationProjectionNode>() as u64),
            |bytes, row| bytes.saturating_add(row.retained_bytes() as u64),
        )
    }
}
