//! Select the installed read operands before handing off to the read implementation.
use super::*;
use crate::visibility::materialization::read_records::reader::context::VisibilityReadContext;

impl VisibilityReadContext<'_> {
    pub(crate) fn paid_entities_in_partition(
        &self,
        state: &(impl PartitionAccess + ?Sized),
        registry: &crate::schema::data::RelationalSchemaRegistry,
        partition: crate::identity::data::PartitionId,
        kind: Option<crate::identity::data::KindId>,
        visibility: KindScanVisibility,
        charge: &mut dyn FnMut(u64) -> Result<(), worth_execution::MapKernelStop>,
        memory: &mut worth_execution::ExecutionResultMemory<'_>,
    ) -> Result<Vec<EntityReadRecord>, crate::execution::denial_cause::RelationalExecutionDenialCause>
    {
        ProjectionRecordReader::new(
            self.runtime.current_version_id(),
            &self.runtime.services.instrumentation,
        )
        .paid_entities_in_partition(state, registry, partition, kind, visibility, charge, memory)
    }
    pub(crate) fn paid_relations_in_partition(
        &self,
        state: &(impl PartitionAccess + ?Sized),
        registry: &crate::schema::data::RelationalSchemaRegistry,
        partition: crate::identity::data::PartitionId,
        kind: Option<crate::identity::data::KindId>,
        visibility: KindScanVisibility,
        charge: &mut dyn FnMut(u64) -> Result<(), worth_execution::MapKernelStop>,
        memory: &mut worth_execution::ExecutionResultMemory<'_>,
    ) -> Result<
        Vec<RelationReadRecord>,
        crate::execution::denial_cause::RelationalExecutionDenialCause,
    > {
        ProjectionRecordReader::new(
            self.runtime.current_version_id(),
            &self.runtime.services.instrumentation,
        )
        .paid_relations_in_partition(state, registry, partition, kind, visibility, charge, memory)
    }
}
