//! Select the installed read operands before handing off to the read implementation.
use super::*;
use crate::visibility::materialization::read_records::reader::context::VisibilityReadContext;

impl<'runtime> VisibilityReadContext<'runtime> {
    /// Scans one relation kind at a version, refusing before spending more
    /// than the declared slot-examination and record-materialization work.
    pub fn bounded_visible_relations_of_kind(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        kind_id: crate::identity::data::KindId,
        version_id: crate::identity::data::VersionId,
        maximum_work_units: usize,
    ) -> Result<
        Result<BoundedRelationKindTruthRead, RelationKindTruthReadDenial>,
        crate::execution::denial_cause::RelationalExecutionDenialCause,
    > {
        let edition = self.runtime.storage_access().current_edition();
        let state = edition
            .demand_all(execution, &self.runtime.services.symbols)
            .map_err(crate::execution::denial_cause::RelationalExecutionDenialCause::from)?;
        crate::visibility::materialization::read_records::read_work::scan_with_memory(
            execution,
            |context, memory| {
                ProjectionRecordReader::new(
                    self.runtime.current_version_id(),
                    &self.runtime.services.instrumentation,
                )
                .bounded_relations_of_kind_in_state(
                    &state,
                    &self.runtime.config.schema.registry,
                    kind_id,
                    KindScanVisibility::for_selected_version(
                        self.runtime.current_version_id(),
                        version_id,
                    ),
                    maximum_work_units,
                    &mut |units| context.checkpoint(units),
                    memory,
                )
            },
            |result| super::super::super::read_footprint::relations(&result.records),
        )
    }
}
