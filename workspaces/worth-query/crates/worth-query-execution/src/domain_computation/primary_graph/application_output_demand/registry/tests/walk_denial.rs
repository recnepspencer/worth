//! Refusal diagnostics identify the walk at either charge point.
use super::super::refreshed_rejoin::{newest_of_occurrence, WalkAdmission, WalkKind};
use super::*;

#[test]
fn every_request_walk_names_its_path_and_row_refusals_and_rejoin_keeps_its_text() {
    let selected = key("diagnostic", 1, 1);
    let mut state = DemandRegistryState::default();
    state.records.insert(
        selected.clone(),
        record(occurrence(), DemandState::Admitted, 0),
    );
    let cases = [
        (
            WalkKind::Rejoin,
            "refreshed output rejoin exceeds request work",
        ),
        (
            WalkKind::Supersession,
            "supersession occurrence walk exceeds request work",
        ),
        (
            WalkKind::TerminalCleanup,
            "terminal cleanup occurrence walk exceeds request work",
        ),
        (
            WalkKind::CacheReclaim,
            "cache reclaim occurrence walk exceeds request work",
        ),
        (
            WalkKind::JoinedReadyRelease,
            "joined Ready release occurrence walk exceeds request work",
        ),
        (
            WalkKind::RequiredMember,
            "required member occurrence walk exceeds request work",
        ),
    ];
    for (kind, subject) in cases {
        for work in [0, 2] {
            let mut admission = InvalidationEditAdmission::new(CompanionPreflightBudget {
                maximum_work_visits: Some(work),
                maximum_preparation_bytes: 8 * 1024 * 1024,
            });
            let denial = newest_of_occurrence(
                &state.records,
                &selected,
                &mut WalkAdmission::new(&mut admission, kind),
            )
            .unwrap_err();
            assert_eq!(
                denial.kind(),
                WorthQueryOutputDemandDenialKind::WorkBudgetExceeded
            );
            assert_eq!(denial.subject(), subject);
            assert_eq!(admission.charged_work(), work);
        }
    }
    let mut original = InvalidationEditAdmission::new(CompanionPreflightBudget {
        maximum_work_visits: Some(0),
        maximum_preparation_bytes: 8 * 1024 * 1024,
    });
    let denial = newest_of_occurrence(&state.records, &selected, &mut original).unwrap_err();
    assert_eq!(
        denial.subject(),
        "refreshed output rejoin exceeds request work"
    );
}
