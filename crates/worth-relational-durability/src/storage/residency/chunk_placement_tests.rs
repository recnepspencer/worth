//! A readmitted cold cell serves its recorded `REG8` chunk placement through
//! its own member custody and working-set pool (Store M4.3c `S2`): scans fault
//! only chunk members, summary pruning skips absent kinds and edges, the open
//! reader pins the chunk roster past the cell's release, and custody retires
//! once the last reader closes.
use super::region_fixture::CustodySource;
use super::*;
use std::sync::Mutex;
use worth_relational_contracts::internal_port::config::data::residency_budget::ResidencyBudget;
use worth_relational_storage::internal_port::storage::residency::chunked::region_reader::RecordedChunkScan;
use worth_relational_storage::internal_port::storage::residency::chunked::slot_range::SlotDomain;
use worth_relational_storage::internal_port::storage::residency::chunked::summary_scan::{
    ScanHit, ScanPruning,
};
use worth_relational_storage::internal_port::storage::residency::working_set::{
    ResidencyPoolKind, WorkingSetAccountant,
};

#[test]
fn a_cold_cell_scans_its_recorded_chunks_without_materializing_the_region() {
    let fixture = region_with_history();
    let active = Arc::new(AtomicUsize::new(0));
    let mut cold = fixture.cold.clone();
    cold.source = Some(Arc::new(CustodySource {
        source: fixture.source.clone(),
        active: active.clone(),
        refusal: Mutex::new(None),
    }));
    let placement = cold.chunks.clone();
    assert!(!placement.chunks.is_empty() && !placement.members.is_empty());
    for member in cold.chunks.members.iter() {
        assert!(
            fixture.source.members.contains_key(member),
            "capture wrote it"
        );
    }
    let cell = RelationalRegionResidency::cold(cold).unwrap();
    let unbound = cell.recorded_chunks().err().unwrap();
    assert_eq!(unbound.cause, Cause::SourceUnavailable);
    let accountant = WorkingSetAccountant::new(ResidencyBudget {
        head_working_set_bytes: 1 << 20,
        historical_working_set_bytes: 1 << 20,
        max_in_flight_fill_bytes: 1 << 20,
        streaming_window_bytes: 4096,
    });
    cell.bind_working_set(&accountant, ResidencyPoolKind::Head)
        .unwrap();
    let chunks = cell.recorded_chunks().unwrap();
    assert_eq!(
        chunks.directory().root_semantic_merkle(),
        placement.root_semantic_merkle
    );
    let slots = |scan, pruning| {
        let outcome = chunks.scan(scan, pruning).unwrap();
        outcome.hits
    };
    let entities = RecordedChunkScan::HeadKind(SlotDomain::Entity, KindId(1));
    let expected = (0..3)
        .map(|slot| ScanHit {
            slot,
            relation: None,
        })
        .collect::<Vec<_>>();
    assert_eq!(slots(entities, ScanPruning::Summaries), expected);
    assert_eq!(slots(entities, ScanPruning::Canonical), expected);
    let relations = RecordedChunkScan::HeadKind(SlotDomain::Relation, KindId(2));
    assert_eq!(slots(relations, ScanPruning::Summaries).len(), 1);
    let outgoing = slots(
        RecordedChunkScan::Adjacency(KindId(2)),
        ScanPruning::Summaries,
    );
    assert_eq!(
        outgoing,
        [ScanHit {
            slot: 0,
            relation: Some(fixture.relation),
        }]
    );
    let absent = RecordedChunkScan::Adjacency(KindId(99));
    assert_eq!(
        chunks
            .scan(absent, ScanPruning::Summaries)
            .unwrap()
            .chunks_read,
        0
    );
    assert!(fixture.source.reads.load(Ordering::SeqCst) > 0);
    assert_eq!(fixture.source.history_reads.load(Ordering::SeqCst), 0);
    assert!(!cell.working_set_observation().resident);

    drop(cell);
    assert!(
        active.load(Ordering::SeqCst) > 0,
        "the open reader still pins the recorded chunk roster"
    );
    assert_eq!(slots(entities, ScanPruning::Canonical), expected);
    drop(chunks);
    assert_eq!(active.load(Ordering::SeqCst), 0, "custody retires cleanly");
}

#[test]
fn recorded_chunks_confines_reads_to_admitted_roster_and_exports_without_fallback() {
    use worth_relational_storage::internal_port::storage::data::checkpoint_member::RelationalCheckpointMember;
    let fixture = region_with_history();
    let cold = fixture.cold.clone();
    let cell = RelationalRegionResidency::cold(cold.clone()).unwrap();
    let carried = cold
        .chunks
        .members
        .iter()
        .map(|&locator| RelationalCheckpointMember::Carried { locator })
        .collect::<Vec<_>>();
    let exports = cell.checkpoint_export_sources(&carried, None).unwrap();
    assert_eq!(exports.len(), cold.chunks.members.len());
    for locator in &cold.chunks.members {
        let (_, export_source) = exports.get(locator).expect("chunk member exported");
        let bytes = export_source.read_member(*locator, locator.length).unwrap();
        assert_eq!(bytes.len() as u64, locator.length);
    }
    let accountant = WorkingSetAccountant::new(ResidencyBudget {
        head_working_set_bytes: 1 << 20,
        historical_working_set_bytes: 1 << 20,
        max_in_flight_fill_bytes: 1 << 20,
        streaming_window_bytes: 4096,
    });
    cell.bind_working_set(&accountant, ResidencyPoolKind::Head)
        .unwrap();
    let chunks = cell.recorded_chunks().unwrap();
    // `cold.head` exists in the cell's backing source, but is outside the
    // admitted chunk roster and must be refused as Damaged.
    let mut buf = [0_u8; 16];
    assert_eq!(
        chunks.source().read_range(cold.head, 0, &mut buf),
        Err(Cause::Damaged)
    );
    // Once a cell latches proven damage, `recorded_chunks()` surfaces that
    // terminal damage instead of serving chunks or `SourceUnavailable`.
    fixture.source.corrupt.store(true, Ordering::SeqCst);
    let (_, export_source) = exports
        .get(&cold.chunks.members[0])
        .expect("chunk member exported");
    let _ = export_source.read_member(cold.chunks.members[0], cold.chunks.members[0].length);
    let damaged = cell.recorded_chunks().err().unwrap();
    assert_eq!(damaged.cause, Cause::IntegrityMismatch);
}
