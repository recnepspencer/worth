//! A paid path cannot inspect or compare a row whose unit is refused.
use super::super::demand_key::row_key_comparisons;
use super::super::supersession::reject_older_successor;
use super::*;

fn admission(work: u64) -> InvalidationEditAdmission {
    InvalidationEditAdmission::new(CompanionPreflightBudget {
        maximum_work_visits: work,
        maximum_preparation_bytes: 8 * 1024 * 1024,
    })
}

#[test]
fn retirement_paid_paths_refuse_the_first_row_before_any_key_comparison() {
    let selected = key("row-retirement", 2, 1);
    let older = key("row-retirement", 1, 1);
    let mut state = DemandRegistryState::default();
    for key in [&older, &selected] {
        state
            .records
            .insert(key.clone(), record(occurrence(), DemandState::Running, 1));
    }
    let mut prelude = record_admission();
    state.charge_record_lookup(&selected, &mut prelude).unwrap();
    let lookup = prelude.charged_work();
    let before = row_key_comparisons();
    let mut short = admission(lookup + 2);
    let denial = state
        .prepare_superseded_settlements(&selected, &mut short)
        .unwrap_err();
    assert_eq!(
        denial.kind(),
        WorthQueryOutputDemandDenialKind::WorkBudgetExceeded
    );
    assert_eq!(short.charged_work(), lookup + 2);
    assert_eq!(row_key_comparisons(), before);
    state
        .prepare_superseded_settlements(&selected, &mut record_admission())
        .unwrap();
    assert!(
        row_key_comparisons() > before,
        "the unrefused older row exercises the comparison witness"
    );
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
    assert_eq!(short.charged_work(), 2);
    assert_eq!(row_key_comparisons(), before);
    reject_older_successor(&state, &successor, &mut record_admission()).unwrap();
    assert!(row_key_comparisons() > before);
}
