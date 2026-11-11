//! Service-free construction from exact Full row data and its original request.
use super::{
    PreparedResidentRegion, RegionContentFailure, RelationalRegionKindCommitment,
    RelationalRegionResidency, RelationalRegionRetention,
};
use crate::storage::overlay::{PartitionContentCommitment, PartitionState};
use crate::storage::residency::kind_projection::full::FullKindProjection;
use worth_execution::ExecutionRequest;

impl PreparedResidentRegion {
    pub(crate) fn initial(
        lane: &crate::storage::substrate::maintenance::ReconstructionLane<'_>,
        request: &ExecutionRequest<'_, '_>,
        mut partition: PartitionState,
        symbols: &dyn crate::symbols::data::SymbolTextLookup,
    ) -> Result<Self, RegionContentFailure> {
        // Never promote a Head frontier, retained sibling hash or source-backed
        // prefix into reconstructed Full truth.
        if partition
            .entity_arena
            .history_prefixes
            .iter()
            .chain(partition.relation_arena.history_prefixes.iter())
            .any(Option::is_some)
        {
            return Err(RegionContentFailure::Content(
                crate::storage::overlay::PartitionContentDigestError::InvalidHistoryCoverage,
            ));
        }
        request
            .consult()
            .map_err(|cause| RegionContentFailure::Execution(cause.into()))?;
        let kind_commitment = {
            let history = FullKindProjection::normalize(lane, &mut partition, request)?;
            let head = FullKindProjection::capture(
                lane,
                &partition,
                crate::storage::partition::AdjacencyKindBasis::Current,
                request,
            )?;
            RelationalRegionKindCommitment {
                head: crate::storage::residency::kind_commitment::digest_full(
                    lane, &head, request,
                )?,
                history: crate::storage::residency::kind_commitment::digest_full(
                    lane, &history, request,
                )?,
            }
        };
        partition.clear_runtime_pin_counters();
        let (commitment, hashed) = PartitionContentCommitment::capture(
            lane, request, &partition, symbols,
        )
        .map_err(|cause| match cause {
            crate::storage::overlay::PartitionContentDigestError::Request(cause)
            | crate::storage::overlay::PartitionContentDigestError::Execution(cause) => {
                RegionContentFailure::Execution(cause)
            }
            other => RegionContentFailure::Content(other),
        })?;
        let kind_indexes = super::kind_admission::reconstruct(lane, request, &partition)
            .map_err(RegionContentFailure::from)?;
        if kind_indexes.commitment() != kind_commitment {
            return Err(RegionContentFailure::Adjacency);
        }
        let requirements =
            super::required_symbols::CapturedSymbolRequirements::capture_for_request(
                lane, &partition, true, request,
            )
            .map_err(RegionContentFailure::from)?;
        let directory = lane
            .recover(request, |lane, work| {
                super::RegionRecordDirectory::capture(lane, &partition, &mut |units, bytes| {
                    Ok(work.prepare(units, bytes)?)
                })
            })
            .map_err(|error| error.into_content_failure(partition.partition_id))?;
        let live_counts = super::LiveRecordCounts::from_partition(&partition);
        let inventory = partition.allocation_inventory();
        let unavailable_records = partition.entity_arena.lifecycle_counts().unavailable
            + partition.relation_arena.lifecycle_counts().unavailable;
        let entity_slots = partition.entity_arena.slot_count();
        let relation_slots = partition.relation_arena.slot_count();
        let partition_id = partition.partition_id;
        let content_digest = commitment.digest(partition_id);
        let lease = RelationalRegionResidency::resident_for_request(
            partition,
            commitment,
            kind_indexes,
            Some(requirements),
            request,
        )?;
        Ok(Self {
            partition_id,
            retention: RelationalRegionRetention::from_lease(lease),
            inventory,
            content_digest,
            kind_commitment,
            directory,
            live_counts,
            entity_slots,
            relation_slots,
            content_values_hashed: hashed,
            unavailable_records,
            copied_truth_bytes: 0,
            new_payload_bytes: inventory.authoritative_bytes,
        })
    }
}
