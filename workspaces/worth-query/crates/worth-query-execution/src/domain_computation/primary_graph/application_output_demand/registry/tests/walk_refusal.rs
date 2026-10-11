//! A request that cannot buy the occurrence paths inspects no occurrence row.
use super::super::record_map::occurrence_rows_visited;
use super::super::supersession::reject_older_successor;
use super::*;

fn admission(work: u64) -> InvalidationEditAdmission {
    InvalidationEditAdmission::new(CompanionPreflightBudget {
        maximum_work_visits: work,
        maximum_preparation_bytes: 8 * 1024 * 1024,
    })
}

#[test]
fn settlement_retirement_refuses_unpaid_paths_before_inspecting_any_row() {
    let selected = key("retirement", 1, 1);
    let mut state = DemandRegistryState::default();
    state.records.insert(
        selected.clone(),
        record(occurrence(), DemandState::Admitted, 0),
    );
    let mut prelude = record_admission();
    state.charge_record_lookup(&selected, &mut prelude).unwrap();
    let lookup = prelude.charged_work();
    let before = occurrence_rows_visited();
    // Pay the real keyed prelude and the old row price, leaving the two seeks unpaid.
    let mut short = admission(lookup + 1);
    let denial = state
        .prepare_superseded_settlements(&selected, &mut short)
        .expect_err("the unpaid walk used to complete on the lookup-plus-row budget");
    assert_eq!(
        denial.kind(),
        WorthQueryOutputDemandDenialKind::WorkBudgetExceeded
    );
    assert_eq!(short.charged_work(), lookup);
    assert_eq!(occurrence_rows_visited(), before);
    assert_eq!(state.records.len(), 1);
    assert!(matches!(
        state.records[&selected].state,
        DemandState::Admitted
    ));

    let mut enough = admission(lookup + 3);
    assert!(state
        .prepare_superseded_settlements(&selected, &mut enough)
        .unwrap()
        .is_empty());
    // Beyond the unchanged keyed prelude: two seeks 2 + one occurrence row 1 = 3 (formerly 1).
    assert_eq!(enough.charged_work() - lookup, 3);
    // One record has bit length 1: two paths of (1 * 11 + 1) = 24 navigation.
    assert_eq!(
        enough.charged_navigation() - prelude.charged_navigation(),
        24
    );
    assert_eq!(occurrence_rows_visited(), before + 1);
}

#[test]
fn supersession_refuses_unpaid_paths_before_inspecting_any_row() {
    let successor = key("supersession", 2, 1);
    let older = key("supersession", 1, 1);
    let mut state = DemandRegistryState::default();
    state.records.insert(
        older.clone(),
        record(occurrence(), DemandState::Admitted, 1),
    );
    let mut prelude = record_admission();
    state
        .charge_record_lookup(&successor, &mut prelude)
        .unwrap();
    let lookup = prelude.charged_work();
    // Registration's real preceding record lookup fits; nothing remains for the walk.
    let mut short = admission(lookup);
    state.charge_record_lookup(&successor, &mut short).unwrap();
    let before = occurrence_rows_visited();
    let denial = reject_older_successor(&state, &successor, &mut short)
        .expect_err("the unpaid successor check used to complete with no work remaining");
    assert_eq!(
        denial.kind(),
        WorthQueryOutputDemandDenialKind::WorkBudgetExceeded
    );
    assert_eq!(short.charged_work(), lookup);
    assert_eq!(occurrence_rows_visited(), before);
    assert!(matches!(state.records[&older].state, DemandState::Admitted));
    assert_eq!(state.records[&older].interests, 1);

    let mut enough = admission(lookup + 3);
    state.charge_record_lookup(&successor, &mut enough).unwrap();
    reject_older_successor(&state, &successor, &mut enough).unwrap();
    // Beyond the unchanged keyed prelude: two seeks 2 + one occurrence row 1 = 3 (formerly 0).
    assert_eq!(enough.charged_work() - lookup, 3);
    assert_eq!(
        enough.charged_navigation() - prelude.charged_navigation(),
        24
    );
    assert_eq!(occurrence_rows_visited(), before + 1);
}
