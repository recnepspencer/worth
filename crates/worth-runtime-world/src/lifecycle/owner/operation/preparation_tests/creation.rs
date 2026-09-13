use super::super::preparation_test_support::{
    advance_product_head, fork_both_intent, reservation_counts, setup, setup_with_custody,
    setup_with_fixture, TestOwner,
};
use crate::branch::{
    ProductBranchCreationIntent, ProductBranchObservation, RelationalBranchCreationPlan,
    RuntimeWorldBranchAdmissionDenial, SignalBranchCreationPlan,
};
use crate::lifecycle::RuntimeWorldPreparationService;
use crate::publication::{
    CompositePublicationOrder, NoEffectCause, ReservedBranchCreationAttempt,
    RuntimeWorldCancellationSource,
};

fn prepare(
    owner: &TestOwner,
    source: &ProductBranchObservation,
    intent: ProductBranchCreationIntent,
) -> Result<ReservedBranchCreationAttempt, RuntimeWorldBranchAdmissionDenial> {
    RuntimeWorldPreparationService::prepare_creation(
        owner,
        source.clone(),
        intent,
        &RuntimeWorldCancellationSource::new().token(),
        None,
    )
}

/// The head comparison in `admit_creation_source` is the creation lane's
/// stale-source gate: a source the reference cell has moved past is refused
/// before a single capacity is charged.
#[test]
fn stale_creation_source_is_denied_before_any_reservation() {
    let (_fixture, owner, source) = setup_with_fixture(2, 8);
    let advanced = advance_product_head(owner.as_ref(), &source);
    assert_ne!(advanced.selected_commit(), source.selected_commit());
    let before = reservation_counts(owner.as_ref());
    assert_eq!(owner.state.operation.active(), 0);

    let denied = prepare(owner.as_ref(), &source, fork_both_intent("stale-child"))
        .expect_err("a source the cell has moved past cannot be admitted");
    assert_eq!(denied, RuntimeWorldBranchAdmissionDenial::StaleSourceHead);
    assert_eq!(reservation_counts(owner.as_ref()), before);
    assert_eq!(owner.state.operation.active(), 0);

    let fresh = prepare(owner.as_ref(), &advanced, fork_both_intent("fresh-child"))
        .expect("the advanced head is admitted by the same gate");
    assert_eq!(fresh.source(), &advanced);
}

/// The creation twin of the publication preparation contract: the reserved
/// attempt carries the exact source it was admitted against and both owner
/// postures verbatim, and it is linear — dropping it returns every capacity.
#[test]
fn healthy_creation_preparation_is_exact_and_reservation_is_linear() {
    let (owner, source) = setup(2);
    let attempt = prepare(owner.as_ref(), &source, fork_both_intent("exact-child"))
        .expect("the current head admits a creation reservation");
    assert_eq!(attempt.source(), &source);
    assert_eq!(
        attempt.plan().relational(),
        &RelationalBranchCreationPlan::ForkExact {
            target: worth_relational::facade::history::BranchId(
                "exact-child-relational".to_owned()
            ),
        }
    );
    assert_eq!(
        attempt.plan().signal(),
        &SignalBranchCreationPlan::ForkExact {
            target: worth_signal::facade::branch::validate_signal_branch_name("exact-child")
                .expect("focused Signal branch name validates"),
        }
    );
    assert_eq!(
        attempt.order(),
        CompositePublicationOrder::RelationalThenSignal
    );
    assert_eq!(reservation_counts(owner.as_ref()), (1, 1, 2, 2, 1));
    drop(attempt);
    assert_eq!(reservation_counts(owner.as_ref()), (0, 0, 0, 0, 0));
    assert_eq!(owner.state.operation.active(), 0);
}

