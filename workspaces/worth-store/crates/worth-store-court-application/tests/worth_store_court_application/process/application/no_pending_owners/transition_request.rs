//! Only the declared open-request transition admits its original intent.
use super::*;
use worth_foundational::facade::HomeProgramTransitionPlanDigest;
use worth_query_testkit_installation::facade::installation::application_home_binding as bindings;
use worth_store_application::{
    ApplicationHomeTransitionIntentDenial as Intent,
    ApplicationHomeTransitionPlanningDenial as Plan,
};

#[test]
fn a_current_home_cannot_reserve_a_transition_without_an_open_request() {
    let directory = tempfile::tempdir().unwrap();
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let fixture = open(directory.path(), &execution);
    let original = fixture.bootstrap_root(&execution);
    let refused = fixture
        .home
        .reserve_home_program_transition_plan(
            original.product_branch().clone(),
            HomeProgramTransitionPlanDigest::from_recorded([7; 32]),
            3,
            super::transition_opening::target_meaning(),
            &crate::batches::transition_body_roster(&execution, 3),
            &execution,
        )
        .err()
        .unwrap();
    assert!(matches!(
        refused,
        Plan::Intent(Intent::TransitionNotRequested)
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
        &fixture
            .home
            .world()
            .observation_port()
            .observe_root_product_branch()
            .unwrap(),
        original.product_branch()
    );
    drop(original);
    close(fixture.home, &execution);
}

#[test]
fn a_transition_intent_cannot_substitute_another_genuinely_admitted_successor() {
    let directory = tempfile::tempdir().unwrap();
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let mut fixture = super::transition_opening::open_transition(directory.path(), &execution);
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
            &crate::batches::transition_body_roster(&execution, 3),
            &execution,
        )
        .unwrap();
    assert!(matches!(
        fixture.home.stage_home_program_transition_intent(
            reserved,
            bindings::admitted_home_binding(),
            &execution
        ),
        Err(Intent::TransitionPending)
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
    close(fixture.home, &execution);
}
