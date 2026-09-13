pub(super) fn projection_row_count(
    projection: &worth_ui_host_contract::UiMountedProjectionView,
) -> u64 {
    let rows = [
        projection.nodes().len(),
        projection.clips().rows().len(),
        projection.layers().rows().len(),
        projection.portal_overlays().rows().len(),
        projection.semantic_text().rows().len(),
        projection.hit_tests().rows().len(),
        projection.paint_batches().rows().len(),
        projection.spatial_batches().rows().len(),
        projection.realtime_batches().rows().len(),
        projection.resources().entries().len(),
    ]
    .into_iter()
    .try_fold(0usize, usize::checked_add)
    .and_then(|rows| u64::try_from(rows).ok());
    rows.expect("an admitted projection row count fits u64")
}
