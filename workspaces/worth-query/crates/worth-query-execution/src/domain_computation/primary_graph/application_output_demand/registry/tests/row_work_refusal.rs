//! A paid path cannot inspect or compare a row whose unit is refused.
use super::super::demand_key::row_key_comparisons;
use super::super::supersession::reject_older_successor;
use super::*;

fn admission(work: u64) -> InvalidationEditAdmission {
    InvalidationEditAdmission::new(CompanionPreflightBudget {
        maximum_work_visits: Some(work),
        maximum_preparation_bytes: 8 * 1024 * 1024,
    })
}

#[test]
fn supersession_paid_paths_refuse_the_first_row_before_any_key_comparison() {
    let successor = key("row-supersession", 2, 1);
    let mut state = DemandRegistryState::default();
    state.records.insert(
        key("row-supersession", 1, 1),
        record(occurrence(), DemandState::Admitted, 1),
    );
    let mut short = admission(2);
    let before = row_key_comparisons();
    let denial = reject_older_successor(&state, &successor, &mut short).unwrap_err();
    assert_eq!(
        denial.kind(),
        WorthQueryOutputDemandDenialKind::WorkBudgetExceeded
    );
    assert_eq!(
        denial.subject(),
        "supersession occurrence walk exceeds request work"
    );
    assert_eq!(short.charged_work(), 2);
    assert_eq!(row_key_comparisons(), before);
    reject_older_successor(&state, &successor, &mut record_admission()).unwrap();
    assert!(row_key_comparisons() > before);
}

#[test]
fn registry_supersede_predecessors_refuses_first_row_after_paid_seeks() {
    let successor = key("collected-supersession", 3, 1);
    let mut state = DemandRegistryState::default();
    for generation in 1..=2 {
        state.records.insert(
            key("collected-supersession", generation, 1),
            record(occurrence(), DemandState::Admitted, 1),
        );
    }
    let mut short = admission(2);
    let before = row_key_comparisons();
    let denial = supersede_predecessors(&mut state, &successor, &mut short).unwrap_err();
    assert_eq!(
        denial.kind(),
        WorthQueryOutputDemandDenialKind::WorkBudgetExceeded
    );
    assert_eq!(short.charged_work(), 2);
    assert_eq!(row_key_comparisons(), before);
    assert!(state
        .records
        .values()
        .all(|row| matches!(row.state, DemandState::Admitted)));
    supersede_predecessors(&mut state, &successor, &mut record_admission()).unwrap();
    assert!(
        row_key_comparisons() > before,
        "the first row really reaches the semantic comparison"
    );
}
