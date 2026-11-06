use worth_query_installation::facade::{
    ApplicationRelationCardinality, ApplicationRelationCrossContextPolicy,
    ApplicationRelationDeletionPolicy, ApplicationRelationEndpoints, ApplicationRelationIntegrity,
};

pub(in crate::domain_computation::graph_layout::workflow) fn live_membership_integrity(
) -> ApplicationRelationIntegrity {
    integrity(ApplicationRelationCardinality::new(
        None,
        Some(1),
        None,
        None,
        None,
        Some(1),
    ))
}

/// A lineage has at most one current definition. Retirement leaves it with
/// none, which denies new starts until the lineage is explicitly reopened.
pub(in crate::domain_computation::graph_layout::workflow) fn current_definition_integrity(
) -> ApplicationRelationIntegrity {
    integrity(ApplicationRelationCardinality::new(
        None,
        Some(1),
        None,
        Some(1),
        None,
        Some(1),
    ))
}

/// Every definition names exactly one start node, and a node starts at most
/// one definition.
pub(in crate::domain_computation::graph_layout::workflow) fn definition_start_integrity(
) -> ApplicationRelationIntegrity {
    integrity(ApplicationRelationCardinality::new(
        Some(1),
        Some(1),
        None,
        Some(1),
        None,
        Some(1),
    ))
}

pub(in crate::domain_computation::graph_layout::workflow) fn connection_endpoint_integrity(
) -> ApplicationRelationIntegrity {
    integrity(ApplicationRelationCardinality::new(
        Some(1),
        Some(1),
        None,
        None,
        None,
        Some(1),
    ))
}

pub(in crate::domain_computation::graph_layout::workflow) fn owned_fact_integrity(
) -> ApplicationRelationIntegrity {
    integrity(ApplicationRelationCardinality::new(
        None,
        None,
        Some(1),
        Some(1),
        None,
        Some(1),
    ))
}

pub(in crate::domain_computation::graph_layout::workflow) fn workflow_proposal_integrity(
) -> ApplicationRelationIntegrity {
    evidence_integrity()
}

pub(in crate::domain_computation::graph_layout::workflow) fn assessment_evidence_integrity(
) -> ApplicationRelationIntegrity {
    evidence_integrity()
}

pub(in crate::domain_computation::graph_layout::workflow) fn approval_evidence_integrity(
) -> ApplicationRelationIntegrity {
    integrity(ApplicationRelationCardinality::new(
        Some(1),
        None,
        None,
        None,
        None,
        Some(1),
    ))
}

pub(in crate::domain_computation::graph_layout::workflow) fn approval_proposal_integrity(
) -> ApplicationRelationIntegrity {
    integrity(ApplicationRelationCardinality::new(
        Some(1),
        Some(1),
        None,
        None,
        None,
        Some(1),
    ))
}

/// One successor names one predecessor, and a predecessor has at most one successor.
pub(in crate::domain_computation::graph_layout::workflow) fn single_successor_integrity(
) -> ApplicationRelationIntegrity {
    integrity(ApplicationRelationCardinality::new(
        None,
        Some(1),
        None,
        Some(1),
        None,
        Some(1),
    ))
}

/// A successor inherits any number of performed transitions, and a chain of
/// migrations references the same performed transition from each successor.
pub(in crate::domain_computation::graph_layout::workflow) fn prior_effect_integrity(
) -> ApplicationRelationIntegrity {
    integrity(ApplicationRelationCardinality::new(
        None,
        None,
        None,
        None,
        None,
        Some(1),
    ))
}

fn evidence_integrity() -> ApplicationRelationIntegrity {
    integrity(ApplicationRelationCardinality::new(
        None,
        Some(1),
        Some(1),
        Some(1),
        None,
        Some(1),
    ))
}

/// A v3 instance binds one actual application subject. A v2 instance has no
/// relation, so the versioned native reader enforces presence, not a catalog
/// minimum that would reject historical records. Definition and subject may
/// live in different contexts; only native enrollment creates this relation.
pub(in crate::domain_computation::graph_layout::workflow) fn subject_integrity(
) -> ApplicationRelationIntegrity {
    ApplicationRelationIntegrity::new(
        ApplicationRelationEndpoints::new(
            false,
            ApplicationRelationCrossContextPolicy::AllowExplicit,
        ),
        ApplicationRelationCardinality::new(None, Some(1), None, None, None, Some(1)),
        ApplicationRelationDeletionPolicy::RequireRelationDeletionInSameCommit,
    )
}

fn integrity(cardinality: ApplicationRelationCardinality) -> ApplicationRelationIntegrity {
    ApplicationRelationIntegrity::new(
        ApplicationRelationEndpoints::new(false, ApplicationRelationCrossContextPolicy::Forbid),
        cardinality,
        ApplicationRelationDeletionPolicy::RequireRelationDeletionInSameCommit,
    )
}
