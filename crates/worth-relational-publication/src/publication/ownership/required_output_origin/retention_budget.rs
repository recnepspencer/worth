//! Retained memory quote for a selected route and its canonical membership.
use super::*;
use worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasis;
pub(super) fn origin_retention_bytes(
    envelope: &CanonicalCommitEnvelopeBody,
    basis: &AdmittedRelationalBranchBasis,
    producer_bytes: usize,
    targets: usize,
    relations: usize,
) -> Result<u64, TransactionCommitError> {
    let parent_bytes = basis
        .reference()
        .target()
        .as_basis()
        .map_or(0, |target| target.parent_commit_ids().len())
        .checked_mul(std::mem::size_of::<u64>());
    let bytes = std::mem::size_of::<CanonicalRequiredOutputOrigin>()
        .checked_add(2 * std::mem::size_of::<usize>())
        .and_then(|bytes| bytes.checked_add(producer_bytes))
        .and_then(|bytes| bytes.checked_add(basis.descriptor().branch_id().0.len()))
        .and_then(|bytes| bytes.checked_add(basis.reference().branch_id().as_str().len()))
        .and_then(|bytes| bytes.checked_add(envelope.commit.branch_id.0.len()))
        .and_then(|bytes| {
            bytes.checked_add(
                envelope
                    .commit
                    .parents
                    .len()
                    .checked_mul(std::mem::size_of::<worth_relational_contracts::internal_port::history::data::coordinates::CommitId>())?,
            )
        })
        .and_then(|bytes| bytes.checked_add(parent_bytes?))
        .and_then(|bytes| bytes.checked_add(targets.checked_mul(std::mem::size_of::<EntityId>())?))
        .and_then(|bytes| {
            bytes.checked_add(
                relations.checked_mul(std::mem::size_of::<CanonicalRequiredOutputRelation>())?,
            )
        })
        .and_then(|bytes| u64::try_from(bytes).ok())
        .ok_or_else(|| {
            origin_request(worth_relational_contracts::internal_port::execution::denial_cause::RelationalExecutionDenialCause::ChargedBytesOverflow)
        })?;
    Ok(bytes)
}
