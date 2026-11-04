use worth_relational::facade::identity::EntityId;
use worth_relational::facade::identity::PartitionId;

use worth_query_execution::publication_boundary::workflow_advance::RequiredWorkflowAssessment;

#[test]
fn exact_requirement_distinguishes_proposal_and_coverage_with_same_execution_affinity() {
    let original = RequiredWorkflowAssessment {
        instance: EntityId::new(PartitionId::main(), 1, 1),
        node_path: "review".to_owned(),
        transition_identity: "transition".to_owned(),
        occurrence: 1,
        query: "query".to_owned(),
        parameter_type: "parameters".to_owned(),
        result_type: "result".to_owned(),
        binding: "binding".to_owned(),
        proposal_identity: "proposal-a".to_owned(),
        coverage_identity: "coverage-a".to_owned(),
        program_revision: "revision-a".to_owned(),
    };
    let mut revised_proposal = original.clone();
    (*revised_proposal.proposal_identity_mut_for_test()) = "proposal-b".to_owned();
    assert_ne!(original, revised_proposal);
    let mut revised_coverage = original.clone();
    (*revised_coverage.coverage_identity_mut_for_test()) = "coverage-b".to_owned();
    assert_ne!(original, revised_coverage);
}
