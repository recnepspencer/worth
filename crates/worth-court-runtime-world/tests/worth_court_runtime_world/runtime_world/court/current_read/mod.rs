//! Current control and candidate truth remain distinct owner-issued inputs.
use crate::court::ordinary_context::CompositeSupplyChainCourt;
use worth_runtime_world::facade::*;

mod fixture;
mod guarded_cutover;
mod memory_handoff;
mod recovery;
mod refusals;
use fixture::*;

#[test]
fn current_control_can_be_the_write_target_and_its_own_effect_discharges_the_condition() {
    let request = crate::test_execution_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&request);
    let court = CompositeSupplyChainCourt::compile();
    let current = court.bootstrap();
    let prepared = prepare(&court, &execution, &current, &current);
    let RuntimeWorldPublicationOutcome::Performed(done) =
        court.world.publication_port().execute_without_signal(
            execution,
            prepared,
            &RuntimeWorldCancellationSource::new().token(),
        )
    else {
        panic!("same-cell cutover releases its read guard before the final product write");
    };
    let selected = court.observe(&current);
    assert_ne!(selected, current);
    assert_eq!(grain(&court, &execution, &selected), "5");
    assert_eq!(
        selected.basis().relational_basis(),
        &direct_basis(&court, &current)
    );
    drop(done.consume());
}

#[test]
fn moved_control_world_head_refuses_without_touching_the_candidate_and_fresh_control_works() {
    let request = crate::test_execution_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&request);
    let court = CompositeSupplyChainCourt::compile();
    let current = court.bootstrap();
    let target = fork(&court, &execution, &current);
    let prepared = prepare(&court, &execution, &target, &current);
    let next_current = move_signal(&court, &execution, &current);
    assert_ne!(next_current, current);
    assert_eq!(
        next_current.basis().relational_basis(),
        current.basis().relational_basis()
    );
    let before = direct_basis(&court, &target);
    let RuntimeWorldPublicationOutcome::NoEffect(no) =
        court.world.publication_port().execute_without_signal(
            execution,
            prepared,
            &RuntimeWorldCancellationSource::new().token(),
        )
    else {
        panic!("old current World image must refuse");
    };
    assert_eq!(
        no.cause(),
        NoEffectCause::CurrentReadDependency(
            CurrentProductReadDependencyDenial::WorldHeadUnavailable
        )
    );
    assert_eq!(court.observe(&target), target);
    assert_eq!(direct_basis(&court, &target), before);
    let fresh = prepare(&court, &execution, &target, &next_current);
    let RuntimeWorldPublicationOutcome::Performed(done) =
        court.world.publication_port().execute_without_signal(
            execution,
            fresh,
            &RuntimeWorldCancellationSource::new().token(),
        )
    else {
        panic!("same target and write with the genuine current source must publish");
    };
    assert_eq!(grain(&court, &execution, &court.observe(&target)), "5");
    assert_eq!(grain(&court, &execution, &court.observe(&current)), "4");
    drop(done.consume());
}

#[test]
fn direct_control_native_movement_refuses_even_when_the_world_image_is_unchanged() {
    let request = crate::test_execution_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&request);
    let court = CompositeSupplyChainCourt::compile();
    let current = court.bootstrap();
    let target = fork(&court, &execution, &current);
    let prepared = prepare(&court, &execution, &target, &current);
    let candidate =
        court
            .records
            .candidate(&execution, current.basis().relational_basis(), "grain", "9");
    let worth_relational::facade::mvcc::RelationalPublicationOutcome::Performed(performed) = court
        .records
        .runtime
        .publication_port()
        .compare_and_publish(&execution, candidate)
    else {
        panic!("real Native control move");
    };
    let settled = court
        .records
        .runtime
        .settle_performed_publication(performed)
        .unwrap();
    assert_eq!(court.observe(&current), current);
    let before = direct_basis(&court, &target);
    let RuntimeWorldPublicationOutcome::NoEffect(no) =
        court.world.publication_port().execute_without_signal(
            execution,
            prepared,
            &RuntimeWorldCancellationSource::new().token(),
        )
    else {
        panic!("World equality cannot replace Native currentness");
    };
    assert_eq!(
        no.cause(),
        NoEffectCause::CurrentReadDependency(CurrentProductReadDependencyDenial::Native(
            worth_relational::facade::mvcc::RelationalPublicationReadDependencyDenial::NotCurrent,
        ))
    );
    assert_eq!(direct_basis(&court, &target), before);
    assert_eq!(court.observe(&target), target);
    drop(settled);
}

#[test]
fn combined_publication_writes_only_the_candidate_and_releases_control_before_signal() {
    let request = crate::test_execution_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&request);
    let court = CompositeSupplyChainCourt::compile();
    let current = court.bootstrap();
    let target = fork(&court, &execution, &current);
    let candidate =
        court
            .records
            .candidate(&execution, target.basis().relational_basis(), "grain", "5");
    let token = RuntimeWorldCancellationSource::new().token();
    let prepared = court
        .world
        .publication_port()
        .prepare_with_signal(
            target.clone(),
            intent(
                CompositePublicationIntent::with_signal(Some(
                    RelationalTransactionIntent::ordinary(),
                ))
                .with_prepared_relational_candidate(candidate),
                &current,
            ),
            &token,
            None,
        )
        .unwrap();
    let mut signal_entered = false;
    let RuntimeWorldPublicationOutcome::Performed(done) =
        court.world.publication_port().execute_with_signal(
            execution,
            prepared,
            &mut court.context(&execution, &target),
            &token,
            |_| {
                signal_entered = true;
                assert!(!court
                    .world
                    .observation_port()
                    .product_branch_writers_are_locked_out_for_test(&current)
                    .unwrap());
                Ok(())
            },
        )
    else {
        panic!("genuine Relational and Signal effects publish");
    };
    assert!(signal_entered);
    let selected = court.observe(&target);
    assert_ne!(
        selected.basis().signal_basis().admission_identity(),
        target.basis().signal_basis().admission_identity()
    );
    assert_eq!(grain(&court, &execution, &selected), "5");
    assert_eq!(court.observe(&current), current);
    assert_eq!(grain(&court, &execution, &current), "4");
    drop(done.consume());
}
