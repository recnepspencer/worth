//! Allocation capacity of complete non-live read results returned by the map.
use super::{RawNonLiveKernelOutcome, RawOneShotRows};
use worth_execution::ChargedBytes;
impl ChargedBytes for RawNonLiveKernelOutcome {
    fn additional_charged_bytes(&self) -> u64 {
        let Self { raw, result_buffer } = self;
        let RawOneShotRows {
            rows,
            source_footprints,
            result_set_source,
            examined_candidates: _examined_candidates,
            predicate_work_units: _predicate_work_units,
            predicate_index_generation: _predicate_index_generation,
            target_identity_index_generation: _target_identity_index_generation,
            target_identity_index_entries_examined: _target_identity_index_entries_examined,
            projected_records: _projected_records,
            projected_fields: _projected_fields,
            adjacency_lists_read: _adjacency_lists_read,
            relation_records_examined: _relation_records_examined,
            ordering_comparisons: _ordering_comparisons,
            ordered_index_generation: _ordered_index_generation,
            ordered_index_entries_examined: _ordered_index_entries_examined,
            next_boundary,
            has_more: _has_more,
            actual_work: _actual_work,
        } = raw;
        // The closed pair read always selects Complete, which cannot mint a
        // native continuation. Refuse unsupported output rather than undercount
        // the inaccessible native continuation payload if this door changes.
        if next_boundary.is_some() {
            return u64::MAX;
        }
        let sources = source_footprints.iter().fold(
            (source_footprints.capacity() as u64).saturating_mul(std::mem::size_of::<
                super::super::observed_source::WorthQueryObservedSourceFootprint,
            >() as u64),
            |bytes, source| {
                bytes
                    .saturating_add(source.retained_bytes() as u64)
                    .saturating_add(
                        source
                            .root_selection
                            .as_ref()
                            .map_or(0, |selection| selection.retained_bytes() as u64),
                    )
            },
        );
        rows.additional_charged_bytes()
            .saturating_add(sources)
            .saturating_add(
                result_set_source
                    .as_ref()
                    .map_or(0, |source| source.retained_bytes() as u64),
            )
            .saturating_add(result_buffer.additional_charged_bytes())
    }
}
