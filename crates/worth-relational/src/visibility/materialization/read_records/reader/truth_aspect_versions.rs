//! Project the declared aspects of one visible record version.
use super::{context::VisibilityReadContext, kind_scan_visibility::KindScanVisibility};
use worth_foundational::facade::AspectKey;

impl<'runtime> VisibilityReadContext<'runtime> {
    pub fn entity_aspects_at_version(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        entity_id: crate::identity::data::EntityId,
        version_id: crate::identity::data::VersionId,
    ) -> Result<
        Option<Vec<AspectKey>>,
        crate::execution::denial_cause::RelationalExecutionDenialCause,
    > {
        let edition = self.runtime.storage_access().current_edition();
        let state = edition
            .demand_selected(
                execution,
                &self.runtime.services.symbols,
                [entity_id.partition_id],
            )
            .map_err(crate::execution::denial_cause::RelationalExecutionDenialCause::from)?;
        crate::visibility::materialization::read_records::read_work::metered_with_memory(
            execution,
            |context, memory| {
                context
                    .checkpoint(super::super::read_prices::KEYED_RECORD)
                    .map_err(crate::execution::denial::kernel_stop)?;
                let clone_bytes = super::super::read_work::record_quote::entity_for_id(
                    &state,
                    &self.runtime.config.schema.registry,
                    entity_id,
                    KindScanVisibility::for_selected_version(
                        self.runtime.current_version_id(),
                        version_id,
                    ),
                )?;
                super::super::read_work::record_memory::claim(memory, clone_bytes)?;
                let Some(record) = self
                    .authoritative_entity_record_for_id_at_version(&state, entity_id, version_id)
                else {
                    super::super::read_work::record_memory::release(memory, clone_bytes)?;
                    return Ok(None);
                };
                let kind = record.kind.kind_id;
                drop(record);
                super::super::read_work::record_memory::release(memory, clone_bytes)?;
                Ok(Some(
                    super::aspect_catalog::declared_aspects_for_entity_kind(
                        &self.runtime.schema_contract_runtime.aspect_contract_plans,
                        kind,
                        &mut |units| context.checkpoint(units),
                        memory,
                    )?,
                ))
            },
            |answer: &Result<
                Option<Vec<AspectKey>>,
                crate::execution::denial_cause::RelationalExecutionDenialCause,
            >| {
                answer
                    .as_ref()
                    .ok()
                    .and_then(Option::as_ref)
                    .map_or(0, |aspects| {
                        aspects.iter().fold(
                            aspects.capacity() as u64 * std::mem::size_of::<AspectKey>() as u64,
                            |bytes, key| {
                                bytes.saturating_add(key.owned_allocation_capacity_bytes() as u64)
                            },
                        )
                    })
            },
        )?
    }

    pub fn relation_aspects_at_version(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        relation_id: crate::identity::data::RelationId,
        version_id: crate::identity::data::VersionId,
    ) -> Result<
        Option<Vec<AspectKey>>,
        crate::execution::denial_cause::RelationalExecutionDenialCause,
    > {
        let edition = self.runtime.storage_access().current_edition();
        let state = edition
            .demand_selected(
                execution,
                &self.runtime.services.symbols,
                [relation_id.partition_id],
            )
            .map_err(crate::execution::denial_cause::RelationalExecutionDenialCause::from)?;
        crate::visibility::materialization::read_records::read_work::metered_with_memory(
            execution,
            |context, memory| {
                context
                    .checkpoint(super::super::read_prices::KEYED_RECORD)
                    .map_err(crate::execution::denial::kernel_stop)?;
                let clone_bytes = super::super::read_work::record_quote::relation_for_id(
                    &state,
                    &self.runtime.config.schema.registry,
                    relation_id,
                    KindScanVisibility::for_selected_version(
                        self.runtime.current_version_id(),
                        version_id,
                    ),
                )?;
                super::super::read_work::record_memory::claim(memory, clone_bytes)?;
                let Some(record) = self.authoritative_relation_record_for_id_at_version(
                    &state,
                    relation_id,
                    version_id,
                ) else {
                    super::super::read_work::record_memory::release(memory, clone_bytes)?;
                    return Ok(None);
                };
                let kind = record.kind.kind_id;
                drop(record);
                super::super::read_work::record_memory::release(memory, clone_bytes)?;
                Ok(Some(
                    super::aspect_catalog::declared_aspects_for_relation_kind(
                        &self.runtime.schema_contract_runtime.aspect_contract_plans,
                        kind,
                        &mut |units| context.checkpoint(units),
                        memory,
                    )?,
                ))
            },
            |answer: &Result<
                Option<Vec<AspectKey>>,
                crate::execution::denial_cause::RelationalExecutionDenialCause,
            >| {
                answer
                    .as_ref()
                    .ok()
                    .and_then(Option::as_ref)
                    .map_or(0, |aspects| {
                        aspects.iter().fold(
                            aspects.capacity() as u64 * std::mem::size_of::<AspectKey>() as u64,
                            |bytes, key| {
                                bytes.saturating_add(key.owned_allocation_capacity_bytes() as u64)
                            },
                        )
                    })
            },
        )?
    }
}
