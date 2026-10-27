//! Physical field and adjacency revisions; descriptive evidence, never authority.
use worth_foundational::facade::{AspectContractRevision, AspectKey, FieldKey};
use worth_relational::facade::identity::{EntityId, KindId, VersionId};

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(in crate::domain_computation) struct WorthQueryObservedFieldRevision {
    pub(in crate::domain_computation::primary_graph) entity: EntityId,
    pub(in crate::domain_computation::primary_graph) entity_name: String,
    pub(in crate::domain_computation::primary_graph) aspect: AspectKey,
    pub(in crate::domain_computation::primary_graph) field: FieldKey,
    pub(in crate::domain_computation::primary_graph) contract_revision: AspectContractRevision,
    pub(in crate::domain_computation::primary_graph) native_revision:
        Option<worth_relational::facade::runtime::RelationalFieldRevision>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::domain_computation) struct WorthQueryObservedAdjacencyRevision {
    pub(in crate::domain_computation::primary_graph) anchor: EntityId,
    pub(in crate::domain_computation::primary_graph) relation_kind: KindId,
    pub(in crate::domain_computation::primary_graph) direction:
        worth_relational::facade::runtime::RelationalAdjacencyDirection,
    pub(in crate::domain_computation::primary_graph) native_revision: Option<VersionId>,
    pub(in crate::domain_computation::primary_graph) comparison_work_limit: usize,
    pub(in crate::domain_computation::primary_graph) endpoints: Vec<EntityId>,
}

impl Ord for WorthQueryObservedAdjacencyRevision {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (
            self.anchor,
            self.relation_kind,
            adjacency_direction_rank(self.direction),
            self.native_revision,
            self.comparison_work_limit,
            &self.endpoints,
        )
            .cmp(&(
                other.anchor,
                other.relation_kind,
                adjacency_direction_rank(other.direction),
                other.native_revision,
                other.comparison_work_limit,
                &other.endpoints,
            ))
    }
}

impl PartialOrd for WorthQueryObservedAdjacencyRevision {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

pub(in crate::domain_computation::primary_graph) const fn adjacency_direction_rank(
    direction: worth_relational::facade::runtime::RelationalAdjacencyDirection,
) -> u8 {
    match direction {
        worth_relational::facade::runtime::RelationalAdjacencyDirection::Outgoing => 0,
        worth_relational::facade::runtime::RelationalAdjacencyDirection::Incoming => 1,
    }
}
