use super::*;

pub(super) fn root_matches(
    source: ReleasedInventoryView<'_>,
    result: ReleasedInventoryView<'_>,
    projection: &PersistedPhysicalRecoveryProjection,
) -> bool {
    let state = projection.root_state();
    let dropped = projection
        .derived_retirement()
        .map(|value| value.dropped_records())
        .unwrap_or_default();
    let removed = |record: PersistedRecordIdentity| dropped.binary_search(&record).is_ok();
    let mut latest = source
        .root
        .latest_blob_publication()
        .filter(|value| !removed(value.record()));
    let mut directory = source.root.derived_family_directory().filter(|value| {
        !removed(value.directory_record())
            && value
                .indexed_through_blob_publication()
                .is_none_or(|publication| !removed(publication.record()))
    });
    let mut quarantine = source
        .root
        .latest_blob_quarantine()
        .filter(|value| !removed(*value));
    match projection.blob_semantic() {
        Semantic::GenerationPublished(binding) => {
            latest = IndexedThroughBlobPublication::new(
                result.root.generation(),
                binding.record(),
                binding.record_payload_sha256(),
            );
        }
        Semantic::DerivedDirectory(binding) => {
            if projection.derived_retirement().is_some_and(|retirement| {
                retirement.expected_previous() != source.root.derived_family_directory()
            }) || binding.indexed_through() != latest
                || binding
                    .indexed_through_quarantine()
                    .is_some_and(|expected| expected != quarantine)
            {
                return false;
            }
            directory = Some(DerivedFamilyRootDirectoryBinding::new(
                binding.record().record(),
                binding.indexed_through(),
            ));
        }
        Semantic::DedupeQuarantined(binding) => quarantine = Some(binding.record()),
        _ => {}
    }
    let prepared_wins = match (
        state.last_inline_segment(),
        source.root.last_inline_segment(),
    ) {
        (Some(prepared), Some(current)) => {
            (prepared.segment_id().get(), prepared.generation().get())
                >= (current.segment_id().get(), current.generation().get())
        }
        (Some(_), None) => true,
        _ => false,
    };
    let (last_record, last_segment) = if prepared_wins {
        (state.last_inline_record(), state.last_inline_segment())
    } else {
        (
            source.root.last_inline_record(),
            source.root.last_inline_segment(),
        )
    };
    result.root.tree_identity() == source.root.tree_identity()
        && result.root.release_custody_head_root()
            == source.root.release_custody_head_root()
        && result.root.next_release_custody_head_block()
            == source.root.next_release_custody_head_block()
        && result.root.requires_maintenance_protocol()
            == source.root.requires_maintenance_protocol()
        && result.root.node_capacity() == state.successor_manifest_capacity()
        && result.root.latest_blob_publication() == latest
        && result.root.derived_family_directory() == directory
        && result.root.latest_blob_quarantine() == quarantine
        && result.root.tier_epoch_anchor() == source.root.tier_epoch_anchor()
        && result.root.last_inline_record() == last_record
        && result.root.last_inline_segment() == last_segment
        && result.root.next_block() >= source.root.next_block()
        && result.root.next_segment_block() >= source.root.next_segment_block()
        && result.free.next_block() >= source.free.next_block()
}
