//! Reserve scalar columns and attach decode failures to their exact region.
use super::*;
use crate::identity::data::PartitionId;
use crate::storage::data::RelationalStorageUnavailableCause as Cause;

pub fn at_region(cause: Failure, partition: PartitionId) -> Failure {
    match cause {
        Failure::Request(cause) => Failure::Request(cause),
        Failure::Content(cause) => Failure::Content(cause),
        Failure::StorageUnavailable(mut cause) => {
            cause.subject = RelationalStorageSubject::Region(partition);
            cause.into()
        }
    }
}

pub fn scalars<T: crate::storage::substrate::StoredValue>(
    raw: Raw<'_>,
    request: &worth_execution::ExecutionRequest<'_, '_>,
) -> Result<Quote> {
    let count = sequence(raw, request, |_| Ok(()))?;
    Ok(Quote::bytes(
        vector::<T::Value>(count)?,
        column::<T>(count)?,
    ))
}

pub fn content(
    cause: crate::storage::overlay::PartitionContentDigestError,
    partition: crate::identity::data::PartitionId,
) -> Failure {
    use crate::storage::overlay::PartitionContentDigestError as Error;
    match cause {
        Error::Request(cause) | Error::Execution(cause) => Failure::Request(cause),
        Error::AllocationAccountingOverflow => overflow(),
        Error::UnresolvedContentSymbol(symbol) => RelationalStorageUnavailable {
            subject: RelationalStorageSubject::Symbols {
                first: symbol,
                last: symbol,
            },
            cause: Cause::SourceUnavailable,
        }
        .into(),
        Error::InvalidHistoryCoverage | Error::Encoding => at_region(damaged(), partition),
    }
}
