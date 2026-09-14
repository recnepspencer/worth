use worth_query_declaration::facade::application_schema::ApplicationInvariantExecutionPoint;
use worth_query_installation::facade::ApplicationSchema;
use worth_relational::facade::runtime::{InvariantDecisionKind, InvariantExecutionPoint};

use super::WorthQueryGeneratedOutputInvariantAdmissionDenial as Denial;
use crate::domain_computation::primary_graph::WorthQueryApplicationProducerBinding;

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
    for requirement in Producer::REQUIRED_INVARIANTS {
        admit_requirement(evidence.custom_invariant_execution_receipts(), *requirement)?;
    }
    Ok(())
}

fn admit_requirement(
    receipts: &[worth_relational::facade::mvcc::CustomInvariantExecutionReceipt],
    requirement: crate::domain_computation::primary_graph::WorthQueryProducerInvariantRequirement,
) -> Result<(), Denial> {
    let execution_point = lower_execution_point(requirement.execution_point());
    let matching_identity = receipts
        .iter()
        .filter(|receipt| {
            receipt.rule_id().as_str() == requirement.identifier()
                && receipt.execution_point() == execution_point
        })
        .collect::<Vec<_>>();
    if matching_identity.is_empty() {
        return Err(Denial::MissingRequiredInvariant);
    }
    let matching_version = matching_identity
        .into_iter()
        .filter(|receipt| {
            let version = receipt.semantic_version();
            version.major == requirement.major() && version.minor == requirement.minor()
        })
        .collect::<Vec<_>>();
    if matching_version.is_empty() {
        return Err(Denial::RequiredInvariantVersionMismatch);
    }
    if matching_version.len() != 1 {
        return Err(Denial::DuplicateRequiredInvariant);
    }
    if matching_version[0].verdict() != InvariantDecisionKind::Passed {
        return Err(Denial::RequiredInvariantDidNotPass);
    }
    Ok(())
}

const fn lower_execution_point(
    point: ApplicationInvariantExecutionPoint,
) -> InvariantExecutionPoint {
    match point {
        ApplicationInvariantExecutionPoint::CommitBoundary => {
            InvariantExecutionPoint::CommitBoundary
        }
        ApplicationInvariantExecutionPoint::MutationSensitive => {
            InvariantExecutionPoint::MutationSensitive
        }
        ApplicationInvariantExecutionPoint::SnapshotPublication => {
            InvariantExecutionPoint::SnapshotPublication
        }
    }
}
