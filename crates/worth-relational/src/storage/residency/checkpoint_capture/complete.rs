//! Complete captures only original Full DATA; ACK retains displaced SDK nodes.
use super::*;
use crate::durability::data::RelationalRecoveryStop;
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
    pub(crate) fn capture_complete_for_request(
        self: &Arc<Self>,
        request: &ExecutionRequest<'_, '_>,
        schema: &RelationalBranchRootSchemaAuthority,
        schema_digest: [u8; 32],
        symbols: &dyn crate::symbols::data::SymbolTextLookup,
    ) -> Result<CapturedCompleteRegion, RelationalRecoveryStop> {
        request
            .consult()
            .map_err(|cause| RelationalRecoveryStop::Request(cause.into()))?;
        let lease = self
            .lease_resident_full()?
            .ok_or_else(|| self.capture_failure(Cause::SourceUnavailable))?;
        let region = lease.resident_region();
        let captured = super::super::region_capture::complete::capture(
            &region.partition,
            region
                .commitment
                .as_ref()
                .ok_or_else(|| self.capture_failure(Cause::SourceUnavailable))?,
            schema,
            schema_digest,
            symbols,
            request,
        )?;
        let partition = &region.partition;
        let leaf = RelationalCheckpointRegion {
            kind_authority: captured.kind_authority,
            partition_id: partition.partition_id,
            content_digest: region
                .commitment
                .as_ref()
                .unwrap()
                .digest(partition.partition_id),
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
