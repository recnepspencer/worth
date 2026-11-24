//! Preserve request stops and damage evidence across ownership reads.
use crate::history::data::coordinates::CommitId;
use crate::publication::data::authored_retirement_refusal::AuthoredRetirementRefusal;
use crate::publication::data::publication_error::PublicationError;
use crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use crate::transactions::data::primitives::RecordRef;

pub(in crate::publication::ownership) fn corrupt(
    publication: CommitId,
    record: RecordRef,
) -> TransactionCommitError {
    TransactionCommitError::publication(PublicationError::authored_retirement(
        AuthoredRetirementRefusal::CorruptCurrentOwnership { publication },
        record,
    ))
}

pub(crate) fn checkpoint(
    context: &mut worth_execution::MapKernelContext<'_, '_>,
    work: u64,
) -> Result<(), TransactionCommitError> {
    crate::execution::request_stage::checkpoint(context, work).map_err(request)
}

pub(crate) fn request(
    cause: impl Into<crate::execution::denial_cause::RelationalExecutionDenialCause>,
) -> TransactionCommitError {
    TransactionCommitError::execution(crate::transactions::data::outcomes::transaction_commit_error::CommitExecutionDenial {
        kind: crate::transactions::data::outcomes::transaction_commit_error::CommitExecutionDenialKind::Cause(cause.into()),
        partition_identity: None,
    })
}

pub(in crate::publication::ownership) fn damaged(record: &RecordRef) -> TransactionCommitError {
    use crate::storage::data::storage_unavailable::{
        RelationalStorageSubject, RelationalStorageUnavailable, RelationalStorageUnavailableCause,
    };
    let partition = match record {
        RecordRef::Entity(id) => id.partition_id,
        RecordRef::Relation(id) => id.partition_id,
    };
    request(
        crate::execution::denial_cause::RelationalExecutionDenialCause::StorageUnavailable(
            RelationalStorageUnavailable {
                subject: RelationalStorageSubject::Region(partition),
                cause: RelationalStorageUnavailableCause::Damaged,
            },
        ),
    )
}
