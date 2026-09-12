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

/// Consumes the owner's exact invocation set, including unrequested extra rules.
pub(super) fn validate_receipt_closure(
    evidence: &RelationalMutationInvariantEvidence,
    requirements: &[WorthQueryInstalledInvariantExecutionRequirement],
    semantic_work: u64,
) -> Result<u64, WorthQueryInvariantExecutionFailure> {
    let receipts = evidence.custom_invariant_execution_receipts();
    let expected_count = requirements
        .iter()
        .filter(|requirement| requirement.application_invariant().is_some())
        .count();
    if receipts.len() != expected_count
        || receipts
            .windows(2)
            .any(|pair| receipt_key(&pair[0]) >= receipt_key(&pair[1]))
    {
        return Err(super::closure_failure(
            "Relational custom invariant receipts do not match the installed invocation set",
        ));
    }
    requirements
        .iter()
        .try_fold(semantic_work, |total, requirement| {
            match requirement.application_invariant() {
                Some(descriptor) => total
                    .checked_add(matched_receipt(evidence, descriptor)?.work_units().get())
                    .ok_or_else(super::owner_failure),
                None => Ok(total),
            }
        })
}

pub(super) fn requirement_work(
    evidence: &RelationalMutationInvariantEvidence,
    requirement: &WorthQueryInstalledInvariantExecutionRequirement,
    semantic_work: usize,
) -> Result<u64, WorthQueryInvariantExecutionFailure> {
    match requirement.application_invariant() {
        Some(descriptor) => Ok(matched_receipt(evidence, descriptor)?.work_units().get()),
        None => u64::try_from(semantic_work).map_err(|_| super::owner_failure()),
    }
}

fn matched_receipt<'evidence>(
    evidence: &'evidence RelationalMutationInvariantEvidence,
    descriptor: &WorthQueryInstalledApplicationInvariantDescriptor,
) -> Result<&'evidence CustomInvariantExecutionReceipt, WorthQueryInvariantExecutionFailure> {
    let receipts = evidence.custom_invariant_execution_receipts();
    let key = (
        descriptor.identifier(),
        descriptor.major(),
        descriptor.minor(),
        execution_point(descriptor.execution_point()),
    );
    let index = receipts
        .binary_search_by(|receipt| receipt_key(receipt).cmp(&key))
        .map_err(|_| super::closure_failure("required custom invariant receipt is absent"))?;
    let receipt = &receipts[index];
    if receipt.provenance().proposal_identity.as_ref() != Some(evidence.proposal_identity())
        || receipt.provenance().version_id != evidence.proposed_version()
        || receipt.verdict() != InvariantDecisionKind::Passed
        || receipt.work_units().get() > descriptor.maximum_work_units().get()
    {
        return Err(super::closure_failure(
            "custom invariant receipt does not prove this candidate within its installed contract",
        ));
    }
    Ok(receipt)
}

fn receipt_key(
    receipt: &CustomInvariantExecutionReceipt,
) -> (&str, u16, u16, InvariantExecutionPoint) {
    let version = receipt.semantic_version();
    (
        receipt.rule_id().as_str(),
        version.major,
        version.minor,
        receipt.execution_point(),
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