/// Every bounded resource a creation can consume — the commit identity and its
/// history slot, the recovery slot, the component pin pair, the publication
/// attempt slot, the operation ledger entry, and one custody slot per owner
/// fork — is charged by `prepare_creation` before the first owner effect, and
/// returns to baseline exactly once on `cancel()` and on drop.
#[test]
fn creation_reservation_holds_every_capacity_and_releases_once() {
    // Two custody slots is exactly one Fork/Fork creation, so a second
    // concurrent creation is the probe that says whether the first one's
    // slots are still charged.
    let (owner, source) = setup_with_custody(2, 2);
    let baseline = reservation_counts(owner.as_ref());
    assert_eq!(baseline, (0, 0, 0, 0, 0));

    let first = prepare(owner.as_ref(), &source, fork_both_intent("custody-first"))
        .expect("the current head admits a creation reservation");
    let first_identity = first.identity().clone();
    assert_eq!(reservation_counts(owner.as_ref()), (1, 1, 2, 2, 1));
    assert_eq!(owner.state.operation.active(), 1);

    let denied = prepare(owner.as_ref(), &source, fork_both_intent("custody-second"))
        .expect_err("the installed custody bound has no free slot for a second Fork/Fork cell");
    assert_eq!(
        denied,
        RuntimeWorldBranchAdmissionDenial::CustodyCapacityExhausted
    );
    assert_eq!(
        reservation_counts(owner.as_ref()),
        (1, 1, 2, 2, 1),
        "a denied creation releases every capacity it took on the way to the denial"
    );
    assert_eq!(owner.state.operation.active(), 1);

    let no_effect = first.cancel();
    assert_eq!(no_effect.cause(), NoEffectCause::CancelledBeforeEffect);
    assert_eq!(reservation_counts(owner.as_ref()), baseline);
    assert_eq!(owner.state.operation.active(), 0);
    drop(no_effect);

    // A second Fork/Fork cell now fits, which is only true if the cancelled
    // attempt released both custody slots. A double release would have
    // underflowed the registry's reserved count instead.
    let second = prepare(owner.as_ref(), &source, fork_both_intent("custody-third"))
        .expect("cancelling the first creation returned its custody slots");
    assert_ne!(second.identity(), &first_identity);
    assert_eq!(reservation_counts(owner.as_ref()), (1, 1, 2, 2, 1));
    drop(second);
    assert_eq!(reservation_counts(owner.as_ref()), baseline);
    assert_eq!(owner.state.operation.active(), 0);

    let third = prepare(owner.as_ref(), &source, fork_both_intent("custody-fourth"))
        .expect("dropping the second creation returned its custody slots too");
    assert_eq!(reservation_counts(owner.as_ref()), (1, 1, 2, 2, 1));
    drop(third);
    assert_eq!(reservation_counts(owner.as_ref()), baseline);
}

/// The creation half of the closed-owner contract: `prepare_creation` refuses
/// a closed owner and consumes nothing on the way out, and it decides
/// availability before it inspects the caller's intent — a bootstrap-form
/// intent that an open owner refuses as `PlansOmitted` is refused as
/// `OwnerUnavailable` once the owner is closed.
#[test]
fn closed_owner_rejects_creation_preparation_without_consuming_capacity() {
    let (owner, source) = setup(2);
    let before = reservation_counts(owner.as_ref());
    let plans_omitted = prepare(
        owner.as_ref(),
        &source,
        ProductBranchCreationIntent::named("open-owner-child")
            .expect("focused product branch name validates"),
    )
    .expect_err("a bootstrap-form intent carries no per-owner postures");
    assert_eq!(
        plans_omitted,
        RuntimeWorldBranchAdmissionDenial::PlansOmitted
    );

    let _report = owner.close().expect("owner closes while idle");
    let denied = prepare(owner.as_ref(), &source, fork_both_intent("closed-child"))
        .expect_err("a closed owner cannot prepare a creation");
    assert_eq!(denied, RuntimeWorldBranchAdmissionDenial::OwnerUnavailable);
    let unavailable = prepare(
        owner.as_ref(),
        &source,
        ProductBranchCreationIntent::named("closed-owner-child")
            .expect("focused product branch name validates"),
    )
    .expect_err("a closed owner is refused before its intent is lowered");
    assert_eq!(
        unavailable,
        RuntimeWorldBranchAdmissionDenial::OwnerUnavailable
    );
    assert_eq!(reservation_counts(owner.as_ref()), before);
    assert_eq!(owner.state.operation.active(), 0);
}
