//! An advancing caller cannot replace the original staged request controls.
use super::*;
use worth_execution::{CancellationSource, MapKernelStop, SerialMemoryBudget, SerialRequest};
use worth_foundational::facade::HomeProgramTransitionPlanDigest;
use worth_query_testkit_installation::facade::installation::application_home_binding as bindings;
use worth_store_application::ApplicationHomeTransitionIntentOutcome as Outcome;

#[test]
fn a_live_advancer_cannot_publish_an_intent_after_its_staging_request_is_cancelled() {
    let directory = tempfile::tempdir().unwrap();
    let driver = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&driver);
    let mut fixture = super::transition_opening::open_transition(directory.path(), &execution);
    let cancellation = CancellationSource::new();
    let budget = SerialMemoryBudget::new(None);
    let original = SerialRequest::from_memory(budget.clone(), cancellation.token(), None);
    let original_execution = ExecutionRequest::serial(&original);
    let source = fixture
        .home
        .world()
        .observation_port()
        .observe_root_product_branch()
        .unwrap();
    let reserved = fixture
        .home
        .reserve_home_program_transition_plan(
            source.clone(),
            HomeProgramTransitionPlanDigest::from_recorded([7; 32]),
            3,
            super::transition_opening::target_meaning(),
            &original_execution,
        )
        .unwrap();
    fixture
        .home
        .stage_home_program_transition_intent(
            reserved,
            bindings::admitted_transition_successor_binding(),
            &original_execution,
        )
        .unwrap();
    cancellation.cancel();
    drop(original);
    assert!(
        budget.charged_memory_bytes() > 0,
        "the original operation retains its actual prepaid backing"
    );
    let Outcome::Refused(refused) = fixture
        .home
        .advance_home_program_transition_intent(&execution)
        .unwrap()
    else {
        panic!("cancelled staging controls must refuse before intent publication")
    };
    assert!(matches!(
        refused.reason(),
        worth_store::physical_runtime::OrderedJournalOperationFailure::OriginalRequestStopped(
            MapKernelStop::Cancelled
        )
    ));
    assert!(fixture
        .home
        .physical_serving_for_test()
        .ordered_journal(OrderedJournalName::new("home.transitions").unwrap())
        .unwrap()
        .head()
        .unwrap()
        .is_none());
    assert_eq!(
        fixture
            .home
            .world()
            .observation_port()
            .observe_root_product_branch()
            .unwrap(),
        source
    );
    drop(source);
    // Abandon this still-owned failed preparation; no successful close is claimed.
    drop(fixture);
    drop(refused);
    assert_eq!(budget.charged_memory_bytes(), 0);
    let reopened = open(directory.path(), &execution);
    close(reopened.home, &execution);
}
