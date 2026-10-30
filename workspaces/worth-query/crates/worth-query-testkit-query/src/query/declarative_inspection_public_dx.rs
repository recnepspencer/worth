pub use worth_query::facade::inspection::{
    declare, inspection_basis, ScopedInspectionBasis, WorthQueryInspectionOutcome,
    WorthQueryOutcomeNavigation, WorthQueryOutcomePosture, WorthQueryWorkspace,
};
pub use worth_query::facade::read::WorthQueryReadCompletion;
pub use worth_query::facade::read::{project_facts, WorthQueryProjectionOutcome};

pub fn ordinary_inspection_journey(
    completion: &WorthQueryReadCompletion,
    basis: ScopedInspectionBasis,
    workspace: &WorthQueryWorkspace,
) -> WorthQueryInspectionOutcome {
    declare(completion)
        .with_rich_inspection()
        .using(inspection_basis(basis))
        .run(workspace)
}

pub fn common_outcome_navigation(outcome: &WorthQueryInspectionOutcome) {
    let posture = outcome.posture();
    assert!(matches!(
        posture,
        WorthQueryOutcomePosture::Completed
            | WorthQueryOutcomePosture::Advisory
            | WorthQueryOutcomePosture::Violation
            | WorthQueryOutcomePosture::Deferred
            | WorthQueryOutcomePosture::Unavailable
    ));
}

pub fn ordinary_projection_journey(
    completion: &WorthQueryReadCompletion,
) -> WorthQueryProjectionOutcome {
    completion.consume_projection(project_facts().entity_identities())
}

pub fn assert_navigation<T: WorthQueryOutcomeNavigation>() {}
