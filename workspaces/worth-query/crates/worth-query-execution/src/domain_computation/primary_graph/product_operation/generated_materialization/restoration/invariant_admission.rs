use worth_query_declaration::facade::application_schema::ApplicationInvariantExecutionPoint;
use worth_query_installation::facade::ApplicationSchema;
use worth_relational::facade::mvcc::CustomInvariantNonApplicability;
use worth_relational::facade::runtime::{InvariantDecisionKind, InvariantExecutionPoint};

use crate::domain_computation::primary_graph::vocabulary::product_operation::invariant_admission::admit_required_invariants;
use crate::domain_computation::primary_graph::WorthQueryApplicationProducerBinding;
use crate::domain_computation::vocabulary::primary_graph::invariant_denial::WorthQueryGeneratedOutputInvariantAdmissionDenial as Denial;

pub(super) fn admit<Schema, Producer>(
    evidence: &worth_relational::facade::mvcc::RelationalMutationInvariantEvidence,
    expected_branch: &worth_relational::facade::history::BranchId,
) -> Result<(), Denial>
where
    Schema: ApplicationSchema,
    Producer: WorthQueryApplicationProducerBinding<Schema>,
{
    if evidence.branch() != expected_branch {
        return Err(Denial::ForeignBranchEvidence);
    }
    let summary = evidence.summary();
    if summary.execution_count != 3
        || !summary.commit_boundary_seen
        || !summary.mutation_sensitive_seen
        || !summary.snapshot_publication_seen
    {
        return Err(Denial::IncompleteOwnerEvidence);
    }
    admit_required_invariants(
        evidence.custom_invariant_execution_receipts(),
        Producer::REQUIRED_INVARIANTS,
    )
}

#[cfg(test)]
mod tests;
