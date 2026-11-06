//! The actual committed intent governs the next Home incarnation.
use super::*;
use worth_foundational::facade::{ApplicationBuildIdentity, HomeProgramTransitionPlanDigest};
use worth_query_testkit_installation::facade::installation::application_home_binding as bindings;
use worth_store_application::{
    ApplicationHomeOpenRefusal as Open, ApplicationHomeTransitionIntentDenial as Intent,
    ApplicationHomeTransitionIntentOutcome as Outcome,
    ApplicationHomeTransitionPlanningDenial as Plan,
};

#[test]
fn a_pending_transition_refuses_a_different_successor_before_predecessor_admission() {
    let directory = tempfile::tempdir().unwrap();
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let (incarnation, _) = commit_original_intent(directory.path(), &execution);
    for invalid_predecessor in [false, true] {
        let mut requested = worth_store_application::ApplicationHomeRequest::new(
            directory.path().into(),
            native(),
            bindings::admitted_other_successor_binding(),
            build(),
        );
        if invalid_predecessor {
            requested = requested.with_program_transition_predecessor(
                native(),
                bindings::admitted_transition_successor_binding(),
            );
        }
        let recovered = worth_store_application::recover_home(requested, &execution).unwrap();
        assert!(matches!(
            worth_store_application::admit_home(recovered, &execution),
            Err(Open::Transition(Intent::TransitionPending))
        ));
    }
    let resumed = worth_store_testkit_application_oracle::world_home::open(
        super::transition_opening::successor_request(directory.path()),
        &execution,
    )
    .unwrap();
    assert_eq!(
        resumed.home.incarnation().get(),
        incarnation + 1,
        "both refused opens leave the incarnation unchanged"
    );
    let source = resumed
        .home
        .world()
        .observation_port()
        .observe_root_product_branch()
        .unwrap();
    let denied = resumed
        .home
        .reserve_home_program_transition_plan(
            source.clone(),
            HomeProgramTransitionPlanDigest::from_recorded([7; 32]),
            3,
            super::transition_opening::target_meaning(),
            &crate::batches::transition_body_roster(&execution, 3),
            &execution,
        )
        .err()
        .unwrap();
    assert!(matches!(
        denied,
        Plan::Intent(Intent::TransitionResumeRequired)
    ));
    assert_eq!(
        resumed
            .home
            .world()
            .observation_port()
            .observe_root_product_branch()
            .unwrap(),
        source
    );
    assert_eq!(
        resumed
            .home
            .physical_serving_for_test()
            .ordered_journal(OrderedJournalName::new("home.transitions").unwrap())
            .unwrap()
            .head()
            .unwrap()
            .unwrap()
            .position(),
        1
    );
    // This boundary deliberately has an unfinished original transition. It is
    // abandoned; it does not pretend that fresh reservation resumed its steps.
    drop(resumed);
}

#[test]
fn reopened_world_issuance_cannot_reuse_intent_only_attempt_assignments() {
    use worth_runtime_world::facade::{RuntimeWorldCancellationSource, WorldOpenAttemptOutcome};
    let directory = tempfile::tempdir().unwrap();
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let (_, last_reserved) = commit_original_intent(directory.path(), &execution);
    let resumed = worth_store_testkit_application_oracle::world_home::open(
        super::transition_opening::successor_request(directory.path()),
        &execution,
    )
    .unwrap();
    let source = resumed
        .home
        .world()
        .observation_port()
        .observe_root_product_branch()
        .unwrap();
    let cancellation = RuntimeWorldCancellationSource::new();
    let port = resumed.home.world().publication_port();
    let mut outcome = port.open_publication_without_signal(
        &execution,
        source.clone(),
        worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
        &cancellation.token(),
        None,
    );
    let opened = loop {
        execution.consult().unwrap();
        match outcome {
            WorldOpenAttemptOutcome::Opened(opened) => break opened,
            WorldOpenAttemptOutcome::Pending(pending) => {
                std::thread::yield_now();
                outcome = pending.advance(&execution);
            }
            other => panic!("actual World issuance must remain usable: {other:?}"),
        }
    };
    assert!(opened.attempt_identity().ordinal() > last_reserved);
    assert_eq!(
        resumed
            .home
            .world()
            .observation_port()
            .observe_root_product_branch()
            .unwrap(),
        source
    );
    // No Native effect or product movement occurred. Retain the real opened
    // custody, then abandon it and the still-pending Home without claiming close.
    drop(opened.retain());
    drop(resumed);
}

fn commit_original_intent(
    location: &std::path::Path,
    execution: &ExecutionRequest<'_, '_>,
) -> (u64, u64) {
    let mut fixture = super::transition_opening::open_transition(location, execution);
    let incarnation = fixture.home.incarnation().get();
    let source = fixture
        .home
        .world()
        .observation_port()
        .observe_root_product_branch()
        .unwrap();
    let reserved = fixture
        .home
        .reserve_home_program_transition_plan(
            source,
            HomeProgramTransitionPlanDigest::from_recorded([7; 32]),
            3,
            super::transition_opening::target_meaning(),
            &crate::batches::transition_body_roster(execution, 3),
            execution,
        )
        .unwrap();
    let last = reserved.assignments().last().unwrap().attempt().ordinal();
    fixture
        .home
        .stage_home_program_transition_intent(
            reserved,
            bindings::admitted_transition_successor_binding(),
            execution,
        )
        .unwrap();
    loop {
        execution.consult().unwrap();
        match fixture
            .home
            .advance_home_program_transition_intent(execution)
            .unwrap()
        {
            Outcome::Pending => std::thread::yield_now(),
            Outcome::Committed => break,
            other => panic!("the real original intent must commit: {other:?}"),
        }
    }
    drop(fixture);
    (incarnation, last)
}

fn native() -> worth_relational::facade::runtime::RelationalRuntimeDefinition {
    crate::workload::runtime_builder(crate::schema_transition::SOURCE).resolve()
}
fn build() -> ApplicationBuildIdentity {
    ApplicationBuildIdentity::parse("11111111-1111-4111-8111-111111111111").unwrap()
}
