use super::super::retained_projection::{storage::DisclosedRows, OriginalProjection};
use super::{OwnedProjectionRows, WorthQueryApplicationProjectionNode};
use crate::ports::QueryGovernancePort as WorthQueryApplicationQueryGovernance;
use worth_relational::facade::identity::EntityId;

pub struct WorthQueryApplicationWorkingProjectionTree {
    rows: Vec<WorthQueryApplicationProjectionNode>,
}

pub struct WorthQueryApplicationDisclosedProjectionTree {
    rows: DisclosedRows,
}

#[derive(Clone, Copy)]
pub struct WorthQueryApplicationDisclosedProjectionNode<
    'a,
> {
    node: &'a WorthQueryApplicationProjectionNode,
}

impl WorthQueryApplicationWorkingProjectionTree {
    pub fn new(rows: Vec<WorthQueryApplicationProjectionNode>) -> Self {
        Self { rows }
    }

    pub fn into_disclosed(
        mut self,
        governance: &dyn WorthQueryApplicationQueryGovernance,
    ) -> WorthQueryApplicationDisclosedProjectionTree {
        if !governance.is_public() {
            strip_undisclosed_nodes(&mut self.rows, governance);
        }

        WorthQueryApplicationDisclosedProjectionTree {
            rows: DisclosedRows::Owned(OwnedProjectionRows::ordinary(self.rows)),
        }
    }
}

impl WorthQueryApplicationDisclosedProjectionTree {
    pub fn from_managed(rows: OwnedProjectionRows) -> Self {
        Self {
            rows: DisclosedRows::Owned(rows),
        }
    }

    pub fn raw_rows(&self) -> &[WorthQueryApplicationProjectionNode] {
        self.rows.rows()
    }

    pub fn iter(
        &self,
    ) -> impl ExactSizeIterator<Item = WorthQueryApplicationDisclosedProjectionNode<'_>> {
        self.rows
            .rows()
            .iter()
            .map(WorthQueryApplicationDisclosedProjectionNode::new)
    }

    pub fn at(
        &self,
        ordinal: usize,
    ) -> Option<WorthQueryApplicationDisclosedProjectionNode<'_>> {
        self.rows
            .rows()
            .get(ordinal)
            .map(WorthQueryApplicationDisclosedProjectionNode::new)
    }

    pub fn only(
        &self,
    ) -> Option<WorthQueryApplicationDisclosedProjectionNode<'_>> {
        let [node] = self.rows.rows().as_slice() else {
            return None;
        };
        Some(WorthQueryApplicationDisclosedProjectionNode::new(node))
    }

    pub fn take(&mut self) -> Self {
        Self {
            rows: std::mem::replace(
                &mut self.rows,
                DisclosedRows::Owned(OwnedProjectionRows::default()),
            ),
        }
    }

    pub fn clear(&mut self) {
        self.rows.clear();
    }

    pub fn retain_projection(
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

    pub const fn entity_id(
        &self,
    ) -> EntityId {
        self.node.entity_id()
    }

    pub fn field(
        &self,
        slot_type: &str,
    ) -> Option<&'a super::WorthQueryApplicationProjectedField> {
        self.node.field(slot_type)
    }

    pub fn relation(
        &self,
        slot_type: &str,
    ) -> Option<&'a super::WorthQueryApplicationProjectedRelation> {
        self.node.relation(slot_type)
    }

    pub fn child(
        &self,
        node: &'a WorthQueryApplicationProjectionNode,
    ) -> Self {
        Self::new(node)
    }
}

fn strip_undisclosed_nodes(
    nodes: &mut [WorthQueryApplicationProjectionNode],
    governance: &dyn WorthQueryApplicationQueryGovernance,
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
        self.rows.payload_bytes() as u64
    }
}

mod court_doors_for_test;
