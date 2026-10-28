use worth_query_declaration::facade::application_schema::ApplicationInvariantExecutionPoint;
use worth_query_installation::facade::ApplicationSchema;
use worth_relational::facade::mvcc::CustomInvariantNonApplicability;
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
    admit_required_invariants(
        evidence.custom_invariant_execution_receipts(),
        Producer::REQUIRED_INVARIANTS,
    )
}

pub(in crate::domain_computation::primary_graph) fn admit_required_invariants(
    receipts: &[worth_relational::facade::mvcc::CustomInvariantExecutionReceipt],
    requirements: &[crate::domain_computation::primary_graph::WorthQueryProducerInvariantRequirement],
) -> Result<(), Denial> {
    for requirement in requirements {
        admit_requirement(receipts, *requirement)?;
    }
    Ok(())
}

fn admit_requirement(
    receipts: &[worth_relational::facade::mvcc::CustomInvariantExecutionReceipt],
    requirement: crate::domain_computation::primary_graph::WorthQueryProducerInvariantRequirement,
) -> Result<RequirementSatisfaction, Denial> {
    admit_requirement_facts(
        receipts.iter().map(|receipt| {
            let version = receipt.semantic_version();
            ReceiptFact {
                identifier: receipt.rule_id().as_str(),
                major: version.major,
                minor: version.minor,
                execution_point: receipt.execution_point(),
                verdict: receipt.verdict(),
                non_applicability: receipt.non_applicability(),
            }
        }),
        requirement,
    )
}

#[derive(Clone, Copy)]
struct ReceiptFact<'a> {
    identifier: &'a str,
    major: u16,
    minor: u16,
    execution_point: InvariantExecutionPoint,
    verdict: InvariantDecisionKind,
    non_applicability: Option<CustomInvariantNonApplicability>,
}

/// How a producer-owned invariant requirement was satisfied.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RequirementSatisfaction {
    /// The rule evaluated the candidate and affirmed it.
    Affirmed,
    /// Relational proved the candidate touches nothing in the rule's declared
    /// closure, so the rule holds over the publication vacuously.
    Vacuous,
}

fn admit_requirement_facts<'a>(
    receipts: impl IntoIterator<Item = ReceiptFact<'a>>,
    requirement: crate::domain_computation::primary_graph::WorthQueryProducerInvariantRequirement,
) -> Result<RequirementSatisfaction, Denial> {
    let execution_point = lower_execution_point(requirement.execution_point());
    let matching_identity = receipts
        .into_iter()
        .filter(|receipt| {
            receipt.identifier == requirement.identifier()
                && receipt.execution_point == execution_point
        })
        .collect::<Vec<_>>();
    if matching_identity.is_empty() {
        return Err(Denial::MissingRequiredInvariant);
    }
    let matching_version = matching_identity
        .into_iter()
        .filter(|receipt| {
            receipt.major == requirement.major() && receipt.minor == requirement.minor()
        })
        .collect::<Vec<_>>();
    if matching_version.is_empty() {
        return Err(Denial::RequiredInvariantVersionMismatch);
    }
    if matching_version.len() != 1 {
        return Err(Denial::DuplicateRequiredInvariant);
    }
    // A producer's explicit required invariant must affirm the output, or the
    // output must provably touch nothing the rule governs. Scoped or refused
    // non-applicability is valid for unrelated application rules, but does not
    // satisfy this producer-owned requirement.
    let receipt = matching_version[0];
    match (receipt.verdict, receipt.non_applicability) {
        (InvariantDecisionKind::Passed, None) => Ok(RequirementSatisfaction::Affirmed),
        (
            InvariantDecisionKind::NotApplicable,
            Some(CustomInvariantNonApplicability::CandidateOutsideClosure),
        ) => Ok(RequirementSatisfaction::Vacuous),
        (verdict, non_applicability) => Err(Denial::RequiredInvariantDidNotPass {
            verdict,
            non_applicability,
        }),
    }
}

#[cfg(test)]
mod tests;

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
