use worth_relational::facade::runtime::RelationalAdjacencyDirection;

#[derive(Clone, Copy)]
pub(in crate::domain_computation::primary_graph) enum WorkflowEvidenceDependencyKind {
    Entity,
    EntityKind,
    AspectRevision,
    FieldRevision,
    AdjacencyRevision,
}

impl WorkflowEvidenceDependencyKind {
    pub(in crate::domain_computation::primary_graph) const fn encode(self) -> &'static str {
        match self {
            Self::Entity => "entity",
            Self::EntityKind => "entity-kind",
            Self::AspectRevision => "aspect-revision",
            Self::FieldRevision => "field-revision",
            Self::AdjacencyRevision => "adjacency-revision",
        }
    }

    pub(in crate::domain_computation::primary_graph) fn decode(value: &str) -> Option<Self> {
        match value {
            "entity" => Some(Self::Entity),
            "entity-kind" => Some(Self::EntityKind),
            "aspect-revision" => Some(Self::AspectRevision),
            "field-revision" => Some(Self::FieldRevision),
            "adjacency-revision" => Some(Self::AdjacencyRevision),
            _ => None,
        }
    }
}

pub(in crate::domain_computation::primary_graph) const fn encode_direction(
    direction: RelationalAdjacencyDirection,
) -> &'static str {
    match direction {
        RelationalAdjacencyDirection::Outgoing => "outgoing",
        RelationalAdjacencyDirection::Incoming => "incoming",
    }
}

pub(in crate::domain_computation::primary_graph) fn decode_direction(
    value: &str,
) -> Option<RelationalAdjacencyDirection> {
    match value {
        "outgoing" => Some(RelationalAdjacencyDirection::Outgoing),
        "incoming" => Some(RelationalAdjacencyDirection::Incoming),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kind_bound_entity_dependency_is_disjoint_from_legacy_liveness() {
        assert_eq!(WorkflowEvidenceDependencyKind::Entity.encode(), "entity");
        assert_eq!(
            WorkflowEvidenceDependencyKind::EntityKind.encode(),
            "entity-kind"
        );
        assert!(matches!(
            WorkflowEvidenceDependencyKind::decode("entity"),
            Some(WorkflowEvidenceDependencyKind::Entity)
        ));
        assert!(matches!(
            WorkflowEvidenceDependencyKind::decode("entity-kind"),
            Some(WorkflowEvidenceDependencyKind::EntityKind)
        ));
        assert!(WorkflowEvidenceDependencyKind::decode("entity-kind-v2").is_none());
    }
}
