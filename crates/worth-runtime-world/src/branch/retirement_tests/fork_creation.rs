use super::super::*;

use crate::branch::{
    ProductBranchCreationIntent, ProductBranchCreationPlans, RelationalBranchCreationPlan,
    SignalBranchCreationPlan,
};
use crate::budget::{
    RuntimeWorldBranchBudgetInstallation, RuntimeWorldBudgetInstallation, RuntimeWorldBudgets,
    RuntimeWorldCustodyBudgetInstallation, RuntimeWorldHistoryBudgetInstallation,
    RuntimeWorldObservationBudgetInstallation, RuntimeWorldPublicationBudgetInstallation,
    RuntimeWorldRecoveryBudgetInstallation, RuntimeWorldRetentionBudgetInstallation,
};
use crate::history::CompositeCommitParent;
use crate::lifecycle::{
    RuntimeWorldBranchCreationOutcome, RuntimeWorldBranchCreationRequest,
    RuntimeWorldBranchService, RuntimeWorldClock, RuntimeWorldObservationService,
    RuntimeWorldOwnerExecutionService, RuntimeWorldPreparationService,
    RuntimeWorldProductPublicationService,
};
use crate::publication::{
    CompositeLateCancellationPosture, CompositePublicationIntent, OwnerExecutionOutcome,
    RuntimeWorldCancellationSource,
};
use crate::recovery::ProductUnpublishedCause;
use worth_relational::facade::history::BranchId;
use worth_relational::facade::mvcc::RelationalTransactionIntent;
use worth_signal::facade::branch::validate_signal_branch_name;

type TestOwner = super::TestOwner;

#[path = "fork_creation/custody.rs"]
mod custody;
#[path = "fork_creation/destinations.rs"]
mod destinations;
#[path = "fork_creation/incarnation.rs"]
mod incarnation;
#[path = "fork_creation/matrix.rs"]
mod matrix;
#[path = "fork_creation/sibling_denial.rs"]
mod sibling_denial;
#[path = "fork_creation/source_token.rs"]
mod source_token;
#[path = "fork_creation/stale_head.rs"]
mod stale_head;

fn fork_budgets(live_branches: u64, custody_records: u64) -> RuntimeWorldBudgets {
    RuntimeWorldBudgets::install(RuntimeWorldBudgetInstallation {
        branches: RuntimeWorldBranchBudgetInstallation {
            live_product_branches: live_branches,
        },
        history: RuntimeWorldHistoryBudgetInstallation {
            retained_composite_commits: 12,
            history_metadata_bytes: 16384,
        },
        observations: RuntimeWorldObservationBudgetInstallation {
            active_observations: 8,
        },
        publication: RuntimeWorldPublicationBudgetInstallation {
            active_publication_attempts: 4,
        },
        recovery: RuntimeWorldRecoveryBudgetInstallation {
            retained_product_unpublished_records: 4,
            retained_partial_metadata_bytes:
                crate::recovery::ProductUnpublishedOwnerEffects::metadata_charge_hint()
                    .saturating_mul(4) as u64,
        },
        retention: RuntimeWorldRetentionBudgetInstallation {
            unique_exact_component_pins: 12,
            in_flight_pin_acquisition_reservations: 12,
        },
        custody: RuntimeWorldCustodyBudgetInstallation {
            owner_created_component_custody_records: custody_records,
        },
    })
    .expect("fork creation budgets")
}

pub(crate) fn setup_with_relational_source(
    live_branches: u64,
) -> (
    crate::branch::reference_test_fixture::RealReferenceFixture,
    TestOwner,
    ProductBranchObservation,
) {
    setup_with_custody_budget(live_branches, 4)
}

/// The same seeded source with an explicit owner-created custody ceiling, so a
/// creation can be starved of the slot its fork posture must charge.
pub(super) fn setup_with_custody_budget(
    live_branches: u64,
    custody_records: u64,
) -> (
    crate::branch::reference_test_fixture::RealReferenceFixture,
    TestOwner,
    ProductBranchObservation,
) {
    let mut fixture = crate::branch::reference_test_fixture::real_fixture(12, 12);
    let owner = TestOwner::new(fixture.owner_inputs(
        fork_budgets(live_branches, custody_records),
        RuntimeWorldClock::from_source(super::FixedClock),
    ))
    .expect("managed branch owner");
    let initial = bootstrap_root(&owner, &mut fixture);
    seed_relational_source(&owner, &mut fixture, initial);
    let source = current_root_observation(&owner);
    (fixture, owner, source)
}

