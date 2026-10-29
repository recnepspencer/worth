pub(crate) use worth_query_testkit_query::facade::query::public_declarative_journeys::declarative_inspection_public_dx::*;

#[test]
fn inspection_journey_uses_only_ordinary_facade_vocabulary() {
    let _ = ordinary_inspection_journey
        as fn(
            &WorthQueryReadCompletion,
            ScopedInspectionBasis,
            &WorthQueryWorkspace,
        ) -> WorthQueryInspectionOutcome;
    let _ = common_outcome_navigation as fn(&WorthQueryInspectionOutcome);
    let _ =
        ordinary_projection_journey as fn(&WorthQueryReadCompletion) -> WorthQueryProjectionOutcome;

    assert_navigation::<worth_query::facade::read::WorthQueryReadOutcome>();
    assert_navigation::<WorthQueryProjectionOutcome>();
    assert_navigation::<worth_query::facade::aggregate::WorthQueryCountOutcome>();
    assert_navigation::<worth_query::facade::live::WorthQueryLiveOpenOutcome>();
    assert_navigation::<worth_query::facade::history::WorthQueryHistoricalOutcome>();
    assert_navigation::<worth_query::facade::comparison::WorthQueryComparisonOutcome>();
    assert_navigation::<worth_query::facade::mutation::WorthQueryMutationOutcome>();
    assert_navigation::<worth_query::facade::preview::WorthQueryPreviewJourneyOutcome>();
    assert_navigation::<worth_query::facade::workflow::WorthQueryWorkflowOutcome>();
    assert_navigation::<worth_query::facade::workflow::WorthQueryWritebackOutcome>();
    assert_navigation::<worth_query::facade::workflow::WorthQueryBranchMergeOutcome>();
    assert_navigation::<worth_query::facade::domain::WorthQueryInstalledDomainWorkflowOutcome>();
    assert_navigation::<WorthQueryInspectionOutcome>();
}
