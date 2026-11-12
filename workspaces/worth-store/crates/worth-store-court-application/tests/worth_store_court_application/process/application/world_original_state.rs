//! Actual acknowledged opens and publication are distinct protected facts.
use worth_execution::ExecutionRequest;
use worth_foundational::facade::{DurableCompositePublicationAttemptIdentity, WorldOriginalFate};
use worth_runtime_world::facade::*;

#[test]
fn protected_original_state_distinguishes_acknowledgement_from_publication_after_graceful_reopen() {
    let directory = tempfile::tempdir().unwrap();
    let foreign_directory = tempfile::tempdir().unwrap();
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let fixture = super::world_occurrence_readmission::open(directory.path(), &execution);
    let bootstrap = fixture.bootstrap_root(&execution);
    let expected = bootstrap.product_branch().clone();
    drop(bootstrap);
    let predecessor = expected.selected_commit().ordinal();
    let branch = expected.branch_identity().name().as_str().to_owned();
    let cancellation = RuntimeWorldCancellationSource::new();
    let prepared = fixture.prepare_publication(
        &execution,
        expected,
        crate::batches::create(0),
        &cancellation.token(),
    );
    let attempt = DurableCompositePublicationAttemptIdentity::from_recorded(
        fixture.home.world().owner_identity().durable().unwrap(),
        prepared.attempt_identity().ordinal(),
    );
    let reserved = prepared.reserved_commit_identity().ordinal();
    let source = fixture
        .home
        .world()
        .inspection_port()
        .observe_program_activation_history(&execution)
        .unwrap();
    assert_eq!(source.original_states().count(), 1);
    let original = source.original_state(attempt).unwrap();
    assert_eq!(original.fate(), WorldOriginalFate::Open);
    assert_eq!(original.attempt(), attempt);
    assert_eq!(original.reserved_commit().ordinal(), reserved);
    assert_eq!(original.predecessor().ordinal(), predecessor);
    assert_eq!(original.association().branch().name(), branch.as_str());
    let revision = original.program_revision();
    drop(source);
    let performed = fixture.publish(&execution, prepared, &cancellation.token());
    assert_eq!(performed.attempt_identity().ordinal(), attempt.ordinal());
    assert_eq!(performed.commit().identity().ordinal(), reserved);
    drop(performed.consume());
    let source = fixture
        .home
        .world()
        .inspection_port()
        .observe_program_activation_history(&execution)
        .unwrap();
    assert_eq!(
        source.original_state(attempt).unwrap().fate(),
        WorldOriginalFate::Published
    );
    drop(source);
    super::world_occurrence_readmission::close(fixture.home, &execution);

    let fixture = super::world_occurrence_readmission::open(directory.path(), &execution);
    let source = fixture
        .home
        .world()
        .inspection_port()
        .observe_program_activation_history(&execution)
        .unwrap();
    assert_eq!(source.original_states().count(), 1);
    let original = source.original_state(attempt).unwrap();
    assert_eq!(original.fate(), WorldOriginalFate::Published);
    assert_eq!(original.program_revision(), revision);
    assert_eq!(original.reserved_commit().ordinal(), reserved);
    assert_eq!(original.predecessor().ordinal(), predecessor);
    let foreign = super::world_occurrence_readmission::open(foreign_directory.path(), &execution);
    let root = foreign.bootstrap_root(&execution);
    let expected = root.product_branch().clone();
    drop(root);
    let prepared = foreign.prepare_publication(
        &execution,
        expected,
        crate::batches::create(0),
        &cancellation.token(),
    );
    let foreign_attempt = DurableCompositePublicationAttemptIdentity::from_recorded(
        foreign.home.world().owner_identity().durable().unwrap(),
        prepared.attempt_identity().ordinal(),
    );
    assert_eq!(foreign_attempt.ordinal(), attempt.ordinal());
    assert!(source.original_state(foreign_attempt).is_none());
    let performed = foreign.publish(&execution, prepared, &cancellation.token());
    drop(performed.consume());
    drop(source);
    super::world_occurrence_readmission::close(fixture.home, &execution);
    super::world_occurrence_readmission::close(foreign.home, &execution);
}

