//! A readmitted cold cell serves its `CHK24` paged record directory through
//! its own member custody and working-set pool (Store M4.3d `G7`, `S2`):
//! readmission reads no member, kind presence and absence answer from the
//! descriptor, a kind scan faults only directory pages (never the region head
//! or history), carried directory pages export without a fallback source, and
//! the directory never holds cold custody past its cell.
use super::region_fixture::CustodySource;
use super::*;
use std::sync::Mutex;
use worth_relational_contracts::internal_port::config::data::residency_budget::ResidencyBudget;
use worth_relational_storage::internal_port::storage::data::checkpoint_member::RelationalCheckpointMember;
use worth_relational_storage::internal_port::storage::residency::working_set::{
    ResidencyPoolKind, WorkingSetAccountant,
};

fn tight() -> Arc<WorkingSetAccountant> {
    WorkingSetAccountant::new(ResidencyBudget {
        head_working_set_bytes: 64 * 1024,
        historical_working_set_bytes: 64 * 1024,
        max_in_flight_fill_bytes: 64 * 1024,
        streaming_window_bytes: 4096,
    })
}

#[test]
fn a_cold_cell_readmits_its_paged_directory_and_faults_only_directory_pages() {
    let fixture = region_with_history();
    let active = Arc::new(AtomicUsize::new(0));
    let mut cold = fixture.cold.clone();
    cold.source = Some(Arc::new(CustodySource {
        source: fixture.source.clone(),
        active: active.clone(),
        refusal: Mutex::new(None),
    }));
    assert!(!cold.directory.pages.is_empty());
    let cell = RelationalRegionResidency::cold(cold.clone()).unwrap();
    cell.bind_working_set(&tight(), ResidencyPoolKind::Head)
        .unwrap();
    let reads = fixture.source.reads.load(Ordering::SeqCst);
    let directory = cell.record_directory(cold.directory.clone());
    assert!(directory.has_entity_kind(KindId(1)) && directory.has_relation_kind(KindId(2)));
    assert_eq!(directory.entity_ids(KindId(99)).count(), 0);
    assert_eq!(directory.page_observation().unwrap().faults, 0);
    assert_eq!(
        fixture.source.reads.load(Ordering::SeqCst),
        reads,
        "readmission, presence and absence read no member"
    );

    let entities = directory
        .entity_ids(KindId(1))
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(entities.len(), 3);
    assert!(entities.contains(&fixture.entity));
    let relations = directory
        .relation_ids(KindId(2))
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(relations, [fixture.relation]);
    let faults = directory.page_observation().unwrap().faults;
    assert!(faults > 0);
    assert_eq!(
        fixture.source.reads.load(Ordering::SeqCst) - reads,
        faults as usize,
        "only directory pages were read"
    );
    assert_eq!(fixture.source.history_reads.load(Ordering::SeqCst), 0);
    assert!(
        !cell.working_set_observation().resident,
        "the cell stays cold"
    );

    let unfaulted = cell.record_directory(cold.directory.clone());
    drop(cell);
    assert_eq!(
        active.load(Ordering::SeqCst),
        0,
        "the directory never extends the cell's custody"
    );
    assert_eq!(directory.entity_ids(KindId(1)).count(), 3, "resident pages");
    let refused = unfaulted
        .entity_ids(KindId(1))
        .filter_map(Result::err)
        .collect::<Vec<_>>();
    assert_eq!(refused.len(), 1);
    assert_eq!(refused[0].cause, Cause::SourceUnavailable);
}

#[test]
fn carried_directory_pages_export_without_fallback_and_damage_refuses_scans() {
    let fixture = region_with_history();
    let cold = fixture.cold.clone();
    let cell = RelationalRegionResidency::cold(cold.clone()).unwrap();
    let carried = cold
        .directory
        .pages
        .iter()
        .map(|&locator| RelationalCheckpointMember::Carried { locator })
        .collect::<Vec<_>>();
    let exports = cell.checkpoint_export_sources(&carried, None).unwrap();
    assert_eq!(exports.len(), cold.directory.pages.len());
    for locator in &cold.directory.pages {
        let (_, source) = exports.get(locator).expect("directory page exported");
        assert_eq!(
            source.read_member(*locator, locator.length).unwrap().len() as u64,
            locator.length
        );
    }
    let directory = cell.record_directory(cold.directory.clone());
    fixture.source.corrupt.store(true, Ordering::SeqCst);
    let page = cold.directory.pages[0];
    let _ = exports[&page].1.read_member(page, page.length);
    let refused = directory
        .entity_ids(KindId(1))
        .filter_map(Result::err)
        .collect::<Vec<_>>();
    assert_eq!(refused.len(), 1);
    assert_eq!(refused[0].cause, Cause::IntegrityMismatch);
}
