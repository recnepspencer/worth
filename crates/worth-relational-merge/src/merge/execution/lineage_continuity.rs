pub(super) fn derive_deleted_on_both_sides_lineage_continuity(
    lowered_record: &crate::merge::data::LoweredMergePlanRecord,
    source_record: &crate::merge::data::VisibleMergeRecord,
) -> crate::merge::data::MergeLineageContinuityVerdict {
    let source_lineage_id = source_record.source_lineage_id.or(source_record.lineage_id);
    let target_lineage_id = source_record.target_lineage_id.or(source_record.lineage_id);

    if lowered_record.target_record.is_none() {
        return crate::merge::data::MergeLineageContinuityVerdict::Unchanged;
    }

    match (source_lineage_id, target_lineage_id) {
        (Some(source_lineage_id), Some(target_lineage_id))
            if source_lineage_id == target_lineage_id =>
        {
            crate::merge::data::MergeLineageContinuityVerdict::Unchanged
        }
        _ => crate::merge::data::MergeLineageContinuityVerdict::Preserved,
    }
}
