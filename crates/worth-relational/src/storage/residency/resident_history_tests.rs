//! Owned Head promotion retains old observations and independently rebuilds truth.
use super::*;
use crate::identity::data::VersionId;

#[test]
fn an_owned_head_promotes_without_a_checkpoint_and_keeps_its_old_observation() {
    let fixture = region_with_history();
    let request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let original = region_load::load_head(&fixture.cold, execution, 1 << 24).unwrap();
    let mut partial = original.partition.clone();
    let mut meanings = fixture.cold.symbols.begin_pending_meanings();
    let family = meanings.reserve("owned-head-unpublished-family").unwrap();
    let candidate_symbols = meanings.binding();
    assert_eq!(fixture.cold.symbols.resolve(family), None);
    let mut extra = partial
        .entity_arena
        .get_slot(fixture.entity.slot_index())
        .unwrap()
        .extra()
        .clone();
    extra.structural_fingerprint = Some(crate::identity::data::StructuralFingerprint::new(
        family, 731,
    ));
    partial.entity_arena.apply_extra_update(
        fixture.entity.slot_index(),
        extra.clone(),
        VersionId(1_000),
    );
    let symbols = candidate_symbols.interner_snapshot();
    assert!(matches!(
        PreparedResidentRegion::initial(partial.clone(), &symbols),
        Err(RegionContentFailure::Content(
            crate::storage::overlay::PartitionContentDigestError::InvalidHistoryCoverage
        ))
    ));
    let prior =
        RelationalRegionRetention::from_head_lease(head_demand(&fixture.cell, 1 << 24).unwrap());
    let mut journal = crate::storage::overlay::PartitionMutationJournal::default();
    journal.entity_slots.insert(fixture.entity.slot_index());
    let before = fixture.source.reads.load(Ordering::SeqCst);
    let prepared = PreparedResidentRegion::incremental(
        RegionCaptureBasis {
            retention: &prior,
            inventory: original.partition.allocation_inventory(),
            unavailable_records: original
                .partition
                .entity_arena
                .lifecycle_counts()
                .unavailable
                + original
                    .partition
                    .relation_arena
                    .lifecycle_counts()
                    .unavailable,
        },
        &partial,
        &journal,
        fixture.cold.schema.clone(),
        &candidate_symbols,
    )
    .unwrap();
    let head = prepared.retention;
    assert_eq!(fixture.source.reads.load(Ordering::SeqCst), before);
    assert!(head.cell().lease_resident_full().unwrap().is_none());
    let accepted = head.admitted_head_view().unwrap();
    let captured = head
        .cell()
        .capture_checkpoint(
            &fixture.cold.schema,
            fixture.cold.schema_carrier_digest,
            &symbols,
            None,
        )
        .unwrap();
    assert_eq!(fixture.source.reads.load(Ordering::SeqCst), before);
    let failure = full_demand(head.cell(), 1).unwrap_err();
    assert!(matches!(failure.cause, Cause::LimitExceeded { .. }));
    head_demand(head.cell(), 1 << 24).unwrap();
    let full = full_demand(head.cell(), 1 << 24).unwrap();
    assert_eq!(
        fixture.cold.symbols.resolve(family),
        None,
        "Full readmission cannot publish candidate meanings"
    );
    assert!(accepted.entity(fixture.entity).is_some());
    assert_eq!(
        accepted
            .entity_arena()
            .current_metadata_at(fixture.entity.slot_index())
            .unwrap()
            .effective_at,
        VersionId(1_000)
    );
    assert_eq!(
        full.full_view()
            .partition()
            .entity_arena
            .metadata_history_at(fixture.entity.slot_index())
            .unwrap()
            .len(),
        fixture.expected_history + 1
    );
    assert_eq!(
        full.full_view()
            .partition()
            .authoritative_content_digest(&symbols)
            .unwrap(),
        captured.leaf.content_digest
    );
    let mut oracle = region_load::load_full(&fixture.cold, &original, execution, 1 << 24)
        .unwrap()
        .partition
        .clone();
    oracle
        .entity_arena
        .apply_extra_update(fixture.entity.slot_index(), extra, VersionId(1_000));
    assert_eq!(
        oracle.authoritative_content_digest(&symbols).unwrap(),
        captured.leaf.content_digest
    );
    assert!(full
        .full_view()
        .partition()
        .entity_arena
        .history_prefixes
        .iter()
        .all(Option::is_none));
}

#[test]
fn owned_head_base_damage_is_sticky_and_never_invalidates_an_accepted_view() {
    let fixture = region_with_history();
    let request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let original = region_load::load_head(&fixture.cold, execution, 1 << 24).unwrap();
    let head = RelationalRegionResidency::resident_head(
        original.partition.clone(),
        original.commitment.clone().unwrap(),
        fixture.cold.schema.clone(),
        &fixture.cold.symbols,
    );
    fixture.source.corrupt.store(true, Ordering::SeqCst);
    let accepted = head.head_view();
    let failure = full_demand(head.cell(), 1 << 24).unwrap_err();
    assert_eq!(
        failure.subject,
        RelationalStorageSubject::Region(fixture.entity.partition_id)
    );
    assert_eq!(failure.cause, Cause::IntegrityMismatch);
    assert_eq!(head.cell().slot_counts(), fixture.cell.slot_counts());
    assert!(accepted.entity(fixture.entity).is_some());
    let reads = fixture.source.reads.load(Ordering::SeqCst);
    assert_eq!(full_demand(head.cell(), 1 << 24).unwrap_err(), failure);
    assert_eq!(head_demand(head.cell(), 1 << 24).unwrap_err(), failure);
    assert!(
        matches!(head.cell().capture_resident_content(&fixture.cold.symbols.interner_snapshot()),
        Err(RegionContentFailure::Storage(refused)) if refused == failure)
    );
    assert_eq!(fixture.source.reads.load(Ordering::SeqCst), reads);
}

#[test]
fn head_content_inspection_hashes_its_suffix_without_loading_the_base_or_trusting_a_cached_digest()
{
    let fixture = region_with_history();
    let request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let original = region_load::load_head(&fixture.cold, execution, 1 << 24).unwrap();
    let mut partial = original.partition.clone();
    let extra = partial
        .entity_arena
        .get_slot(fixture.entity.slot_index())
        .unwrap()
        .extra()
        .clone();
    partial.entity_arena.apply_extra_update(
        fixture.entity.slot_index(),
        extra.clone(),
        VersionId(1_000),
    );
    // Deliberately retain the old cached digest: inspection must hash the rows.
    let owned = RelationalRegionResidency::resident_head(
        partial,
        original.commitment.clone().unwrap(),
        fixture.cold.schema.clone(),
        &fixture.cold.symbols,
    );
    let reads = fixture.source.reads.load(Ordering::SeqCst);
    let digest = owned
        .cell()
        .capture_resident_content(&crate::symbols::data::StringInterner::default())
        .unwrap()
        .unwrap();
    assert_eq!(fixture.source.reads.load(Ordering::SeqCst), reads);
    assert_ne!(digest, fixture.expected_digest);
    let mut complete = region_load::load_full(&fixture.cold, &original, execution, 1 << 24)
        .unwrap()
        .partition
        .clone();
    complete
        .entity_arena
        .apply_extra_update(fixture.entity.slot_index(), extra, VersionId(1_000));
    assert_eq!(
        digest,
        complete
            .authoritative_content_digest(&fixture.cold.symbols.interner_snapshot())
            .unwrap()
    );
    assert!(owned.cell().reclaimable_unique_resident_payload_bytes() > 0);
}
