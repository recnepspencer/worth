//! Original-request driver for the shared kind validation and backing owners.
use super::super::super::RelationalResidencyDemandFailure as Failure;
use super::*;
use crate::execution::RelationalExecutionDenialCause;
use worth_execution::ExecutionRequest;

pub(in crate::storage::residency) fn rebuild_for_request(
    lane: &ReconstructionLane<'_>,
    partition: &mut PartitionState,
    projection: &[RegionRelationProjection],
    basis: AdjacencyKindBasis,
    request: &ExecutionRequest<'_, '_>,
) -> Result<(), Failure> {
    let id = partition.partition_id;
    drive(request, id, &mut |funding, checkpoint| {
        super::rebuild_using(
            lane,
            partition,
            projection,
            basis,
            Some(funding),
            checkpoint,
            &mut |_, _| Ok(None),
        )
    })
}
pub(in crate::storage::residency) fn rebuild_head_history_for_request(
    lane: &ReconstructionLane<'_>,
    partition: &mut PartitionState,
    projection: &[RegionRelationProjection],
    request: &ExecutionRequest<'_, '_>,
) -> Result<(), Failure> {
    let id = partition.partition_id;
    drive(request, id, &mut |funding, checkpoint| {
        super::rebuild_head_using(lane, partition, projection, Some(funding), checkpoint)
    })
}

pub(super) struct RequestProjection<'request, 'scope, 'host> {
    pub(super) execution: &'request ExecutionRequest<'scope, 'host>,
    stopped: std::cell::Cell<Option<RelationalExecutionDenialCause>>,
}
impl RequestProjection<'_, '_, '_> {
    pub(super) fn stop(&self, cause: RelationalExecutionDenialCause) -> Cause {
        if self.stopped.get().is_none() {
            self.stopped.set(Some(cause));
        }
        Cause::SourceUnavailable
    }
}

fn drive(
    request: &ExecutionRequest<'_, '_>,
    partition: crate::identity::data::PartitionId,
    operation: &mut dyn FnMut(
        &RequestProjection<'_, '_, '_>,
        &mut ProjectionCheckpoint<'_>,
    ) -> Result<(), Cause>,
) -> Result<(), Failure> {
    let funding = RequestProjection {
        execution: request,
        stopped: std::cell::Cell::new(None),
    };
    let result = operation(&funding, &mut |units, _| {
        if funding.stopped.get().is_some() {
            return Err(Cause::SourceUnavailable);
        }
        match crate::storage::residency::region_load::materialization_funding::work(request, units)
        {
            Ok(()) => Ok(()),
            Err(Failure::Request(cause)) => Err(funding.stop(cause)),
            Err(Failure::Content(_) | Failure::StorageUnavailable(_)) => {
                unreachable!("work admission has only request outcomes")
            }
        }
    });
    if let Some(cause) = funding.stopped.get() {
        return Err(Failure::Request(cause));
    }
    result.map_err(|cause| {
        crate::storage::data::RelationalStorageUnavailable {
            subject: crate::storage::data::RelationalStorageSubject::Region(partition),
            cause,
        }
        .into()
    })
}
