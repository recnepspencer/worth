use std::sync::Arc;

use crate::branch::reference_test_fixture::{self, RealReferenceFixture};
use crate::branch::{ProductBranchObservation, RuntimeWorldBootstrapOutcome};
use crate::lifecycle::RuntimeWorldCloseDenial;
use crate::publication::RuntimeWorldCancellationSource;
use crate::publication::{CompositePublicationIntent, NoEffectCause};

type TestOwner = super::super::RuntimeWorldOwnerRoot<(), (), (), (), ()>;

fn setup() -> (
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
        RuntimeWorldBootstrapOutcome::Performed(performed) => performed,
        RuntimeWorldBootstrapOutcome::NoEffect(no_effect) => {
            panic!("bootstrap unexpectedly denied: {:?}", no_effect.cause())
        }
    };
    (fixture, owner, performed.product_branch().clone())
}

/// One Signal-only publication reservation off the given head. Preparation
/// and reservation are a single owner step, so a racing caller cannot hold a
/// lowered plan without the capacity that authorizes it.
fn prepare(
    owner: &TestOwner,
    expected: ProductBranchObservation,
) -> Result<
    crate::publication::PreparedCompositePublicationWithSignal,
    crate::publication::NoEffectCompositePublication,
> {
    let cancellation = RuntimeWorldCancellationSource::new();
    crate::lifecycle::RuntimeWorldPreparationService::prepare_publication(
        owner,
        expected,
        CompositePublicationIntent::with_signal(None),
        &cancellation.token(),
        None,
    )
}

#[test]
fn publication_attempts_own_independent_phase_state() {
    let (_fixture, owner, expected) = setup();
    let first = prepare(&owner, expected.clone()).expect("first attempt reserves");
    let second =
        prepare(&owner, expected).expect("second attempt is not serialized behind the first");

    assert_eq!(owner.state.operation.active(), 2);
    drop(first);
    assert_eq!(owner.state.operation.active(), 1);
    drop(second);
    assert_eq!(owner.state.operation.active(), 0);
}

#[test]
fn admitted_attempt_refuses_close_and_closed_owner_refuses_a_new_attempt() {
    let (_fixture, owner, expected) = setup();
    let policy = crate::test_execution_policy::serial_request();
    let request = worth_execution::ExecutionRequest::serial(&policy);
    let attempt = prepare(&owner, expected.clone()).unwrap();
    assert_eq!(
        owner.close(&request).unwrap_err(),
        RuntimeWorldCloseDenial::InFlightCriticalSection
    );
    drop(attempt);
    drop(owner.close(&request).unwrap());
    assert_eq!(
        prepare(&owner, expected).unwrap_err().cause(),
        NoEffectCause::OwnerUnavailable
    );
}
