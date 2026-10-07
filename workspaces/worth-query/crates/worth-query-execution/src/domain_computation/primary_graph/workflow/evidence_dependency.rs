use worth_relational::facade::runtime::RelationalAdjacencyDirection;

pub(in crate::domain_computation::primary_graph) const OBSERVED_FACTS_PER_EVIDENCE_DEPENDENCY:
    usize = 13;

pub(in crate::domain_computation::primary_graph) const fn evidence_dependency_observation_facts(
    dependency_count: usize,
) -> usize {
    dependency_count
        .saturating_mul(OBSERVED_FACTS_PER_EVIDENCE_DEPENDENCY)
        .saturating_add(1)
}

pub(in crate::domain_computation::primary_graph) const fn maximum_evidence_dependencies(
    fact_budget: usize,
) -> usize {
    fact_budget.saturating_sub(1) / OBSERVED_FACTS_PER_EVIDENCE_DEPENDENCY
}

pub(in crate::domain_computation::primary_graph) const fn evidence_dependency_adjacency_work(
    dependency_count: usize,
) -> usize {
    dependency_count.saturating_mul(2).saturating_add(1)
}

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
    fn dependency_readback_cost_admits_the_exact_bound_and_rejects_one_over() {
        let two_dependencies = evidence_dependency_observation_facts(2);
        // One dependency row, four required fields, and eight optional
        // presence/value observations are retained per dependency.
        assert_eq!(two_dependencies, 27);
        assert_eq!(maximum_evidence_dependencies(two_dependencies), 2);
        assert_eq!(maximum_evidence_dependencies(two_dependencies - 1), 1);
        assert_eq!(evidence_dependency_adjacency_work(2), 5);
        assert_eq!(
            evidence_dependency_observation_facts(3) - two_dependencies,
            OBSERVED_FACTS_PER_EVIDENCE_DEPENDENCY
        );
    }

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
