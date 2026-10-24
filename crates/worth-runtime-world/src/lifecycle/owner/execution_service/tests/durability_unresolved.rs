//! A Relational publication whose durable fate is unknown may already be
//! durable, so World retains it as an owner effect instead of reporting
//! no-effect, and offers only release, inspection and owner closure.
use std::sync::atomic::{AtomicBool, Ordering};

use worth_relational::facade::durability::{
    RelationalCanonicalCommitArtifact, RelationalDurableMedium, RelationalDurablePublication,
};
use worth_relational::facade::runtime::RelationalRuntimeApi;

use super::*;

/// A durable medium that commits until it is told to leave the next
/// publication's fate unknown, as a crash after the medium's effect would.
#[derive(Debug, Default)]
struct UnresolvingMedium {
    unresolve_next: AtomicBool,
}

impl RelationalDurableMedium for UnresolvingMedium {
    fn publish(
        &self,
        _artifact: &RelationalCanonicalCommitArtifact,
    ) -> RelationalDurablePublication {
        if self.unresolve_next.swap(false, Ordering::AcqRel) {
            return RelationalDurablePublication::Indeterminate {
                detail: "the medium lost its answer after the effect".to_owned(),
            };
        }
        RelationalDurablePublication::Committed
    }
}

#[test]
fn unresolved_relational_durability_retains_a_possible_effect() {
    let medium = Arc::new(UnresolvingMedium::default());
    let relational = RelationalRuntimeApi::builder()
        .durable_medium(medium.clone())
        .build();
    let mut fixture =
        reference_test_fixture::real_fixture_with_relational_runtime(relational, 12, 12);
    let owner = Arc::new(
        TestOwner::new(
            fixture.owner_inputs(budgets(4), RuntimeWorldClock::from_source(FixedClock)),
        )
        .expect("managed owner construction"),
    );
    let expected = match owner.bootstrap_root(fixture.bootstrap_intent()) {
        RuntimeWorldBootstrapOutcome::Performed(performed) => performed.product_branch().clone(),
        RuntimeWorldBootstrapOutcome::NoEffect(no_effect) => {
            panic!("bootstrap unexpectedly denied: {:?}", no_effect.cause())
        }
    };
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::from_policy(&owner.state.execution.request_policy()),
        worth_execution::CancellationToken::new(),
        None,
    );
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);
    let prepared = prepare_relational(&execution, &fixture, &owner, &expected, "unresolved");
    medium.unresolve_next.store(true, Ordering::Release);

    let retained = match execute_without_signal(execution, &owner, prepared) {
        OwnerExecutionOutcome::ProductUnpublished(retained) => retained,
        other => panic!("an unresolved durable fate is never no-effect: {other:?}"),
    };

    assert_eq!(
        retained.cause(),
        ProductUnpublishedCause::RelationalDurabilityUnresolved
    );
    assert_eq!(retained.owner_effect_count(), 1);
    assert_eq!(
        retained.progress().relational_posture(),
        RelationalAttemptProgressPosture::DurabilityUnresolved
    );
    assert_eq!(
        retained.progress().signal_posture(),
        SignalAttemptProgressPosture::Untouched
    );
    assert!(!retained.progress().relational_requires_settlement());
    assert_eq!(
        retained.next_actions(),
        [
            ProductUnpublishedNextAction::ReleaseObligations,
            ProductUnpublishedNextAction::Inspect,
            ProductUnpublishedNextAction::CloseOwner,
        ],
        "nothing is settled or settleable here; reopening the owner resolves the fate"
    );
    assert_eq!(
        RuntimeWorldObservationService::observe_product_branch(
            owner.as_ref(),
            &expected.branch_identity().clone(),
        )
        .expect("the product branch stays observable")
        .snapshot(),
        expected.snapshot(),
        "the product head does not move for an unresolved owner effect"
    );
}
