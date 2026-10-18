use super::*;
pub(super) fn origin_request(
    cause: impl Into<crate::execution::RelationalExecutionDenialCause>,
) -> TransactionCommitError {
    TransactionCommitError::execution(crate::transactions::data::CommitExecutionDenial {
        kind: crate::transactions::data::CommitExecutionDenialKind::Cause(cause.into()),
        partition_identity: None,
    })
}

pub(super) fn origin_retention_bytes(
    envelope: &CanonicalCommitEnvelope,
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
                    .checked_mul(std::mem::size_of::<crate::history::data::CommitId>())?,
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
            origin_request(crate::execution::RelationalExecutionDenialCause::ChargedBytesOverflow)
        })?;
    Ok(bytes)
}
