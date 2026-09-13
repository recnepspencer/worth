use std::sync::Arc;

use worth_relational::facade::mvcc::RelationalTransactionIntent;

use crate::branch::reference_test_fixture::{self, RealReferenceFixture};
use crate::branch::ProductBranchObservation;
#[cfg(feature = "test-operation-control")]
use crate::branch::ProductBranchReferenceSnapshot;
use crate::lifecycle::{
    RuntimeWorldObservationService, RuntimeWorldOwnerExecutionService,
    RuntimeWorldPreparationService, RuntimeWorldProductPublicationService,
};
#[cfg(feature = "test-operation-control")]
use crate::publication::{
    CompositeAttemptProgress, RelationalAttemptProgress, SignalAttemptProgress,
};
use crate::publication::{
    CompositeLateCancellationPosture, CompositePublicationIntent, OwnerExecutionOutcome,
    PreparedCompositePublicationWithoutSignal, RuntimeWorldPublicationOutcome,
};
#[cfg(feature = "test-operation-control")]
use crate::recovery::{ProductUnpublishedCause, ProductUnpublishedRetentionPosture};

pub(super) type TestOwner = super::super::RuntimeWorldOwnerRoot<(), (), (), (), ()>;

pub(super) fn setup() -> (
    RealReferenceFixture,
    Arc<TestOwner>,
    ProductBranchObservation,
) {
    let mut fixture = reference_test_fixture::real_fixture(8, 8);
    let owner = Arc::new(
        TestOwner::new(fixture.owner_inputs(
            super::bootstrap_budgets(),
            crate::lifecycle::RuntimeWorldClock::from_source(super::FixedClock),
        ))
        .expect("managed owner construction"),
    );
    let performed = match owner.bootstrap_root(fixture.bootstrap_intent()) {
        crate::branch::RuntimeWorldBootstrapOutcome::Performed(performed) => performed,
        crate::branch::RuntimeWorldBootstrapOutcome::NoEffect(no_effect) => {
            panic!("bootstrap unexpectedly denied: {:?}", no_effect.cause())
        }
    };
    (fixture, owner, performed.product_branch().clone())
}

pub(super) fn setup_with_relational_source() -> (
    RealReferenceFixture,
    Arc<TestOwner>,
    ProductBranchObservation,
) {
    let (fixture, owner, expected) = setup();
    let ready = ready_relational_publication(&fixture, &owner, expected.clone());
    let cell = owner.state.branches.root_cell().expect("bootstrapped cell");
    match RuntimeWorldProductPublicationService::publish(
        owner.as_ref(),
        ready,
        &cell,
        CompositeLateCancellationPosture::NotRequested,
    ) {
        RuntimeWorldPublicationOutcome::Performed(_) => {}
        other => panic!("the canonical seed publishes its product head: {other:?}"),
    }
    let expected = RuntimeWorldObservationService::observe_product_branch(
        owner.as_ref(),
        &expected.branch_identity().clone(),
    )
    .expect("the owner re-observes the published source basis");
    (fixture, owner, expected)
}

pub(super) fn prepare_relational(
    fixture: &RealReferenceFixture,
    owner: &TestOwner,
    expected: ProductBranchObservation,
    candidate_name: &str,
) -> PreparedCompositePublicationWithoutSignal {
    RuntimeWorldPreparationService::prepare_publication(
        owner,
        expected,
        CompositePublicationIntent::without_signal(RelationalTransactionIntent::ordinary())
            .with_prepared_relational_candidate(
                fixture.prepare_relational_owner_candidate(candidate_name),
            ),
        &crate::publication::RuntimeWorldCancellationSource::new().token(),
        None,
    )
    .expect("the current product head admits Relational preparation")
}

pub(super) fn ready_relational_publication(
    fixture: &RealReferenceFixture,
    owner: &TestOwner,
    expected: ProductBranchObservation,
) -> crate::publication::CompositePublicationReady {
    ready_from_prepared(
        owner,
        prepare_relational(fixture, owner, expected, "publication"),
        "the production owner execution must settle",
    )
}

pub(super) fn ready_from_prepared(
    owner: &TestOwner,
    prepared: PreparedCompositePublicationWithoutSignal,
    context: &str,
) -> crate::publication::CompositePublicationReady {
    let cancellation = crate::publication::RuntimeWorldCancellationSource::new();
    let outcome = RuntimeWorldOwnerExecutionService::execute_without_signal(
        owner,
        prepared,
        &cancellation.token(),
    );
    let settlement = match outcome {
        OwnerExecutionOutcome::Settled(settlement) => settlement,
        other => panic!("{context}: {other:?}"),
    };
    let successor = settlement
        .successor_basis()
        .cloned()
        .expect("owner execution returns its successor basis");
    settlement
        .ready(successor)
        .expect("real successor retention binds after owner execution")
}

#[test]
fn complete_ready_publish_path_derives_and_installs_the_successor_snapshot() {
    let (fixture, owner, expected) = setup();
    let cell = owner.state.branches.root_cell().expect("bootstrapped cell");
    let ready = ready_relational_publication(&fixture, &owner, expected.clone());
    assert_eq!(owner.state.operation.active(), 1);

    let outcome = ready.publish(&cell, CompositeLateCancellationPosture::NotRequested);
    let performed = match outcome {
        RuntimeWorldPublicationOutcome::Performed(performed) => performed,
        other => panic!("the uncontended ready publication must perform: {other:?}"),
    };

    assert_eq!(performed.old_product_head(), expected.snapshot());
    assert_eq!(performed.new_product_head(), &cell.atomic_snapshot());
    assert_eq!(
        performed.new_product_head().selected_commit(),
        performed.commit().identity()
    );
    assert_eq!(owner.state.operation.active(), 0);
}

