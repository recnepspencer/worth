//! Real Home reserves original attempts without publishing speculative effects.
use super::*;
use worth_foundational::facade::HomeProgramTransitionPlanDigest;
use worth_proof::{HomeProgramTransitionAdmissionDenial, HomeProgramTransitionOwner};
use worth_runtime_world::facade::WorldHomeTransitionReservationRefusal;
use worth_store::physical_runtime::{RELATIONAL_BRANCH_TRANSITIONS, RELATIONAL_CANONICAL_COMMITS};
use worth_store_application::ApplicationHomeTransitionPlanningDenial;

#[test]
fn transition_plan_reserves_original_attempts_without_publishing_future_heads_or_steps() {
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
    let home = fixture.home;
    let native_journals = [RELATIONAL_CANONICAL_COMMITS, RELATIONAL_BRANCH_TRANSITIONS];
    let original_native_heads = native_journals.map(|name| {
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
        .expect("the actual Home owner supplies its attached planning admission");
    assert_eq!(
        reserved.description().identity().home(),
        home.home_identity()
    );
    assert_eq!(reserved.description().identity().intent_position().get(), 1);
    assert_eq!(reserved.assignments().len(), 3);
    for (index, assignment) in reserved.assignments().iter().enumerate() {
        assert_eq!(
            assignment.attempt().owner(),
            home.world().owner_identity().durable().unwrap()
        );
        assert_eq!(
            assignment.branch(),
            reserved.description().predecessor().branch()
        );
        assert_eq!(
            assignment.incarnation(),
            reserved.description().predecessor().incarnation()
        );
        if index != 0 {
            assert_eq!(
                assignment.attempt().ordinal(),
                reserved.assignments()[index - 1].attempt().ordinal() + 1
            );
        }
    }
    let foreign =
        HomeProgramTransitionOwner::fresh(home.home_identity().bytes(), home.incarnation().fence());
    let refused = home
        .world()
        .publication_port()
        .reserve_home_transition_assignments(
            &execution,
            source.clone(),
            reserved.description().clone(),
            foreign.admit_plan(reserved.description().commitment()),
        )
        .unwrap_err();
    assert!(matches!(
        refused,
        WorldHomeTransitionReservationRefusal::Admission(
            HomeProgramTransitionAdmissionDenial::ForeignOwner
        )
    ));
    let selected = home
        .world()
        .observation_port()
        .observe_root_product_branch()
        .unwrap();
    assert_eq!(selected, source);
    for (index, name) in native_journals.into_iter().enumerate() {
        assert_eq!(
            home.physical_serving_for_test()
                .ordered_journal(OrderedJournalName::new(name).unwrap())
                .unwrap()
                .head()
                .unwrap(),
            original_native_heads[index],
            "planning must not publish a Native effect to {name}"
        );
    }
    for name in ["home.transitions", "world.custody", "world.occurrences"] {
        assert!(
            home.physical_serving_for_test()
                .ordered_journal(OrderedJournalName::new(name).unwrap())
                .unwrap()
                .head()
                .unwrap()
                .is_none(),
            "planning must not publish {name}"
        );
    }
    drop(selected);
    drop(reserved);
    drop(source);
    close(home, &execution);
}

#[test]
fn stopped_transition_planning_retains_the_source_without_issuing_attempt_assignments() {
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
    let home = fixture.home;
    let digest = HomeProgramTransitionPlanDigest::from_recorded([7; 32]);
    let first = home
        .reserve_home_program_transition_plan(
            source.clone(),
            digest,
            1,
            super::transition_opening::target_meaning(),
            &execution,
        )
        .expect("the actual issuer reserves the first assignment");
    let ordinal = first.assignments()[0].attempt().ordinal();
    drop(first);
    let cancellation = worth_execution::CancellationSource::new();
    let stopped = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(None),
        cancellation.token(),
        None,
    );
    cancellation.cancel();
    let refused = home
        .reserve_home_program_transition_plan(
            source.clone(),
            digest,
            8,
            super::transition_opening::target_meaning(),
            &ExecutionRequest::serial(&stopped),
        )
        .err()
        .expect("the stopped original request refuses planning");
    assert!(matches!(
        refused,
        ApplicationHomeTransitionPlanningDenial::Execution(_)
    ));
    assert_contended(directory.path(), &execution);
    let next = home
        .reserve_home_program_transition_plan(
            source.clone(),
            digest,
            1,
            super::transition_opening::target_meaning(),
            &execution,
        )
        .expect("the same actual owner remains usable");
    assert_eq!(
        next.assignments()[0].attempt().ordinal(),
        ordinal + 1,
        "a stopped plan must not issue its proposed eight assignments"
    );
    let current = home
        .world()
        .observation_port()
        .observe_root_product_branch()
        .unwrap();
    assert_eq!(&current, &source);
    drop(current);
    drop(next);
    drop(source);
    close(home, &execution);
}
