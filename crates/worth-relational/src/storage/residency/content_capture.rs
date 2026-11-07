//! Capture resident canonical content without reconstructing a cold history.
use super::RelationalRegionResidency;
use crate::storage::data::RelationalStorageUnavailable;
use crate::storage::overlay::{PartitionContentCommitment, PartitionContentDigestError};
use crate::symbols::data::StringInterner;
use std::sync::Arc;

#[derive(Debug)]
pub(crate) enum RegionContentFailure {
    Execution(crate::execution::RelationalExecutionDenialCause),
    Content(PartitionContentDigestError),
    Adjacency,
    Storage(RelationalStorageUnavailable),
}

impl RelationalRegionResidency {
    pub(crate) fn capture_resident_content(
        self: &Arc<Self>,
        lane: &crate::storage::substrate::maintenance::ReconstructionLane<'_>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        symbols: &StringInterner,
    ) -> Result<Option<[u8; 32]>, RegionContentFailure> {
        let Some(lease) = self
            .lease_checkpoint_contents()
            .map_err(RegionContentFailure::Storage)?
        else {
            return Ok(None);
        };
        let partition = lease.checkpoint_contents();
        let symbols = self.content_symbols(symbols);
        let (commitment, _) =
            PartitionContentCommitment::capture_layered(lane, execution, partition, &symbols)
                .map_err(RegionContentFailure::Content)?;
        Ok(Some(commitment.digest(partition.partition_id)))
    }
}

impl From<super::RelationalResidencyDemandFailure> for RegionContentFailure {
    fn from(failure: super::RelationalResidencyDemandFailure) -> Self {
        match failure {
            super::RelationalResidencyDemandFailure::Request(cause) => Self::Execution(cause),
            super::RelationalResidencyDemandFailure::StorageUnavailable(cause) => {
                Self::Storage(cause)
            }
            super::RelationalResidencyDemandFailure::Content(cause) => {
                Self::Execution(cause.into_execution_cause())
            }
        }
    }
}

/// Preserve original-request refusals through checkpoint and cold verification.
pub(crate) fn restore_failure(
    partition: crate::identity::data::PartitionId,
    error: PartitionContentDigestError,
) -> super::RelationalResidencyDemandFailure {
    restore_failure_with_cause(
        partition,
        error,
        crate::storage::data::RelationalStorageUnavailableCause::Damaged,
    )
}
pub(super) fn restore_failure_with_cause(
    partition: crate::identity::data::PartitionId,
    error: PartitionContentDigestError,
    corruption: crate::storage::data::RelationalStorageUnavailableCause,
) -> super::RelationalResidencyDemandFailure {
    match error {
        PartitionContentDigestError::Execution(cause) => {
            super::RelationalResidencyDemandFailure::Request(cause)
        }
        other => super::RelationalResidencyDemandFailure::StorageUnavailable(
            RelationalStorageUnavailable {
                subject: match other {
                    PartitionContentDigestError::UnresolvedContentSymbol(symbol) => {
                        crate::storage::data::RelationalStorageSubject::Symbols {
                            first: symbol,
                            last: symbol,
                        }
                    }
                    _ => crate::storage::data::RelationalStorageSubject::Region(partition),
                },
                cause: match other {
                    PartitionContentDigestError::UnresolvedContentSymbol(_) => {
                        crate::storage::data::RelationalStorageUnavailableCause::SourceUnavailable
                    }
                    _ => corruption,
                },
            },
        ),
    }
}
