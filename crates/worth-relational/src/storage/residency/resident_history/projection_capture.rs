//! Original-request work for the owned Head's separate projection capture.
use super::*;
use crate::storage::partition::RegionRelationProjection;

pub(super) fn capture(
    lane: &crate::storage::substrate::maintenance::ReconstructionLane<'_>,
    partition: &PartitionState,
    incidences: usize,
    request: &ExecutionRequest<'_, '_>,
) -> Result<Vec<RegionRelationProjection>, Failure> {
    // One ordered relation-map operation and endpoint-set insertion per
    // incidence. Incidences bound both map and set cardinality, without pricing
    // unused node slots. Include actual claim moves / Vec relocations.
    let levels = incidences
        .checked_add(1)
        .ok_or_else(funding::overflow)?
        .ilog2() as u64
        + 1;
    let navigation = funding::times(levels, 44)?;
    let copies = funding::sum(
        funding::times(size_of::<RegionRelationProjection>() as u64, 3)?,
        size_of::<usize>() as u64,
    )?;
    funding::work(
        request,
        funding::times(funding::sum(navigation, copies)?, incidences)?,
    )?;
    let mut stopped = None;
    let projection = super::super::kind_projection::capture_controlled(
        lane,
        partition,
        AdjacencyKindBasis::Historical,
        &mut |units, _backing| match funding::work(request, units) {
            Ok(()) => Ok(()),
            Err(Failure::Request(cause)) => {
                stopped = Some(cause);
                Err(Cause::SourceUnavailable)
            }
            Err(Failure::Content(_) | Failure::StorageUnavailable(_)) => {
                unreachable!("work admission has only request outcomes")
            }
        },
    );
    if let Some(cause) = stopped {
        return Err(Failure::Request(cause));
    }
    projection.map_err(|cause| {
        RelationalStorageUnavailable {
            subject: RelationalStorageSubject::Region(partition.partition_id),
            cause,
        }
        .into()
    })
}
