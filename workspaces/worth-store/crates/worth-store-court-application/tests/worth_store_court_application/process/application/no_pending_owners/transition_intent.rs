//! The Store intent protocol uses genuine Home and Query binding admission.
//! This court proves the intent boundary, not Query-program continuation.
use super::*;
use worth_foundational::facade::HomeProgramTransitionPlanDigest;
use worth_store_application::{
    ApplicationHomeTransitionIntentDenial, ApplicationHomeTransitionIntentOutcome,
};

#[test]
fn home_transition_intent_is_positioned_before_any_world_or_native_migration_effect() {
    intent_boundary(false);
}

#[test]
fn transition_intent_retains_the_exact_canonical_predecessor_and_world_assignments() {
    intent_boundary(true);
}

fn intent_boundary(committed_native_predecessor: bool) {
    let directory = tempfile::tempdir().unwrap();
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let fixture = super::transition_opening::open_transition(directory.path(), &execution);
    let source = fixture
        .home
        .world()
        .observation_port()
        .observe_root_product_branch()
        .unwrap();
    if committed_native_predecessor {
        let cancellation = worth_runtime_world::facade::RuntimeWorldCancellationSource::new();
        let prepared = fixture.prepare_publication(
            &execution,
            source.clone(),
            crate::batches::create(0),
            &cancellation.token(),
        );
        let performed = fixture.publish(&execution, prepared, &cancellation.token());
        drop(performed.consume());
    }
    let mut home = fixture.home;
    let source = home
        .world()
        .observation_port()
        .observe_root_product_branch()
        .unwrap();
    let native_predecessor = home
        .relational()
        .runtime()
        .owner_component_services()
        .basis_port()
        .export_recorded_current_component(&execution, source.basis().relational_basis())
        .unwrap();
    match native_predecessor.target() {
        worth_relational::facade::branch::RecordedRelationalComponentTarget::Empty => {
            assert!(!committed_native_predecessor)
        }
        worth_relational::facade::branch::RecordedRelationalComponentTarget::Canonical(_) => {
            assert!(committed_native_predecessor)
        }
    }
    let effect_journals = [
        worth_store::physical_runtime::RELATIONAL_CANONICAL_COMMITS,
        worth_store::physical_runtime::RELATIONAL_BRANCH_TRANSITIONS,
        "world.custody",
        "world.occurrences",
    ];
    let original_heads = effect_journals.map(|name| {
        home.physical_serving_for_test()
            .ordered_journal(OrderedJournalName::new(name).unwrap())
            .unwrap()
            .head()
            .unwrap()
    });
    let reserved = home
        .reserve_home_program_transition_plan(
            source.clone(),
            HomeProgramTransitionPlanDigest::from_recorded([7; 32]),
            3,
            super::transition_opening::target_meaning(),
            &execution,
        )
        .unwrap();
    let description = reserved.description().clone();
    let assignments = reserved.assignments().to_vec();
    let expected_binding = worth_query_testkit_installation::facade::installation::application_home_binding::admitted_home_binding()
        .consume(|admitted| admitted.into_description());
    home.stage_home_program_transition_intent(reserved,
        worth_query_testkit_installation::facade::installation::application_home_binding::admitted_transition_successor_binding(), &execution).unwrap();
    assert!(home
        .physical_serving_for_test()
        .ordered_journal(OrderedJournalName::new("home.transitions").unwrap())
        .unwrap()
        .head()
        .unwrap()
        .is_none());
    let intent = home.home_program_transition_intent().unwrap();
    assert_eq!(intent.plan(), &description);
    assert_eq!(intent.assignments(), assignments.as_slice());
    assert_eq!(intent.predecessor(), &expected_binding);
    let expected_successor = worth_query_testkit_installation::facade::installation::application_home_binding::admitted_transition_successor_binding().consume(|admitted| admitted.into_description());
    assert_eq!(intent.successor(), &expected_successor);
    assert_eq!(intent.native_predecessor(), &native_predecessor);
    loop {
        execution
            .consult()
            .expect("the original intent request remains live");
        match home
            .advance_home_program_transition_intent(&execution)
            .unwrap()
        {
            ApplicationHomeTransitionIntentOutcome::Pending => std::thread::yield_now(),
            ApplicationHomeTransitionIntentOutcome::Committed => break,
            other => panic!("the actual original intent must commit: {other:?}"),
        }
    }
    let head = home
        .physical_serving_for_test()
        .ordered_journal(OrderedJournalName::new("home.transitions").unwrap())
        .unwrap()
        .head()
        .unwrap()
        .unwrap();
    assert_eq!(
        head.position(),
        description.identity().intent_position().get()
    );
    for _ in 0..2 {
        assert!(matches!(
            home.advance_home_program_transition_intent(&execution)
                .unwrap(),
            ApplicationHomeTransitionIntentOutcome::Committed
        ));
        assert_eq!(
            home.physical_serving_for_test()
                .ordered_journal(OrderedJournalName::new("home.transitions").unwrap())
                .unwrap()
                .head()
                .unwrap(),
            Some(head)
        );
    }
    assert_eq!(
        home.world()
            .observation_port()
            .observe_root_product_branch()
            .unwrap(),
        source
    );
    for (index, name) in effect_journals.into_iter().enumerate() {
        assert_eq!(
            home.physical_serving_for_test()
                .ordered_journal(OrderedJournalName::new(name).unwrap())
                .unwrap()
                .head()
                .unwrap(),
            original_heads[index],
            "intent must precede every effect in {name}"
        );
    }
    let refused = home
        .seal_world(&execution)
        .err()
        .expect("the original plan is still in flight");
    assert!(matches!(
        refused.reason(),
        worth_store_application::ApplicationHomeCloseDenial::World(
            worth_runtime_world::facade::RuntimeWorldServiceDenial::Denied(
                worth_runtime_world::facade::RuntimeWorldCloseDenial::InFlightCriticalSection
            )
        )
    ));
    assert_contended(directory.path(), &execution);
    // This boundary court deliberately abandons the still-pending transition;
    // it does not report successful close or completed migration.
    drop(source);
    drop(refused);
}

#[test]
fn edited_transition_plan_refuses_before_starting_the_original_intent() {
    let directory = tempfile::tempdir().unwrap();
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let fixture = super::transition_opening::open_transition(directory.path(), &execution);
    let source = fixture
        .home
        .world()
        .observation_port()
        .observe_root_product_branch()
        .unwrap();
    let mut home = fixture.home;
    for (index, digest) in [[7; 32], [8; 32]].into_iter().enumerate() {
        let reserved = home
            .reserve_home_program_transition_plan(
                source.clone(),
                HomeProgramTransitionPlanDigest::from_recorded(digest),
                3,
                super::transition_opening::target_meaning(),
                &execution,
            )
            .unwrap();
        let result = home.stage_home_program_transition_intent(reserved,
            worth_query_testkit_installation::facade::installation::application_home_binding::admitted_transition_successor_binding(), &execution);
        if index == 0 {
            result.unwrap();
        } else {
            assert!(matches!(
                result,
                Err(ApplicationHomeTransitionIntentDenial::TransitionPlanDrift)
            ));
        }
    }
    assert!(home
        .physical_serving_for_test()
        .ordered_journal(OrderedJournalName::new("home.transitions").unwrap())
        .unwrap()
        .head()
        .unwrap()
        .is_none());
    assert_eq!(
        &home
            .world()
            .observation_port()
            .observe_root_product_branch()
            .unwrap(),
        &source
    );
    assert_contended(directory.path(), &execution);
    drop(source);
    drop(home);
}