/// Acknowledgement without an owner effect keeps the admitted no-effect route.
#[test]
fn released_no_effect_original_stays_released_after_graceful_close_and_reopen() {
    let directory = tempfile::tempdir().unwrap();
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let fixture = super::world_occurrence_readmission::open(directory.path(), &execution);
    let bootstrap = fixture.bootstrap_root(&execution);
    let expected = bootstrap.product_branch().clone();
    let predecessor = expected.selected_commit().ordinal();
    drop(bootstrap);
    let cancellation = RuntimeWorldCancellationSource::new();
    let prepared = fixture.prepare_publication(
        &execution,
        expected,
        crate::batches::create(0),
        &cancellation.token(),
    );
    let attempt = DurableCompositePublicationAttemptIdentity::from_recorded(
        fixture.home.world().owner_identity().durable().unwrap(),
        prepared.attempt_identity().ordinal(),
    );
    // No publication was requested. Dropping the real preparation aborts the
    // Native candidate; the acknowledged World Open remains for its receiver.
    drop(prepared);
    drop(fixture);
    let fixture = super::world_occurrence_readmission::open(directory.path(), &execution);
    let page = fixture
        .home
        .world()
        .inspection_port()
        .recovery_page(None, std::num::NonZeroUsize::MIN)
        .unwrap();
    let [row] = page.rows() else {
        panic!("one actual acknowledged Open");
    };
    let recovery = fixture.home.world().recovery_port();
    let claim = recovery.inspect_recovered_effects(row.handle()).unwrap();
    assert!(claim.native_receipt().is_none());
    let mut outcome = recovery.release_recovered_effects(&execution, claim);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    let released = loop {
        match outcome {
            WorldRecoveredReleaseOutcome::Released(released) => break released,
            WorldRecoveredReleaseOutcome::Pending(claim) => {
                assert!(std::time::Instant::now() < deadline);
                std::thread::yield_now();
                outcome = recovery.release_recovered_effects(&execution, claim);
            }
            other => panic!("the actual no-effect original must release: {other:?}"),
        }
    };
    assert!(released.original().native_receipt().is_none());
    assert_eq!(
        released.original().attempt_identity().ordinal(),
        attempt.ordinal()
    );
    drop((released, page));
    let source = fixture
        .home
        .world()
        .inspection_port()
        .observe_program_activation_history(&execution)
        .unwrap();
    assert_eq!(
        source.original_state(attempt).unwrap().fate(),
        WorldOriginalFate::Released
    );
    drop(source);
    super::world_occurrence_readmission::close(fixture.home, &execution);
    let fixture = super::world_occurrence_readmission::open(directory.path(), &execution);
    let source = fixture
        .home
        .world()
        .inspection_port()
        .observe_program_activation_history(&execution)
        .unwrap();
    assert_eq!(source.original_states().count(), 1);
    assert_eq!(
        source.original_state(attempt).unwrap().fate(),
        WorldOriginalFate::Released
    );
    assert!(source.original_observations().next().is_none());
    drop(source);
    let current = fixture
        .home
        .world()
        .observation_port()
        .observe_root_product_branch()
        .unwrap();
    assert_eq!(current.selected_commit().ordinal(), predecessor);
    assert!(fixture
        .home
        .world()
        .inspection_port()
        .recovery_page(None, std::num::NonZeroUsize::MIN)
        .unwrap()
        .rows()
        .is_empty());
    let prepared = fixture.prepare_publication(
        &execution,
        current,
        crate::batches::create(1),
        &cancellation.token(),
    );
    let mut performed = fixture
        .publish(&execution, prepared, &cancellation.token())
        .consume();
    let successor = performed.take_successor_observation().unwrap();
    assert_eq!(successor.reference_generation().get(), 1);
    let selected = successor.selected_commit().ordinal();
    assert!(
        performed
            .component_results()
            .relational_settlement()
            .unwrap()
            .parents
            .is_empty(),
        "released no-effect preparation leaves the actual Empty predecessor"
    );
    drop((successor, performed));
    super::world_occurrence_readmission::close(fixture.home, &execution);
    let fixture = super::world_occurrence_readmission::open(directory.path(), &execution);
    let source = fixture
        .home
        .world()
        .inspection_port()
        .observe_program_activation_history(&execution)
        .unwrap();
    assert_eq!(
        source.original_state(attempt).unwrap().fate(),
        WorldOriginalFate::Released
    );
    assert_eq!(source.original_states().count(), 2);
    assert_eq!(source.original_observations().count(), 1);
    drop(source);
    let current = fixture
        .home
        .world()
        .observation_port()
        .observe_root_product_branch()
        .unwrap();
    assert_eq!(current.selected_commit().ordinal(), selected);
    assert_eq!(current.reference_generation().get(), 1);
    drop(current);
    super::world_occurrence_readmission::close(fixture.home, &execution);
}