fn bootstrap_root(
    owner: &TestOwner,
    fixture: &mut crate::branch::reference_test_fixture::RealReferenceFixture,
) -> ProductBranchObservation {
    match owner.bootstrap_root(fixture.bootstrap_intent()) {
        crate::branch::RuntimeWorldBootstrapOutcome::Performed(performed) => {
            performed.product_branch().clone()
        }
        crate::branch::RuntimeWorldBootstrapOutcome::NoEffect(no_effect) => {
            panic!(
                "branch bootstrap unexpectedly denied: {:?}",
                no_effect.cause()
            )
        }
    }
}

/// Give the source product head one real Relational occurrence, so a later
/// creation forks from a component commit the owner actually published.
pub(super) fn seed_relational_source(
    owner: &TestOwner,
    fixture: &mut crate::branch::reference_test_fixture::RealReferenceFixture,
    initial: ProductBranchObservation,
) {
    let cancellation = RuntimeWorldCancellationSource::new();
    let prepared = RuntimeWorldPreparationService::prepare_publication(
        owner,
        initial.clone(),
        CompositePublicationIntent::without_signal(RelationalTransactionIntent::ordinary())
            .with_prepared_relational_candidate(
                fixture.prepare_relational_owner_candidate("branch-fork-source-seed"),
            ),
        &cancellation.token(),
        None,
    )
    .expect("the source seed reserves its bounded publication resources");
    let settlement = match RuntimeWorldOwnerExecutionService::execute_without_signal(
        owner,
        prepared,
        &cancellation.token(),
    ) {
        OwnerExecutionOutcome::Settled(settlement) => settlement,
        other => panic!("source seed must settle: {other:?}"),
    };
    let successor = settlement
        .successor_basis()
        .cloned()
        .expect("source seed successor basis");
    let ready = settlement
        .ready(successor)
        .expect("source seed owner result is publication-ready");
    let cell = owner
        .state
        .branches
        .root_cell()
        .expect("bootstrapped root cell");
    match RuntimeWorldProductPublicationService::publish(
        owner,
        ready,
        &cell,
        CompositeLateCancellationPosture::NotRequested,
    ) {
        crate::publication::RuntimeWorldPublicationOutcome::Performed(_) => {}
        other => panic!("source seed must publish: {other:?}"),
    }
    drop(initial);
}

pub(super) fn current_root_observation(owner: &TestOwner) -> ProductBranchObservation {
    let cell = owner
        .state
        .branches
        .root_cell()
        .expect("bootstrapped root cell");
    RuntimeWorldObservationService::observe_product_branch(
        owner,
        &cell.atomic_snapshot().branch_identity().clone(),
    )
    .expect("the seeded root remains observable")
}

/// One cell of the creation matrix: each owner is told, on its own, whether to
/// reuse the component commit the source names or to fork an owner-issued
/// destination.
pub(crate) fn fork_intent(
    name: &str,
    relational: RelationalBranchCreationPlan,
    signal: SignalBranchCreationPlan,
) -> ProductBranchCreationIntent {
    ProductBranchCreationIntent::from_source(
        name,
        ProductBranchCreationPlans::new(relational, signal),
    )
    .expect("valid product branch name")
}

pub(crate) fn relational_fork(target: &str) -> RelationalBranchCreationPlan {
    RelationalBranchCreationPlan::ForkExact {
        target: BranchId(target.to_owned()),
    }
}

pub(crate) fn signal_fork(target: &str) -> SignalBranchCreationPlan {
    SignalBranchCreationPlan::ForkExact {
        target: validate_signal_branch_name(target).expect("valid Signal name"),
    }
}

