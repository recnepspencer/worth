//! Select the installed read operands before handing off to the read implementation.
use super::*;
use crate::visibility::materialization::read_records::reader::context::VisibilityReadContext;

impl<'runtime> VisibilityReadContext<'runtime> {
    pub fn visible_entity_at_version(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        entity_id: crate::identity::data::EntityId,
        version_id: crate::identity::data::VersionId,
    ) -> Result<
        Option<EntityReadRecord>,
        crate::execution::denial_cause::RelationalExecutionDenialCause,
    > {
        let edition = self.runtime.storage_access().current_edition();
        super::super::super::read_work::with_full_residency(
            execution,
            &edition,
            &self.runtime.services.symbols,
            |state| {
                crate::visibility::materialization::read_records::reader::projection_record_reader::ProjectionRecordReader::new(
                    self.runtime.current_version_id(),
                    &self.runtime.services.instrumentation,
                )
                .read_live_entity_at_version(
                    execution,
                    state,
                    &self.runtime.config.schema.registry,
                    entity_id,
                    version_id,
                )
            },
        )
    }

    pub fn visible_relation_at_version(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        relation_id: crate::identity::data::RelationId,
        version_id: crate::identity::data::VersionId,
    ) -> Result<
        Option<RelationReadRecord>,
        crate::execution::denial_cause::RelationalExecutionDenialCause,
    > {
        let edition = self.runtime.storage_access().current_edition();
        super::super::super::read_work::with_full_residency(
            execution,
            &edition,
            &self.runtime.services.symbols,
            |state| {
                crate::visibility::materialization::read_records::read_work::point_with_quote(
                    execution,
                    || {
                        super::super::super::read_work::record_quote::relation_for_id(
                            state,
                            &self.runtime.config.schema.registry,
                            relation_id,
                            KindScanVisibility::for_selected_version(
                                self.runtime.current_version_id(),
                                version_id,
                            ),
                        )
                    },
                    || {
                        self.authoritative_relation_record_for_id_at_version(
                            state,
                            relation_id,
                            version_id,
                        )
                        .filter(|record| record.lifecycle == RecordLifecycleState::Live)
                    },
                    |record| {
                        record
                            .as_ref()
                            .map_or(0, super::super::super::read_footprint::relation)
                    },
                )
            },
        )
    }

    pub fn visible_entities_of_kind(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        kind_id: crate::identity::data::KindId,
        version_id: crate::identity::data::VersionId,
    ) -> Result<Vec<EntityReadRecord>, crate::execution::denial_cause::RelationalExecutionDenialCause>
    {
        let edition = self.runtime.storage_access().current_edition();
        let state = edition
            .demand_all(execution, &self.runtime.services.symbols)
            .map_err(crate::execution::denial_cause::RelationalExecutionDenialCause::from)?;
        crate::visibility::materialization::read_records::read_work::metered_with_memory(
            execution,
            |context, memory| {
                let mut records = Vec::new();
                let (partitions, partition_bytes) =
                    super::super::super::read_work::record_memory::partition_ids(&state, memory)?;
                for partition_id in partitions {
                    let incoming = self.paid_entities_in_partition(
                        &state,
                        &self.runtime.config.schema.registry,
                        partition_id,
                        Some(kind_id),
                        KindScanVisibility::for_selected_version(
                            self.runtime.current_version_id(),
                            version_id,
                        ),
                        &mut |units| context.checkpoint(units),
                        memory,
                    )?;
                    super::super::super::read_work::record_memory::append(
                        memory,
                        &mut records,
                        incoming,
                    )?;
                }
                super::super::super::read_work::record_memory::release(memory, partition_bytes)?;
                Ok::<_, crate::execution::denial_cause::RelationalExecutionDenialCause>(records)
            },
            |result| {
                result
                    .as_ref()
                    .map_or(0, super::super::super::read_footprint::entities)
            },
        )?
    }

