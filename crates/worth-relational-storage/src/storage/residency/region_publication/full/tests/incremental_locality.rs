use super::*;
pub(super) fn publish_one_slot_and_assert(
    execution: &super::FullTestExecution<'_, '_>,
    initial: &PreparedResidentRegion,
    initial_region: &crate::storage::residency::ResidentRegion,
    extra: EntityExtra,
    version: VersionId,
    symbols: &RuntimeSymbolTable,
    expect_zero_scans: bool,
    label: &str,
) {
    let mut overlay = initial_region.partition.clone();
    overlay
        .entity_arena
        .apply_extra_update(0, extra, version)
        .unwrap();
    let mut journal = PartitionMutationJournal::default();
    journal.entity_slots.insert(0);
    let scans_before = crate::storage::substrate::extra_delta_resolve_scans_on_current_thread();
    let updated = PreparedResidentRegion::incremental(
        execution,
        RegionCaptureBasis {
            retention: &initial.retention,
            scope: &crate::storage::residency::OperationResidency::new(),
            directory: &initial.directory,
            inventory: initial.inventory,
            unavailable_records: initial.unavailable_records,
            kind_commitment: initial.kind_commitment,
        },
        &overlay,
        &journal,
        || panic!("a Full predecessor needs no new reconstruction schema"),
        symbols,
        None,
    )
    .unwrap();
    let scans_after = crate::storage::substrate::extra_delta_resolve_scans_on_current_thread();
    assert_eq!(
        overlay.entity_arena.slot_count(),
        initial_region.partition.entity_arena.slot_count()
    );
    if expect_zero_scans {
        assert_eq!(
            scans_after - scans_before,
            0,
            "{label}: must never scan prior partition"
        );
    }
    let actual_lease = updated
        .retention
        .pin_lease(&crate::storage::residency::OperationResidency::new())
        .unwrap();
    let actual = &actual_lease.resident_region().partition;
    let exp_new = locality_helpers::assert_directed_ids(
        &initial_region.partition,
        actual,
        initial_region.sealed_watermarks,
    );
    assert_eq!(updated.new_payload_bytes, exp_new);
    assert_eq!(updated.inventory, actual.allocation_inventory().unwrap());
    drop(overlay);
}
