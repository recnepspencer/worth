use super::{
    assessment_evidence_integrity, connection_endpoint_integrity, current_definition_integrity,
    definition_start_integrity, owned_fact_integrity, workflow_proposal_integrity,
};

#[test]
fn workflow_relation_roles_have_distinct_exact_cardinality() {
    let current = current_definition_integrity().cardinality;
    assert_eq!((current.source_min, current.source_max), (None, Some(1)));
    assert_eq!(current.target_max, Some(1));

    let start = definition_start_integrity().cardinality;
    assert_eq!((start.source_min, start.source_max), (Some(1), Some(1)));
    assert_eq!(start.target_max, Some(1));

    let owned = owned_fact_integrity().cardinality;
    assert_eq!((owned.target_min, owned.target_max), (Some(1), Some(1)));
    assert_eq!(owned.source_max, None);

    let endpoint = connection_endpoint_integrity().cardinality;
    assert_eq!(
        (endpoint.source_min, endpoint.source_max),
        (Some(1), Some(1))
    );
    assert_eq!(endpoint.target_max, None);

    let proposal = workflow_proposal_integrity().cardinality;
    assert_eq!((proposal.source_min, proposal.source_max), (None, Some(1)));
    assert_eq!(
        (proposal.target_min, proposal.target_max),
        (Some(1), Some(1))
    );

    let evidence = assessment_evidence_integrity().cardinality;
    assert_eq!((evidence.source_min, evidence.source_max), (None, Some(1)));
    assert_eq!(
        (evidence.target_min, evidence.target_max),
        (Some(1), Some(1))
    );
}