    pub fn visible_entities_of_kind_in_partition(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        partition_id: crate::identity::data::PartitionId,
        kind_id: crate::identity::data::KindId,
        version_id: crate::identity::data::VersionId,
    ) -> Result<Vec<EntityReadRecord>, crate::execution::denial_cause::RelationalExecutionDenialCause>
    {
        let edition = self.runtime.storage_access().current_edition();
        let state = edition
            .demand_selected(execution, &self.runtime.services.symbols, [partition_id])
            .map_err(crate::execution::denial_cause::RelationalExecutionDenialCause::from)?;
        crate::visibility::materialization::read_records::read_work::metered_with_memory(
            execution,
            |context, memory| {
                self.paid_entities_in_partition(
                    &state,
                    &self.runtime.config.schema.registry,
                    partition_id,
                    Some(kind_id),
                    KindScanVisibility::for_selected_version(
                        self.runtime.current_version_id(),
                        version_id,
                    ),
                    &mut |units| context.checkpoint(units),
                    memory,
                )
            },
            |result| {
                result
                    .as_ref()
                    .map_or(0, super::super::super::read_footprint::entities)
            },
        )?
    }

    pub fn visible_relations_of_kind(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        kind_id: crate::identity::data::KindId,
        version_id: crate::identity::data::VersionId,
    ) -> Result<
        Vec<RelationReadRecord>,
        crate::execution::denial_cause::RelationalExecutionDenialCause,
    > {
        let edition = self.runtime.storage_access().current_edition();
        let state = edition
            .demand_all(execution, &self.runtime.services.symbols)
            .map_err(crate::execution::denial_cause::RelationalExecutionDenialCause::from)?;
        crate::visibility::materialization::read_records::read_work::metered_with_memory(
            execution,
            |context, memory| {
                let mut records = Vec::new();
                let (partitions, partition_bytes) =
                    super::super::super::read_work::record_memory::partition_ids(&state, memory)?;
                for partition_id in partitions {
                    let incoming = self.paid_relations_in_partition(
                        &state,
                        &self.runtime.config.schema.registry,
                        partition_id,
                        Some(kind_id),
                        KindScanVisibility::for_selected_version(
                            self.runtime.current_version_id(),
                            version_id,
                        ),
                        &mut |units| context.checkpoint(units),
                        memory,
                    )?;
                    super::super::super::read_work::record_memory::append(
                        memory,
                        &mut records,
                        incoming,
                    )?;
                }
                super::super::super::read_work::record_memory::release(memory, partition_bytes)?;
                super::super::truth_record_access::sort_authoritative_relation_records(
                    &mut records,
                );
                Ok::<_, crate::execution::denial_cause::RelationalExecutionDenialCause>(records)
            },
            |result| {
                result
                    .as_ref()
                    .map_or(0, super::super::super::read_footprint::relations)
            },
        )?
    }

    pub fn visible_relations_of_kind_in_partition(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        partition_id: crate::identity::data::PartitionId,
        kind_id: crate::identity::data::KindId,
        version_id: crate::identity::data::VersionId,
    ) -> Result<
        Vec<RelationReadRecord>,
        crate::execution::denial_cause::RelationalExecutionDenialCause,
    > {
        let edition = self.runtime.storage_access().current_edition();
        let state = edition
            .demand_selected(execution, &self.runtime.services.symbols, [partition_id])
            .map_err(crate::execution::denial_cause::RelationalExecutionDenialCause::from)?;
        crate::visibility::materialization::read_records::read_work::metered_with_memory(
            execution,
            |context, memory| {
                let mut records = self.paid_relations_in_partition(
                    &state,
                    &self.runtime.config.schema.registry,
                    partition_id,
                    Some(kind_id),
                    KindScanVisibility::for_selected_version(
                        self.runtime.current_version_id(),
                        version_id,
                    ),
                    &mut |units| context.checkpoint(units),
                    memory,
                )?;
                super::super::truth_record_access::sort_authoritative_relation_records(
                    &mut records,
                );
                Ok::<_, crate::execution::denial_cause::RelationalExecutionDenialCause>(records)
            },
            |result| {
                result
                    .as_ref()
                    .map_or(0, super::super::super::read_footprint::relations)
            },
        )?
    }
}
