//! The bounded exact scan visits only owner-maintained typed candidates.
use super::*;
use crate::visibility::materialization::read_records::projection::exact_basis_reads::ExactProjectionReader;

impl ProjectionRecordReader<'_> {
    pub(in crate::visibility::materialization::read_records) fn entities_from_directory(
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
        for id in source.entity_ids(kind, partition) {
            scan.examine_slot()?;
            let head = source
                .demand_head(execution, id.partition_id)
                .map_err(NativeScanStop::Request)?;
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
