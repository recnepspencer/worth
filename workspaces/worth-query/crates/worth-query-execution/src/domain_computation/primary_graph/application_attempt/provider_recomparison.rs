use super::precondition_binding::WorthQueryBoundMutationPreconditions;

pub use worth_query_execution_lineage::facade::primary_graph::application_attempt::provider_recomparison::WorthQueryMutationPreconditionComparisonEvidence;

pub(super) fn certify_provider_recomparison(
    preconditions: &WorthQueryBoundMutationPreconditions,
) -> WorthQueryMutationPreconditionComparisonEvidence {
    WorthQueryMutationPreconditionComparisonEvidence::from_parts(
        preconditions.expected_version_count(),
        preconditions.expected_fact_count(),
        preconditions.identity().copied(),
    )
}
