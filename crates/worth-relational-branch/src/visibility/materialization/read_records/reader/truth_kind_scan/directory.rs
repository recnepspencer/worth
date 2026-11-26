//! The bounded exact scan visits only owner-maintained typed candidates.
use super::*;
use crate::visibility::materialization::read_records::projection::exact_basis_reads::ExactProjectionReader;

impl ProjectionRecordReader<'_> {
    pub(in crate::internal_port::visibility::materialization::read_records) fn entities_from_directory(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        source: &ExactProjectionReader<'_>,
        kind: crate::identity::data::KindId,
        partition: Option<crate::identity::data::PartitionId>,
        maximum: Option<usize>,
        charge: &mut dyn FnMut(u64) -> Result<(), worth_execution::MapKernelStop>,
        memory: &mut worth_execution::ExecutionResultMemory<'_>,
    ) -> Result<BoundedEntityKindTruthRead, NativeScanStop<EntityKindTruthReadLimitExceeded>> {
        let mut scan = EntityKindScan::with_limit(maximum, charge, Some(memory));
        let registry = source.registry();
        // Rolling one-region window (Store M4 D7): the previous borrow drops
        // before the next region is demanded, so the scan never pins the root.
        let mut window = None;
        for id in source.entity_ids(kind, partition) {
            let id = id.map_err(|failure| {
                NativeScanStop::Request(
                    crate::execution::denial_cause::RelationalExecutionDenialCause::StorageUnavailable(failure),
                )
            })?;
            scan.examine_slot()?;
            if window.as_ref().map(|(region, _)| *region) != Some(id.partition_id) {
                // Release the previous borrow before demanding the next region.
                drop(window.take());
                let lease = source
                    .borrow_head(execution, id.partition_id)
                    .map_err(NativeScanStop::Request)?;
                window = Some((id.partition_id, lease));
            }
            let head = window.as_ref().map(|(_, lease)| lease.head_view()).expect(
                /* Proof: The window was filled for this exact partition immediately above. */
                "window holds the current region",
            );
            if head
                .entity_arena()
                .get(&id)
                .is_none_or(|record| record.kind_id() != Some(kind))
            {
                return Err(NativeScanStop::Request(
                    source.membership_failure(id.partition_id),
                ));
            }
            scan.reserve_with(super::super::super::read_work::record_quote::entity_head(registry, head, id.slot_index()).map_err(NativeScanStop::Request)?, || {
                self.instrumentation.count(|counts| counts.visible_authoritative_entity_records_materialized += 1);
                head.entity_record(registry, id).expect(/* Proof: The exact directory ID was admitted against this same Head's kind and generation before materialization; valid live entities retain their mandatory schema metadata. */ "admitted directory entity materializes")
            })?;
        }
        Ok(scan.finish())
    }
}
