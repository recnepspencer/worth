use worth_store_physical_backend::QualifiedFilesystemMedia;

/// Buffered file placement follows format and qualified filesystem geometry.
/// Direct I/O requires its separately qualified offset and transfer alignment;
/// this function does not grant buffer-memory or direct-I/O capability.
pub(in crate::physical_runtime::record_serving) fn qualified_arena_alignment(
    media: &QualifiedFilesystemMedia,
) -> Option<u64> {
    let minimum =
        u64::from(worth_store_physical_format::PhysicalAlignmentClass::extent_start_4k().bytes());
    let mut alignment = minimum.max(media.profile().allocation_granularity().get());
    if let Some(direct) = media.capabilities().direct_io() {
        alignment = alignment
            .max(direct.offset_granularity().get())
            .max(direct.transfer_granularity().get());
    }
    alignment.is_power_of_two().then_some(alignment)
}
