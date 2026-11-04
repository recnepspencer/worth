use worth_relational::facade::identity::EntityId;
use worth_relational::facade::identity::PartitionId;

use worth_query_execution::publication_boundary::workflow_advance::RequiredWorkflowAssessment;

#[test]
fn exact_requirement_distinguishes_proposal_and_coverage_with_same_execution_affinity() {
    let original = RequiredWorkflowAssessment::new_for_test(EntityId::new(PartitionId::main(), 1, 1), "review".to_owned(), "transition".to_owned(), 1, "query".to_owned(), "parameters".to_owned(), "result".to_owned(), "binding".to_owned(), "proposal-a".to_owned(), "coverage-a".to_owned(), "revision-a".to_owned());
    let mut revised_proposal = original.clone();
    (*revised_proposal.proposal_identity_mut_for_test()) = "proposal-b".to_owned();
    assert_ne!(original, revised_proposal);
    let mut revised_coverage = original.clone();
    (*revised_coverage.coverage_identity_mut_for_test()) = "coverage-b".to_owned();
    assert_ne!(original, revised_coverage);
}
