//! Exact relation candidates preserve the existing materialization refusal.
use super::*;
use crate::visibility::materialization::read_records::projection::exact_basis_reads::ExactProjectionReader;

impl ProjectionRecordReader<'_> {
    pub(in crate::visibility::materialization::read_records) fn relations_from_directory(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        source: &ExactProjectionReader<'_>,
        kind: crate::identity::data::KindId,
        partition: Option<crate::identity::data::PartitionId>,
        maximum: Option<usize>,
        charge: &mut dyn FnMut(u64) -> Result<(), worth_execution::MapKernelStop>,
        memory: &mut worth_execution::ExecutionResultMemory<'_>,
    ) -> Result<BoundedRelationKindTruthRead, NativeScanStop<RelationKindTruthReadDenial>> {
        let mut scan =
            RelationKindScan::with_limit(maximum, maximum.is_some(), charge, Some(memory));
        let registry = source.registry();
        for id in source.relation_ids(kind, partition) {
            scan.examine_slot()?;
            let head = source
                .demand_head(execution, id.partition_id)
                .map_err(NativeScanStop::Request)?;
            if head
                .relation_arena()
                .get(&id)
                .is_none_or(|record| record.kind_id() != Some(kind))
            {
                return Err(NativeScanStop::Request(
                    source.membership_failure(id.partition_id),
                ));
            }
            scan.reserve_with(
                id.partition_id,
                id.slot_index(),
                super::super::super::read_work::record_quote::relation_head(
                    registry,
                    head,
                    id.slot_index(),
                )
                .map_err(NativeScanStop::Request)?,
                || {
                    self.instrumentation.count(|counts| {
                        counts.visible_authoritative_relation_records_materialized += 1
                    });
                    head.relation_record(registry, id)
                },
            )?;
        }
        Ok(scan.finish())
    }
}
