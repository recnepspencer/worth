use super::WorthQueryApplicationProjectionNode;
use crate::domain_computation::primary_graph::application_query::{
    disclosure::WorthQueryApplicationQueryGovernance,
    resource_lifecycle::WorthQueryApplicationResultBufferReservation,
};
use worth_relational::facade::identity::EntityId;

pub(in crate::domain_computation::primary_graph::application_query) struct WorthQueryApplicationWorkingProjectionTree
{
    rows: Vec<WorthQueryApplicationProjectionNode>,
}

pub(in crate::domain_computation::primary_graph::application_query) struct WorthQueryApplicationDisclosedProjectionTree
{
    rows: Vec<WorthQueryApplicationProjectionNode>,
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
        result_buffer: &mut WorthQueryApplicationResultBufferReservation,
    ) -> WorthQueryApplicationDisclosedProjectionTree {
        let released_bytes = strip_undisclosed_nodes(&mut self.rows, governance);
        result_buffer.release_temporary(released_bytes);
        WorthQueryApplicationDisclosedProjectionTree { rows: self.rows }
    }
}

impl WorthQueryApplicationDisclosedProjectionTree {
    pub(in crate::domain_computation::primary_graph::application_query) fn raw_rows(
        &self,
    ) -> &[WorthQueryApplicationProjectionNode] {
        &self.rows
    }

    pub(in crate::domain_computation::primary_graph::application_query) fn capacity(
        &self,
    ) -> usize {
        self.rows.capacity()
    }

    pub(in crate::domain_computation::primary_graph::application_query) fn iter(
        &self,
    ) -> impl ExactSizeIterator<Item = WorthQueryApplicationDisclosedProjectionNode<'_>> {
        self.rows
            .iter()
            .map(WorthQueryApplicationDisclosedProjectionNode::new)
    }

    pub(in crate::domain_computation::primary_graph::application_query) fn only(
        &self,
    ) -> Option<WorthQueryApplicationDisclosedProjectionNode<'_>> {
        let [node] = self.rows.as_slice() else {
            return None;
        };
        Some(WorthQueryApplicationDisclosedProjectionNode::new(node))
    }

    pub(in crate::domain_computation::primary_graph::application_query) fn clear(&mut self) {
        self.rows.clear();
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
) -> usize {
    nodes.iter_mut().fold(0, |released, node| {
        released.saturating_add(strip_undisclosed_node(node, governance))
    })
}

fn strip_undisclosed_node(
    node: &mut WorthQueryApplicationProjectionNode,
    governance: &WorthQueryApplicationQueryGovernance,
) -> usize {
    let mut released = 0usize;
    node.fields.retain(|field| {
        if governance.is_disclosed(field.slot_key.as_ref()) {
            true
        } else {
            released = released.saturating_add(field.retained_bytes());
            false
        }
    });
    node.relations.retain_mut(|relation| {
        if governance.is_disclosed(relation.slot_key.as_ref()) {
            released =
                released.saturating_add(strip_undisclosed_nodes(&mut relation.rows, governance));
            true
        } else {
            released = released.saturating_add(relation.retained_bytes());
            false
        }
    });
    released
}
