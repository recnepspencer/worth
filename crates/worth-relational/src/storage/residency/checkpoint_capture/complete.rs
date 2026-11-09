//! Complete captures only original Full DATA; ACK retains displaced SDK nodes.
use super::*;
use crate::durability::data::RelationalRecoveryStop;
use crate::storage::overlay::PartitionContentDigestError;
use worth_execution::{ExecutionOrderedMap, ExecutionRequest};

#[derive(Debug)]
pub(crate) struct CapturedCompleteRegion {
    pub(crate) leaf: RelationalCheckpointRegion,
    pub(crate) members: [RelationalCheckpointMember; 2],
    pub(crate) publication: RelationalCheckpointRegionPublication,
}
#[derive(Debug)]
pub(crate) struct RetiredRegionCheckpointBindings {
    _original: ExecutionOrderedMap<[u8; 32], RecordedRegionCheckpoint>,
}
#[derive(Debug)]
pub(crate) enum RegionCheckpointAcknowledgementDenial {
    Busy,
    Stopped(RelationalRecoveryStop),
}
impl RelationalCheckpointRegionPublication {
    /// The caller retains the actual ComponentCaptureCustody CellArc roster
    /// through every call and retry. Therefore this temporary upgrade is never
    /// the last Cell DATA owner. An idle publication still owns only Weak.
    pub(crate) fn try_acknowledge_for_request(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Option<RetiredRegionCheckpointBindings>, RegionCheckpointAcknowledgementDenial>
    {
        use RegionCheckpointAcknowledgementDenial as Denial;
        request
            .consult()
            .map_err(|cause| Denial::Stopped(RelationalRecoveryStop::Request(cause.into())))?;
        let Some(cell) = self.cell.upgrade() else {
            return Ok(None);
        };
        let mut bindings = match cell.checkpoint_bindings.try_lock() {
            Ok(guard) => guard,
            Err(std::sync::TryLockError::Poisoned(error)) => error.into_inner(),
            Err(std::sync::TryLockError::WouldBlock) => return Err(Denial::Busy),
        };
        let mut candidate = bindings.clone();
        candidate
            .insert_cloned_native_for_request(&self.schema_digest, &self.recorded, *request)
            .map_err(|cause| {
                Denial::Stopped(crate::durability::checkpoint_history::capture::map_stop(
                    cause,
                ))
            })?;
        let original = std::mem::replace(&mut *bindings, candidate);
        drop(bindings);
        Ok(Some(RetiredRegionCheckpointBindings {
            _original: original,
        }))
    }
}
impl RelationalRegionResidency {
    pub(super) fn capture_full_contents_for_request(
        self: &Arc<Self>,
        lane: &crate::storage::substrate::maintenance::ReconstructionLane<'_>,
        lease: &super::super::RelationalResidencyLease<super::super::Full>,
        request: &ExecutionRequest<'_, '_>,
        schema: &RelationalBranchRootSchemaAuthority,
        schema_digest: [u8; 32],
        symbols: &dyn crate::symbols::data::SymbolTextLookup,
    ) -> Result<CapturedCompleteRegion, RelationalRecoveryStop> {
        let region = lease.resident_region();
        let computed;
        let commitment = match region.commitment.as_ref() {
            Some(commitment) => commitment,
            None => {
                computed =
                    PartitionContentCommitment::capture(lane, request, &region.partition, symbols)
                        .map_err(|cause| match cause {
                            PartitionContentDigestError::Request(cause)
                            | PartitionContentDigestError::Execution(cause) => {
                                RelationalRecoveryStop::Request(cause)
                            }
                            PartitionContentDigestError::UnresolvedContentSymbol(symbol) => {
                                RelationalStorageUnavailable {
                                    subject: RelationalStorageSubject::Symbols {
                                        first: symbol,
                                        last: symbol,
                                    },
                                    cause: Cause::SourceUnavailable,
                                }
                                .into()
                            }
                            PartitionContentDigestError::InvalidHistoryCoverage
                            | PartitionContentDigestError::Encoding
                            | PartitionContentDigestError::AllocationAccountingOverflow => {
                                self.capture_failure(Cause::IntegrityMismatch).into()
                            }
                        })?
                        .0;
                &computed
            }
        };
        let captured = super::super::region_capture::complete::capture(
            lane,
            &region.partition,
            commitment,
            schema,
            schema_digest,
            symbols,
            request,
        )?;
        let partition = &region.partition;
        let leaf = RelationalCheckpointRegion {
            kind_authority: captured.kind_authority,
            partition_id: partition.partition_id,
            content_digest: commitment.digest(partition.partition_id),
            directory: captured.directory,
            live_counts: super::super::LiveRecordCounts::from_partition(partition),
            entity_slots: partition.entity_arena.slot_count(),
            relation_slots: partition.relation_arena.slot_count(),
            unavailable_records: partition.entity_arena.lifecycle_counts().unavailable
                + partition.relation_arena.lifecycle_counts().unavailable,
            head: captured.head.0,
            history: captured.history.0,
        };
        let recorded = RecordedRegionCheckpoint {
            leaf: leaf.clone(),
            history_members: captured.history_members,
        };
        Ok(CapturedCompleteRegion {
            leaf,
            members: [
                RelationalCheckpointMember::New {
                    locator: captured.head.0,
                    bytes: captured.head.1,
                },
                RelationalCheckpointMember::New {
                    locator: captured.history.0,
                    bytes: captured.history.1,
                },
            ],
            publication: RelationalCheckpointRegionPublication {
                cell: Self::downgrade(self),
                schema_digest,
                recorded,
            },
        })
    }
}
#[cfg(test)]
mod tests;