/// One resolved post-effect retention denial: the record it installed is in the
/// `ReacquisitionPending` posture, holding a reserved component pin pair instead
/// of issued pins.
#[cfg(feature = "test-operation-control")]
pub(super) struct PendingRetentionDenial {
    /// Keeps the component fixture carrying the injected Signal retention panic
    /// alive for as long as the owner under proof.
    _fixture: RealReferenceFixture,
    pub(super) owner: Arc<TestOwner>,
    pub(super) expected: ProductBranchObservation,
    pub(super) retained: crate::recovery::ProductUnpublishedOwnerEffects,
}

/// Drive one publication past its component effect and deny its post-effect
/// retention reacquisition, which is the only way to reach the pending posture.
#[cfg(feature = "test-operation-control")]
pub(super) fn resolve_post_effect_retention_denial() -> PendingRetentionDenial {
    let (mut fixture, owner, expected) = setup();
    let cancellation = crate::publication::RuntimeWorldCancellationSource::new();
    let mut attempt = RuntimeWorldPreparationService::prepare_publication(
        owner.as_ref(),
        expected.clone(),
        CompositePublicationIntent::with_signal(None),
        &cancellation.token(),
        None,
    )
    .expect("the owner reserves complete publication capacity")
    .into_attempt();
    attempt.begin_owner_execution();

    let advanced = fixture.perform_signal_owner_change();
    let successor_signal = advanced.advanced_basis().clone();
    let successor_basis = crate::basis::admit_current(
        &owner
            .state
            .identities
            .lock()
            .unwrap_or_else(|error| error.into_inner()),
        &owner.state.relational.basis_port(),
        &owner.state.signal.basis_port(),
        &owner.state.bridge,
        expected.basis().relational_basis().clone(),
        successor_signal,
        expected.basis().correspondence_basis().clone(),
    )
    .expect("the component owners admit the exact Signal successor tuple");
    fixture.inject_signal_retention_panic();
    let progress = CompositeAttemptProgress::new(
        RelationalAttemptProgress::untouched(),
        SignalAttemptProgress::advanced(advanced),
    );

    let retained = attempt
        .settle(progress)
        .ready(successor_basis)
        .expect_err("the injected post-effect retention denial becomes recovery");
    PendingRetentionDenial {
        _fixture: fixture,
        owner,
        expected,
        retained,
    }
}

#[cfg(feature = "test-operation-control")]
#[test]
fn post_effect_retention_denial_installs_recovery_and_preserves_retry_capacity() {
    let PendingRetentionDenial {
        _fixture,
        owner,
        expected,
        retained,
    } = resolve_post_effect_retention_denial();

    assert_eq!(
        retained.cause(),
        ProductUnpublishedCause::RetentionAdmissionDenied
    );
    assert!(!retained
        .next_actions()
        .contains(&crate::recovery::ProductUnpublishedNextAction::CloseOwner));
    assert_eq!(
        retained.retention_posture(),
        ProductUnpublishedRetentionPosture::ReacquisitionPending
    );
    assert_eq!(retained.owner_effect_count(), 1);
    assert_eq!(
        retained.live_obligation_count(),
        4,
        "the reserved pin pair, the recovery slot, and the reserved successor history"
    );
    assert_eq!(
        retained.component_results().signal_posture(),
        crate::history::CompositeComponentChangePosture::Published
    );
    assert_eq!(owner.state.history.len(), 1);
    assert_eq!(owner.state.history.reserved_len(), 1);
    assert!(
        retained.successor_commit().is_none(),
        "a denied pin pair cannot install unprotected history"
    );
    assert!(
        retained.successor_basis().is_some(),
        "performed owner evidence stays in recovery custody"
    );
    assert_eq!(&cell_snapshot(&owner), expected.snapshot());
    let recovery_handle = retained.recovery_handle();
    assert_eq!(owner.state.recovery.reserved_slots(), 0);
    assert_eq!(owner.recovery_record_count(), 1);
    assert_retry_capacity(&owner, 2);
    assert_eq!(owner.state.operation.active(), 0);
    drop(retained);
    assert_eq!(owner.recovery_record_count(), 1);
    assert_retry_capacity(&owner, 2);
    assert!(owner.cleanup_recovery_handle(&recovery_handle).is_some());
    assert_eq!(owner.recovery_record_count(), 0);
    assert_eq!(owner.state.recovery.reserved_slots(), 0);
    assert_retry_capacity(&owner, 0);
}

/// The reacquisition capacity a pending record keeps reserved for its retry:
/// one exact pin pair and one in-flight acquisition pair.
#[cfg(feature = "test-operation-control")]
#[track_caller]
fn assert_retry_capacity(owner: &TestOwner, expected: usize) {
    assert_eq!(
        owner.state.retention.reserved_unique_pin_capacity(),
        expected
    );
    assert_eq!(
        owner
            .state
            .retention
            .reserved_in_flight_acquisition_capacity(),
        expected
    );
}
#[cfg(feature = "test-operation-control")]
fn cell_snapshot(owner: &TestOwner) -> ProductBranchReferenceSnapshot {
    owner
        .state
        .branches
        .root_cell()
        .expect("bootstrapped cell")
        .atomic_snapshot()
}
