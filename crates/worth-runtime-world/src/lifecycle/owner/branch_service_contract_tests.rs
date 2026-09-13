use crate::branch::{
    ProductBranchCreationIntent, ProductBranchCreationPlans, ProductBranchName,
    ProductBranchObservation, RelationalBranchCreationPlan, RuntimeWorldBootstrapOutcome,
    RuntimeWorldBranchAdmissionDenial, SignalBranchCreationPlan,
};
use crate::budget::{
    RuntimeWorldBranchBudgetInstallation, RuntimeWorldBudgetInstallation, RuntimeWorldBudgets,
    RuntimeWorldCustodyBudgetInstallation, RuntimeWorldHistoryBudgetInstallation,
    RuntimeWorldObservationBudgetInstallation, RuntimeWorldPublicationBudgetInstallation,
    RuntimeWorldRecoveryBudgetInstallation, RuntimeWorldRetentionBudgetInstallation,
};
use crate::lifecycle::{
    RuntimeWorldBranchCreationOutcome, RuntimeWorldBranchCreationRequest,
    RuntimeWorldBranchService, RuntimeWorldClock, RuntimeWorldClockSource, RuntimeWorldInstant,
};
use crate::publication::RuntimeWorldCancellationSource;

type TestOwner = super::super::RuntimeWorldOwnerRoot<(), (), (), (), ()>;

struct FixedClock;

impl RuntimeWorldClockSource for FixedClock {
    fn now(&self) -> RuntimeWorldInstant {
        RuntimeWorldInstant::from_ticks(7)
    }
}

fn budgets() -> RuntimeWorldBudgets {
    RuntimeWorldBudgets::install(RuntimeWorldBudgetInstallation {
        branches: RuntimeWorldBranchBudgetInstallation {
            live_product_branches: 3,
        },
        history: RuntimeWorldHistoryBudgetInstallation {
            retained_composite_commits: 8,
            history_metadata_bytes: 4096,
        },
        observations: RuntimeWorldObservationBudgetInstallation {
            active_observations: 8,
        },
        publication: RuntimeWorldPublicationBudgetInstallation {
            active_publication_attempts: 4,
        },
        recovery: RuntimeWorldRecoveryBudgetInstallation {
            retained_product_unpublished_records: 2,
            retained_partial_metadata_bytes: 4096,
        },
        retention: RuntimeWorldRetentionBudgetInstallation {
            unique_exact_component_pins: 8,
            in_flight_pin_acquisition_reservations: 8,
        },
        custody: RuntimeWorldCustodyBudgetInstallation {
            owner_created_component_custody_records: 2,
        },
    })
    .expect("branch service contract budgets")
}

fn reuse_intent(name: &str) -> ProductBranchCreationIntent {
    ProductBranchCreationIntent::from_source(
        name,
        ProductBranchCreationPlans::new(
            RelationalBranchCreationPlan::ReuseExact,
            SignalBranchCreationPlan::ReuseExact,
        ),
    )
    .expect("valid product branch name")
}

fn setup() -> (
    crate::branch::reference_test_fixture::RealReferenceFixture,
    TestOwner,
    ProductBranchObservation,
) {
    let mut fixture = crate::branch::reference_test_fixture::real_fixture(8, 8);
    let owner =
        TestOwner::new(fixture.owner_inputs(budgets(), RuntimeWorldClock::from_source(FixedClock)))
            .expect("managed branch owner");
    let root = match owner.bootstrap_root(fixture.bootstrap_intent()) {
        RuntimeWorldBootstrapOutcome::Performed(performed) => performed.product_branch().clone(),
        RuntimeWorldBootstrapOutcome::NoEffect(no_effect) => {
            panic!(
                "branch bootstrap unexpectedly denied: {:?}",
                no_effect.cause()
            )
        }
    };
    (fixture, owner, root)
}

fn create_reused_branch(
    owner: &TestOwner,
    source: &ProductBranchObservation,
    intent: ProductBranchCreationIntent,
) -> ProductBranchObservation {
    let cancellation = RuntimeWorldCancellationSource::new();
    match RuntimeWorldBranchService::create_product_branch(
        owner,
        RuntimeWorldBranchCreationRequest::new(source.clone(), intent, &cancellation.token()),
    )
    .expect("exact reuse is admitted")
    {
        RuntimeWorldBranchCreationOutcome::Performed(observation) => observation,
        RuntimeWorldBranchCreationOutcome::ProductUnpublished(_) => {
            panic!("exact reuse did not produce a branch observation")
        }
    }
}

#[test]
fn installed_duplicate_name_maps_to_the_exact_branch_denial() {
    let (_fixture, owner, root) = setup();
    let cancellation = RuntimeWorldCancellationSource::new();
    create_reused_branch(&owner, &root, reuse_intent("installed-duplicate"));
    let before = owner.state.branches.branch_count();

    assert!(matches!(
        RuntimeWorldBranchService::create_product_branch(
            &owner,
            RuntimeWorldBranchCreationRequest::new(
                root.clone(),
                reuse_intent("installed-duplicate"),
                &cancellation.token(),
            ),
        ),
        Err(RuntimeWorldBranchAdmissionDenial::DuplicateName)
    ));
    assert_eq!(owner.state.branches.branch_count(), before);
}

#[test]
fn held_duplicate_name_maps_to_the_same_denial_and_drop_releases_it() {
    let (_fixture, owner, root) = setup();
    let cancellation = RuntimeWorldCancellationSource::new();
    let held = owner
        .state
        .branches
        .reserve_branch(
            owner.owner_identity(),
            ProductBranchName::try_new("held-duplicate").expect("valid held name"),
        )
        .expect("the registry holds the named slot");

    assert!(matches!(
        RuntimeWorldBranchService::create_product_branch(
            &owner,
            RuntimeWorldBranchCreationRequest::new(
                root.clone(),
                reuse_intent("held-duplicate"),
                &cancellation.token(),
            ),
        ),
        Err(RuntimeWorldBranchAdmissionDenial::DuplicateName)
    ));
    drop(held);

    create_reused_branch(&owner, &root, reuse_intent("held-duplicate"));
}

/// A fork plan always carries its own owner-issued destination, so the only
/// way to reach creation without a per-owner plan is a bootstrap intent. That
/// is denied by name before any owner is contacted.
#[test]
fn creation_without_per_owner_plans_is_denied_before_any_owner_effect() {
    let (_fixture, owner, root) = setup();
    let cancellation = RuntimeWorldCancellationSource::new();
    let before_history = owner.state.history.counters();
    let before_retention = owner.state.retention.cost_snapshot();

    let intent =
        ProductBranchCreationIntent::named("missing-creation-plans").expect("valid branch name");
    assert!(intent.plans().is_none());
    assert!(matches!(
        RuntimeWorldBranchService::create_product_branch(
            &owner,
            RuntimeWorldBranchCreationRequest::new(root.clone(), intent, &cancellation.token()),
        ),
        Err(RuntimeWorldBranchAdmissionDenial::PlansOmitted)
    ));
    assert_eq!(owner.state.branches.branch_count(), 1);
    assert_eq!(owner.state.branches.reserved_branch_count(), 0);
    assert_eq!(owner.state.history.counters(), before_history);
    assert_eq!(owner.state.retention.cost_snapshot(), before_retention);
    assert_eq!(owner.recovery_record_count(), 0);
}
