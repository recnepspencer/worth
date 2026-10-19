//! A request that cannot buy the occurrence paths inspects no occurrence row.
use super::super::record_map::occurrence_rows_visited;
use super::super::supersession::reject_older_successor;
use super::*;

fn admission(work: u64) -> InvalidationEditAdmission {
    InvalidationEditAdmission::new(CompanionPreflightBudget {
        maximum_work_visits: Some(work),
        maximum_preparation_bytes: None,
    })
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
