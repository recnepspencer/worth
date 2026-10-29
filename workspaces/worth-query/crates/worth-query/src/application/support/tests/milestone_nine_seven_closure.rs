use crate::application::{
    WorthQueryMilestoneClosureStatus, WorthQueryMilestoneNineSevenDerivedClosure,
    WorthQueryMilestoneNineSevenPhaseClosure,
};

#[test]
fn closes_only_from_required_phase_local_closures() {
    let closed = phase_local_closure(WorthQueryMilestoneClosureStatus::Closed);

    assert_eq!(closed.status(), WorthQueryMilestoneClosureStatus::Closed);
    assert_eq!(
        closed.phase_closures().len(),
        WorthQueryMilestoneNineSevenDerivedClosure::required_phases().len()
    );
    assert!(!closed.closure_digest().is_empty());

    for index in 0..WorthQueryMilestoneNineSevenDerivedClosure::required_phases().len() {
        let sabotaged = phase_local_closure_with_status_at(
            index,
            WorthQueryMilestoneClosureStatus::Open,
            "phase-local-sabotage",
        );
        assert_ne!(
            sabotaged.status(),
            WorthQueryMilestoneClosureStatus::Closed,
            "required phase {index} must reopen milestone 9.7 when its local proof opens"
        );
    }

    let empty_digest =
        phase_local_closure_with_status_at(2, WorthQueryMilestoneClosureStatus::Closed, "");
    assert_ne!(
        empty_digest.status(),
        WorthQueryMilestoneClosureStatus::Closed,
        "closed status without a phase-local evidence digest must not close milestone 9.7"
    );
}

#[test]
fn support_profile_contract_names_defended_exclusion() {
    let contract =
        WorthQueryMilestoneNineSevenDerivedClosure::support_profile_publication_contract();

    assert_eq!(contract.status(), WorthQueryMilestoneClosureStatus::Partial);
    assert_eq!(
        contract.defended_exclusions(),
        &["store-backed execution parity belongs to Milestone 10".to_string()]
    );
    assert!(contract
        .phase_closures()
        .iter()
        .all(|closure| closure.status() == WorthQueryMilestoneClosureStatus::Partial));
}

fn phase_local_closure(
    status: WorthQueryMilestoneClosureStatus,
) -> WorthQueryMilestoneNineSevenDerivedClosure {
    WorthQueryMilestoneNineSevenDerivedClosure::derive_from_phase_closures(
        WorthQueryMilestoneNineSevenDerivedClosure::required_phases()
            .iter()
            .copied()
            .map(|phase| {
                WorthQueryMilestoneNineSevenPhaseClosure::new(
                    phase,
                    status,
                    format!("{phase}:digest"),
                )
            }),
        ["store-backed execution parity belongs to Milestone 10"],
    )
}

fn phase_local_closure_with_status_at(
    target_index: usize,
    target_status: WorthQueryMilestoneClosureStatus,
    target_digest: &str,
) -> WorthQueryMilestoneNineSevenDerivedClosure {
    WorthQueryMilestoneNineSevenDerivedClosure::derive_from_phase_closures(
        WorthQueryMilestoneNineSevenDerivedClosure::required_phases()
            .iter()
            .copied()
            .enumerate()
            .map(|(index, phase)| {
                let status = if index == target_index {
                    target_status
                } else {
                    WorthQueryMilestoneClosureStatus::Closed
                };
                let digest = if index == target_index {
                    target_digest.to_string()
                } else {
                    format!("{phase}:digest")
                };
                WorthQueryMilestoneNineSevenPhaseClosure::new(phase, status, digest)
            }),
        ["store-backed execution parity belongs to Milestone 10"],
    )
}