pub(super) fn create_forked_branch(
    owner: &TestOwner,
    source: &ProductBranchObservation,
    intent: ProductBranchCreationIntent,
) -> ProductBranchObservation {
    let cancellation = RuntimeWorldCancellationSource::new();
    match RuntimeWorldBranchService::create_product_branch(
        owner,
        RuntimeWorldBranchCreationRequest::new(source.clone(), intent, &cancellation.token()),
    )
    .expect("owner-issued fork reaches a performed branch outcome")
    {
        RuntimeWorldBranchCreationOutcome::Performed(observation) => observation,
        RuntimeWorldBranchCreationOutcome::ProductUnpublished(_) => {
            panic!("fully admitted fork unexpectedly became unpublished")
        }
    }
}

pub(super) fn assert_new_branch_observation(
    owner: &TestOwner,
    source: &ProductBranchObservation,
    child: &ProductBranchObservation,
    history_before: usize,
) {
    assert_ne!(child.branch_identity(), source.branch_identity());
    assert_ne!(
        child.lifecycle_incarnation(),
        source.lifecycle_incarnation()
    );
    assert_eq!(child.reference_generation().get(), 0);
    assert_ne!(child.selected_commit(), source.selected_commit());
    assert_ne!(child.basis(), source.basis());
    let observed =
        RuntimeWorldObservationService::observe_product_branch(owner, child.branch_identity())
            .expect("the new branch observation is live");
    assert_eq!(observed, *child);
    let source_after =
        RuntimeWorldObservationService::observe_product_branch(owner, source.branch_identity())
            .expect("the source remains live");
    assert_eq!(source_after, *source);
    let commit = owner
        .state
        .history
        .lookup(child.selected_commit())
        .expect("the destination occurrence is installed in history");
    match commit.parent() {
        CompositeCommitParent::Ordinary(parent) => {
            assert_eq!(parent.commit(), source.selected_commit())
        }
        CompositeCommitParent::Root => panic!("a fork destination must have an ordinary parent"),
    }
    assert_eq!(owner.state.history.len(), history_before + 1);
    assert_eq!(owner.state.branches.branch_count(), 2);
    assert_eq!(owner.state.branches.reserved_branch_count(), 0);
}

#[test]
fn fork_exact_creates_a_distinct_composite_branch_after_both_owner_forks() {
    let (_fixture, owner, source) = setup_with_relational_source(3);
    let history_before = owner.state.history.len();
    let costs_before = owner.state.retention.cost_snapshot();
    let child = create_forked_branch(
        &owner,
        &source,
        fork_intent(
            "branch-fork-exact",
            relational_fork("relational-branch-fork-exact"),
            signal_fork("signal-branch-fork-exact"),
        ),
    );
    assert_new_branch_observation(&owner, &source, &child, history_before);
    assert_ne!(
        child.basis().relational_basis().identity(),
        source.basis().relational_basis().identity()
    );
    assert_ne!(
        child.basis().signal_basis().admission_identity(),
        source.basis().signal_basis().admission_identity()
    );
    let costs_after = owner.state.retention.cost_snapshot();
    assert!(costs_after.relational_contacts() > costs_before.relational_contacts());
    assert!(costs_after.signal_contacts() > costs_before.signal_contacts());
}

/// The observation a fork finalization issues joins the exact component slots
/// the fork's own publication already pinned on the destination basis. That is
/// what makes observation issuance after product movement reserved by
/// construction: it adds no unique pin, so it can meet no unique-pin capacity
/// denial once the head has moved. A fork of both owners moves both component
/// axes, so the world grows by exactly that pin pair and nothing more.
#[test]
fn fork_observation_issuance_adds_no_unique_pin_beyond_the_published_head() {
    let (_fixture, owner, source) = setup_with_relational_source(3);
    let pins_before = owner.state.retention.unique_pin_count();
    let child = create_forked_branch(
        &owner,
        &source,
        fork_intent(
            "branch-fork-observation-pins",
            relational_fork("relational-branch-fork-observation-pins"),
            signal_fork("signal-branch-fork-observation-pins"),
        ),
    );
    assert_eq!(
        owner.state.retention.unique_pin_count(),
        pins_before + 2,
        "the destination pins one new slot per forked owner; the observation pair joins them"
    );
    drop(child);
}
