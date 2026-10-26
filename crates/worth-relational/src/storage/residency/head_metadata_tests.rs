//! Current metadata is authenticated and sufficient for an append, not history.
use super::*;
use crate::storage::residency::region_image::{
    RegionHeadImage, RegionHistoryImage, RelationalRegionImage, RelationalRegionImageKind,
};

#[test]
fn head_contains_current_metadata_and_history_contains_only_earlier_versions() {
    let fixture = region_with_history();
    let head = decode_head(&fixture);
    let head_rows = &head.partition.entity_arena.metadata_history[0];
    assert_eq!(
        head_rows.len(),
        1,
        "one current metadata version belongs in Head"
    );
    let bytes = fixture.source.members.get(&fixture.cold.history).unwrap();
    let frame = RelationalRegionImage::decode(
        bytes,
        fixture.cold.history,
        fixture.cold.partition_id,
        fixture.cold.schema_carrier_digest,
        RelationalRegionImageKind::History,
    )
    .unwrap();
    let history: RegionHistoryImage = rmp_serde::from_slice(frame.payload).unwrap();
    assert_eq!(
        history.entities[0].len() + head_rows.len(),
        fixture.expected_history
    );
    assert!(history.entities[0].last().unwrap().effective_at < head_rows[0].effective_at);
    assert!(
        !history.relations[0].is_empty(),
        "fixture has real earlier endpoint versions"
    );
    assert_eq!(
        fixture.source.reads.load(Ordering::SeqCst),
        0,
        "inspect producer bytes without warming the receiver"
    );
}

#[test]
fn a_head_append_matches_a_separately_recomputed_full_history_root() {
    let fixture = region_with_history();
    let backing = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&backing);
    let head = region_load::load_head(&fixture.cold, execution, 1 << 24).unwrap();
    let symbols = fixture.cold.symbols.interner_snapshot();
    let slot = fixture.entity.slot_index();
    let mut written_head = head.partition.clone();
    let extra = written_head
        .entity_arena
        .get_slot(slot)
        .unwrap()
        .extra()
        .clone();
    let version = crate::identity::data::VersionId(1_000);
    written_head
        .entity_arena
        .apply_extra_update(slot, extra.clone(), version);
    let mut journal = crate::storage::overlay::PartitionMutationJournal::default();
    journal.entity_slots.insert(slot);
    let (appended, _) = PartitionContentCommitment::capture(
        &written_head,
        &symbols,
        Some((head.commitment.as_ref().unwrap(), &head.partition, &journal)),
    )
    .unwrap();
    assert_eq!(
        written_head
            .entity_arena
            .metadata_history_at(slot)
            .unwrap()
            .len(),
        2,
        "only the old current row and the appended row are resident"
    );
    assert_eq!(
        fixture.source.history_reads.load(Ordering::SeqCst),
        0,
        "retiring and appending under the carried frontier loads no history"
    );

    let full = region_load::load_full(&fixture.cold, &head, execution, 1 << 24).unwrap();
    let mut written_full = full.partition.clone();
    written_full
        .entity_arena
        .apply_extra_update(slot, extra, version);
    let (oracle, _) = PartitionContentCommitment::capture(&written_full, &symbols, None).unwrap();
    assert_eq!(
        appended.digest(fixture.entity.partition_id),
        oracle.digest(fixture.entity.partition_id)
    );
    assert_eq!(
        written_full
            .entity_arena
            .metadata_history_at(slot)
            .unwrap()
            .len(),
        fixture.expected_history + 1
    );
}

#[derive(Clone, Copy, Debug)]
enum Tamper {
    MissingCurrent,
    ChangedCurrent,
    ExtraFrontier,
}

#[test]
fn malformed_current_metadata_and_frontier_coverage_are_sticky_typed_refusals() {
    for tamper in [
        Tamper::MissingCurrent,
        Tamper::ChangedCurrent,
        Tamper::ExtraFrontier,
    ] {
        let fixture = region_with_history();
        let mut head = decode_head(&fixture);
        match tamper {
            Tamper::MissingCurrent => head.partition.entity_arena.metadata_history[0].clear(),
            Tamper::ChangedCurrent => {
                head.partition.entity_arena.metadata_history[0][0].generation ^= 1
            }
            Tamper::ExtraFrontier => {
                let mut wire = serde_json::to_value(&head.history_frontiers).unwrap();
                let entries = wire["entities"].as_array_mut().unwrap();
                let mut extra = entries[0].clone();
                extra[0] = serde_json::Value::from(u64::MAX - 1);
                entries.push(extra);
                head.history_frontiers = serde_json::from_value(wire).unwrap();
            }
        }
        let (cell, source) = cell_with_head(fixture, head);
        let first = head_demand(&cell, 1 << 24).unwrap_err();
        assert_eq!(first.cause, Cause::Damaged, "{tamper:?}");
        assert!(matches!(first.subject, RelationalStorageSubject::Region(_)));
        let reads = source.reads.load(Ordering::SeqCst);
        assert_eq!(
            full_demand(&cell, 1 << 24).unwrap_err(),
            first,
            "{tamper:?}"
        );
        assert_eq!(source.reads.load(Ordering::SeqCst), reads);
        assert_eq!(cell.leases.load(Ordering::SeqCst), 0);
    }
}

fn decode_head(fixture: &RegionFixture) -> RegionHeadImage {
    let bytes = fixture.source.members.get(&fixture.cold.head).unwrap();
    let frame = RelationalRegionImage::decode(
        bytes,
        fixture.cold.head,
        fixture.cold.partition_id,
        fixture.cold.schema_carrier_digest,
        RelationalRegionImageKind::Head,
    )
    .unwrap();
    rmp_serde::from_slice(frame.payload).unwrap()
}

fn cell_with_head(
    fixture: RegionFixture,
    head: RegionHeadImage,
) -> (Arc<RelationalRegionResidency>, Arc<CountedSource>) {
    let mut cold = fixture.cold;
    let (locator, bytes) = RelationalRegionImage::encode(
        cold.partition_id,
        cold.schema_carrier_digest,
        RelationalRegionImageKind::Head,
        &head,
    )
    .unwrap();
    let mut members = fixture.source.members.clone();
    members.insert(locator, bytes);
    let source = Arc::new(CountedSource {
        members,
        history: cold.history,
        reads: AtomicUsize::new(0),
        history_reads: AtomicUsize::new(0),
        corrupt: AtomicBool::new(false),
    });
    cold.head = locator;
    cold.source = source.clone();
    (RelationalRegionResidency::cold(cold).unwrap(), source)
}
