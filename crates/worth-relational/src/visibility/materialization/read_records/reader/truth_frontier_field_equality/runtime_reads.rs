//! Select the installed read operands before handing off to the read implementation.
use worth_relational_branch::internal_port::visibility::materialization::read_records::reader::truth_frontier_field_equality::assemble_bounded_frontier_field_equality_truth_read;
use super::*;
use crate::visibility::materialization::read_records::reader::context::VisibilityReadContext;

impl<'runtime> VisibilityReadContext<'runtime> {
    pub fn bounded_entity_field_equals_for_frontier_at_version(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        entity_ids: &BTreeSet<crate::identity::data::EntityId>,
        kind_id: crate::identity::data::KindId,
        field_locator: &AspectFieldLocator,
        expected: &AspectValue,
        version_id: crate::identity::data::VersionId,
        maximum_work_units: usize,
    ) -> Result<
        Result<BoundedFrontierFieldEqualityTruthRead, FrontierFieldEqualityTruthReadLimitExceeded>,
        crate::execution::denial_cause::RelationalExecutionDenialCause,
    > {
        let edition = self.runtime.storage_access().current_edition();
        let partitions = entity_ids
            .iter()
            .map(|id| id.partition_id)
            .collect::<BTreeSet<_>>();
        let state = edition
            .demand_selected(execution, &self.runtime.services.symbols, partitions)
            .map_err(crate::execution::denial_cause::RelationalExecutionDenialCause::from)?;
        crate::visibility::materialization::read_records::read_request::checked(execution, || {
            let expected = AuthoritativeFieldComparisonKey::from_aspect_value(expected);
            let mut matching_entity_ids = BTreeSet::new();
            let mut entity_records_examined = 0_usize;
            let mut field_reads = 0_usize;
            for entity_id in entity_ids {
                charge_frontier_field_work(
                    Some(maximum_work_units),
                    entity_records_examined,
                    matching_entity_ids.len(),
                )?;
                entity_records_examined += 1;
                let Some(record) = self
                    .authoritative_entity_record_for_id_at_version(&state, *entity_id, version_id)
                else {
                    continue;
                };
                if record.kind.kind_id != kind_id
                    || record.lifecycle != crate::storage::data::RecordLifecycleState::Live
                {
                    continue;
                }
                field_reads += 1;
                if entity_query_locus_comparison_key(&record, field_locator).as_ref()
                    != Some(&expected)
                {
                    continue;
                }
                charge_frontier_field_work(
                    Some(maximum_work_units),
                    entity_records_examined,
                    matching_entity_ids.len(),
                )?;
                matching_entity_ids.insert(record.entity_id);
            }
            Ok(assemble_bounded_frontier_field_equality_truth_read(
                matching_entity_ids,
                entity_records_examined,
                field_reads,
            ))
        })
    }
}
