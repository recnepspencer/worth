use worth_query_installation::facade::{
    ApplicationRelationCardinality, ApplicationRelationCrossContextPolicy,
    ApplicationRelationDeletionPolicy, ApplicationRelationEndpoints, ApplicationRelationIntegrity,
};

pub(in crate::domain_computation::primary_graph::workflow::schema) fn live_membership_integrity(
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
pub(in crate::domain_computation::primary_graph::workflow::schema) fn current_definition_integrity(
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
pub(in crate::domain_computation::primary_graph::workflow::schema) fn definition_start_integrity(
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

pub(in crate::domain_computation::primary_graph::workflow::schema) fn connection_endpoint_integrity(
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

pub(in crate::domain_computation::primary_graph::workflow::schema) fn owned_fact_integrity(
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

pub(in crate::domain_computation::primary_graph::workflow::schema) fn workflow_proposal_integrity(
) -> ApplicationRelationIntegrity {
    evidence_integrity()
}

pub(in crate::domain_computation::primary_graph::workflow::schema) fn assessment_evidence_integrity(
) -> ApplicationRelationIntegrity {
    evidence_integrity()
}

pub(in crate::domain_computation::primary_graph::workflow::schema) fn approval_evidence_integrity(
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

pub(in crate::domain_computation::primary_graph::workflow::schema) fn approval_proposal_integrity(
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

/// A successor names the one instance its migration ended, and an instance
/// is migrated at most once.
pub(in crate::domain_computation::primary_graph::workflow::schema) fn migration_successor_integrity(
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
pub(in crate::domain_computation::primary_graph::workflow::schema) fn prior_effect_integrity(
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

fn integrity(cardinality: ApplicationRelationCardinality) -> ApplicationRelationIntegrity {
    ApplicationRelationIntegrity::new(
        ApplicationRelationEndpoints::new(false, ApplicationRelationCrossContextPolicy::Forbid),
        cardinality,
        ApplicationRelationDeletionPolicy::RequireRelationDeletionInSameCommit,
    )
}
