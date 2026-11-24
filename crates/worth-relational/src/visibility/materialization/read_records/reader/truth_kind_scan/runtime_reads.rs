//! Select the installed read operands before handing off to the read implementation.
use worth_relational_branch::internal_port::visibility::materialization::read_records::reader::truth_kind_scan::BoundedEntityKindTruthReadFieldsPort as _;
use super::*;
use crate::visibility::materialization::read_records::reader::context::VisibilityReadContext;

impl<'runtime> VisibilityReadContext<'runtime> {
    /// Reads the live entities of one kind across every partition, spending at
    /// most `maximum_work_units` units of work.
    ///
    /// One unit is charged for each occupied entity slot examined — including a
    /// slot skipped for belonging to another kind, which the scan still had to
    /// look at — and one further unit for each record materialized into the
    /// answer. The bound is tested before each unit is spent, so a maximum of
    /// zero refuses without examining anything.
    pub fn bounded_visible_entities_of_kind(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        kind_id: crate::identity::data::KindId,
        version_id: crate::identity::data::VersionId,
        maximum_work_units: usize,
    ) -> Result<
        Result<BoundedEntityKindTruthRead, EntityKindTruthReadLimitExceeded>,
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
                .bounded_entities_of_kind_in_state(
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
            |result| super::super::super::read_footprint::entities(&(*result.owner_records())),
        )
    }
}
