//! A product-branch identity is the owner plus the name, so it outlives the
//! branch it named. The incarnation is what tells two occurrences of one name
//! apart, and everything keyed to an occurrence must be keyed to it as well.

use super::*;

use crate::branch::ProductBranchObservationMismatchAxis;

const ABA_NAME: &str = "branch-aba";
const PARTIAL_NAME: &str = "branch-aba-partial";

#[test]
fn retire_and_recreate_the_same_name_keeps_the_identity_and_advances_the_incarnation() {
    let (_fixture, owner, source) = super::super::setup(3);
    let first =
        super::super::create_reused_branch(&owner, &source, super::super::reuse_intent(ABA_NAME));
    let report = RuntimeWorldBranchService::retire_product_branch(&owner, &first)
        .expect("the first occurrence retires");
    assert!(report.owner_retirement_work().is_empty());

    let second =
        super::super::create_reused_branch(&owner, &source, super::super::reuse_intent(ABA_NAME));
    assert_eq!(
        second.branch_identity(),
        first.branch_identity(),
        "the identity is the name, so recreating it recovers the same identity"
    );
    assert_ne!(
        second.lifecycle_incarnation(),
        first.lifecycle_incarnation(),
        "a recreated name is a new occurrence"
    );
    assert_eq!(
        RuntimeWorldObservationService::observe_product_branch(&owner, first.branch_identity())
            .expect("the recreated name is observable"),
        second,
        "the identity alone resolves to the current occurrence"
    );
    assert!(matches!(
        owner.retire_product_branch(&first),
        Err(crate::branch::RuntimeWorldBranchRetirementDenial::AlreadyRetired)
    ));
    assert_eq!(
        owner
            .observe_product_branch(second.branch_identity())
            .unwrap(),
        second
    );
    assert_aba_observation_is_refused(&owner, &first, &second);
}

/// The observation admitted against the retired occurrence is not accepted for
/// the one that took its name.
fn assert_aba_observation_is_refused(
    owner: &TestOwner,
    first: &ProductBranchObservation,
    second: &ProductBranchObservation,
) {
    let mismatch = first
        .compare(second)
        .expect_err("two occurrences of one name are not the same observation");
    assert!(mismatch
        .axes()
        .contains(&ProductBranchObservationMismatchAxis::LifecycleIncarnation));

    let lifecycles_before = super::super::owner_lifecycles(owner);
    let branches_before = owner.state.branches.branch_count();
    let cancellation = RuntimeWorldCancellationSource::new();
    let denial = RuntimeWorldBranchService::create_product_branch(
        owner,
        RuntimeWorldBranchCreationRequest::new(
            first.clone(),
            fork_intent(
                "branch-aba-child",
                relational_fork("relational-branch-aba-child"),
                SignalBranchCreationPlan::ReuseExact,
            ),
            &cancellation.token(),
        ),
    )
    .expect_err("a stale occurrence cannot be the source of a creation");
    assert_eq!(denial, RuntimeWorldBranchAdmissionDenial::StaleSourceHead);
    assert_eq!(super::super::owner_lifecycles(owner), lifecycles_before);
    assert_eq!(owner.state.branches.branch_count(), branches_before);
    assert_eq!(owner.state.branches.reserved_branch_count(), 0);
}

/// A creation that ends product-unpublished keeps its custody record under the
/// occurrence that performed the fork. A later occurrence of the same name owns
/// its own component branches and must never inherit the earlier ones.
#[test]
fn custody_is_keyed_to_the_occurrence_that_created_it_not_the_reused_name() {
    let (_fixture, owner, source) = setup_with_relational_source(3);
    let effects = partial_creation_under_the_shared_name(&owner, &source);
    assert_eq!(owner.state.custody.installed(), 1);

    let recreated = create_forked_branch(
        &owner,
        &source,
        fork_intent(
            PARTIAL_NAME,
            relational_fork("relational-branch-aba-recreated"),
            SignalBranchCreationPlan::ReuseExact,
        ),
    );
    assert_eq!(owner.state.custody.installed(), 2);
    let report = RuntimeWorldBranchService::retire_product_branch(&owner, &recreated)
        .expect("the recreated occurrence retires");
    assert_eq!(report.owner_retirement_work().len(), 1);
    assert_eq!(
        report.owner_retirement_work()[0].target_name(),
        "relational-branch-aba-recreated"
    );
    let remaining = owner.state.custody.installed_records();
    assert_eq!(remaining.len(), 1);
    assert_eq!(
        remaining[0].target().name(),
        "relational-branch-aba-partial"
    );
    drop(effects);
}

/// One occurrence of the shared name that forks its Relational leg and is then
/// denied its Signal leg, so it holds real custody and no product branch.
fn partial_creation_under_the_shared_name(
    owner: &TestOwner,
    source: &ProductBranchObservation,
) -> crate::recovery::ProductUnpublishedOwnerEffects {
    let held = owner
        .state
        .signal
        .mutation_port()
        .reserve_fork_exact(
            validate_signal_branch_name("signal-branch-aba-partial").expect("valid Signal name"),
            source.basis().signal_basis(),
        )
        .expect("the destination the partial creation will ask for is already held");
    let cancellation = RuntimeWorldCancellationSource::new();
    let outcome = RuntimeWorldBranchService::create_product_branch(
        owner,
        RuntimeWorldBranchCreationRequest::new(
            source.clone(),
            fork_intent(
                PARTIAL_NAME,
                relational_fork("relational-branch-aba-partial"),
                signal_fork("signal-branch-aba-partial"),
            ),
            &cancellation.token(),
        ),
    )
    .expect("a held sibling destination is a product-unpublished outcome");
    drop(held);
    match outcome {
        RuntimeWorldBranchCreationOutcome::ProductUnpublished(effects) => effects,
        RuntimeWorldBranchCreationOutcome::Performed(_) => {
            panic!("a held Signal destination must deny the Signal fork")
        }
    }
}
