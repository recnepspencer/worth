use worth_query_declaration::facade::application_schema::ApplicationInvariantExecutionPoint;
use worth_query_installation::facade::{
    WorthQueryInstalledApplicationInvariantDescriptor,
    WorthQueryInstalledInvariantExecutionRequirement,
};
use worth_relational::facade::{
    mvcc::{CustomInvariantExecutionReceipt, RelationalMutationInvariantEvidence},
    runtime::{InvariantDecisionKind, InvariantExecutionPoint},
};

use crate::domain_computation::WorthQueryInvariantExecutionFailure;

#[cfg(test)]
mod tests;

type ReceiptKey<'a> = (&'a str, u16, u16, InvariantExecutionPoint);

#[derive(Clone, Copy)]
struct ReceiptFact<'a> {
    key: ReceiptKey<'a>,
    decision: InvariantDecisionKind,
    bound: bool,
    work: u64,
}

#[derive(Clone, Copy)]
struct RequirementFact<'a> {
    key: ReceiptKey<'a>,
    maximum_work: u64,
}

/// Consumes the owner's exact invocation set, including unrequested extra rules.
pub(super) fn validate_receipt_closure(
    evidence: &RelationalMutationInvariantEvidence,
    requirements: &[WorthQueryInstalledInvariantExecutionRequirement],
    semantic_work: u64,
) -> Result<u64, WorthQueryInvariantExecutionFailure> {
    let receipts = evidence
        .custom_invariant_execution_receipts()
        .iter()
        .map(|receipt| receipt_fact(receipt, evidence))
        .collect::<Vec<_>>();
    let requirements = requirements
        .iter()
        .filter_map(|requirement| requirement.application_invariant())
        .map(requirement_fact)
        .collect::<Vec<_>>();
    close(&receipts, &requirements, semantic_work)
}

fn close(
    receipts: &[ReceiptFact<'_>],
    requirements: &[RequirementFact<'_>],
    semantic_work: u64,
) -> Result<u64, WorthQueryInvariantExecutionFailure> {
    if receipts.windows(2).any(|pair| pair[0].key >= pair[1].key)
        || receipts.iter().any(|receipt| {
            !receipt.bound
                || (receipt.decision != InvariantDecisionKind::NotApplicable
                    && !requirements
                        .iter()
                        .any(|required| required.key == receipt.key))
        })
    {
        return Err(super::closure_failure(
            "Relational custom invariant receipts do not match the installed invocation set",
        ));
    }
    requirements
        .iter()
        .try_fold(semantic_work, |total, required| {
            let index = receipts
                .binary_search_by(|receipt| receipt.key.cmp(&required.key))
                .map_err(|_| {
                    super::closure_failure("required custom invariant receipt is absent")
                })?;
            let work = validate_required_receipt(receipts[index], *required)?;
            total.checked_add(work).ok_or_else(super::owner_failure)
        })
}

pub(super) fn requirement_work(
    evidence: &RelationalMutationInvariantEvidence,
    requirement: &WorthQueryInstalledInvariantExecutionRequirement,
    semantic_work: usize,
) -> Result<u64, WorthQueryInvariantExecutionFailure> {
    match requirement.application_invariant() {
        Some(descriptor) => matched_receipt_work(evidence, descriptor),
        None => u64::try_from(semantic_work).map_err(|_| super::owner_failure()),
    }
}

fn matched_receipt_work(
    evidence: &RelationalMutationInvariantEvidence,
    descriptor: &WorthQueryInstalledApplicationInvariantDescriptor,
) -> Result<u64, WorthQueryInvariantExecutionFailure> {
    let receipts = evidence.custom_invariant_execution_receipts();
    let key = requirement_key(descriptor);
    let index = receipts
        .binary_search_by(|receipt| receipt_key(receipt).cmp(&key))
        .map_err(|_| super::closure_failure("required custom invariant receipt is absent"))?;
    validate_required_receipt(
        receipt_fact(&receipts[index], evidence),
        requirement_fact(descriptor),
    )
}

fn validate_required_receipt(
    receipt: ReceiptFact<'_>,
    required: RequirementFact<'_>,
) -> Result<u64, WorthQueryInvariantExecutionFailure> {
    if !receipt.bound
        || !matches!(
            receipt.decision,
            InvariantDecisionKind::Passed | InvariantDecisionKind::NotApplicable
        )
        || receipt.work > required.maximum_work
    {
        return Err(super::closure_failure(
            "custom invariant receipt does not prove this candidate within its installed contract",
        ));
    }
    Ok(receipt.work)
}

fn receipt_fact<'a>(
    receipt: &'a CustomInvariantExecutionReceipt,
    evidence: &RelationalMutationInvariantEvidence,
) -> ReceiptFact<'a> {
    ReceiptFact {
        key: receipt_key(receipt),
        decision: receipt.verdict(),
        bound: receipt.provenance().proposal_identity.as_ref()
            == Some(evidence.proposal_identity())
            && receipt.provenance().version_id == evidence.proposed_version(),
        work: receipt.work_units().get(),
    }
}

fn requirement_fact(
    descriptor: &WorthQueryInstalledApplicationInvariantDescriptor,
) -> RequirementFact<'_> {
    RequirementFact {
        key: requirement_key(descriptor),
        maximum_work: descriptor.maximum_work_units().get(),
    }
}

fn receipt_key(receipt: &CustomInvariantExecutionReceipt) -> ReceiptKey<'_> {
    let version = receipt.semantic_version();
    (
        receipt.rule_id().as_str(),
        version.major,
        version.minor,
        receipt.execution_point(),
    )
}

fn requirement_key(
    descriptor: &WorthQueryInstalledApplicationInvariantDescriptor,
) -> ReceiptKey<'_> {
    (
        descriptor.identifier(),
        descriptor.major(),
        descriptor.minor(),
        execution_point(descriptor.execution_point()),
    )
}

fn execution_point(point: ApplicationInvariantExecutionPoint) -> InvariantExecutionPoint {
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
