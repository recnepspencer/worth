//! Service-free construction from exact Full row data and its original request.
use super::*;
use crate::storage::residency::kind_projection::full::FullKindProjection;
use worth_execution::ExecutionRequest;

impl PreparedResidentRegion {
    pub(crate) fn initial_for_request(
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
            .map_err(|cause| RegionContentFailure::Request(cause.into()))?;
        let kind_commitment = {
            let history = FullKindProjection::normalize(&mut partition, request)?;
            let head = FullKindProjection::capture(
                &partition,
                crate::storage::partition::AdjacencyKindBasis::Current,
                request,
            )?;
            RelationalRegionKindCommitment {
                head: crate::storage::residency::kind_commitment::digest_full(&head, request)?,
                history: crate::storage::residency::kind_commitment::digest_full(
                    &history, request,
                )?,
            }
        };
        partition.clear_runtime_pin_counters();
        let (commitment, hashed) =
            PartitionContentCommitment::capture_full_for_request(&partition, symbols, request)
                .map_err(|cause| match cause {
                    crate::storage::overlay::PartitionContentDigestError::Request(cause) => {
                        RegionContentFailure::Request(cause)
                    }
                    other => RegionContentFailure::Content(other),
                })?;
        let inventory = partition.allocation_inventory();
        let unavailable_records = partition.entity_arena.lifecycle_counts().unavailable
            + partition.relation_arena.lifecycle_counts().unavailable;
        let entity_slots = partition.entity_arena.slot_count();
        let relation_slots = partition.relation_arena.slot_count();
        let partition_id = partition.partition_id;
        let content_digest = commitment.digest(partition_id);
        let lease =
            RelationalRegionResidency::resident_for_request(partition, commitment, request)?;
        Ok(Self {
            partition_id,
            retention: RelationalRegionRetention::from_lease(lease),
            inventory,
            content_digest,
            kind_commitment,
            entity_slots,
            relation_slots,
            content_values_hashed: hashed,
            unavailable_records,
            copied_truth_bytes: 0,
            new_payload_bytes: inventory.authoritative_bytes,
        })
    }
}

#[cfg(test)]
mod tests;
