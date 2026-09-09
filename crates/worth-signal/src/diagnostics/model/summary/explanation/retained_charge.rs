use super::ExplanationSummary;
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Work, RetainedStoragePreparationDenial as Denial,
};
impl RetainedStorageMeasurement for ExplanationSummary {
    fn retained_heap_charge(&self, work: &mut Work) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            profile: _,
            node: _,
            materialization_mode: _,
            state: _,
            dirty_aspect_count: _,
            upstream_count: _,
            changed_upstream_count: _,
            skipped_upstream_count: _,
            condition_deferred_count: _,
            clean_upstream_count: _,
            missing_snapshot_count: _,
            dependency_removed_count: _,
            conservative_cause_count: _,
            direct_scope_count: _,
            translated_scope_count: _,
            discarded_scope_count: _,
            insufficient_scope_count: _,
            rewired_dependency_count: _,
            direct_cause_kinds,
            scope_provenance_kinds,
            cause_note_samples,
            triage_classes,
            propagation_suppressed: _,
            contract_reads_mask: _,
            contract_produces_mask: _,
            contract_partition_scope_count: _,
            required_context,
            execution_record_id: _,
            semantic_segment_id: _,
            output_change: _,
            memoized_origin: _,
            reuse_basis,
            reuse_origin: _,
            reuse_certification_proof_count: _,
            changed_region_count: _,
            causality_kind,
        } = self;
        direct_cause_kinds
            .retained_heap_charge(work)?
            .checked_add(scope_provenance_kinds.retained_heap_charge(work)?)?
            .checked_add(cause_note_samples.retained_heap_charge(work)?)?
            .checked_add(triage_classes.retained_heap_charge(work)?)?
            .checked_add(required_context.retained_heap_charge(work)?)?
            .checked_add(reuse_basis.retained_heap_charge(work)?)?
            .checked_add(causality_kind.retained_heap_charge(work)?)
    }
}
