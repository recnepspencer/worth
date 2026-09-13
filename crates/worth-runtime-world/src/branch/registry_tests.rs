use super::{ProductBranchRegistry, ProductBranchRegistryDenial};
use crate::branch::ProductBranchName;
use crate::budget::{
    RuntimeWorldBranchBudgetInstallation, RuntimeWorldBudgetInstallation, RuntimeWorldBudgets,
    RuntimeWorldCustodyBudgetInstallation, RuntimeWorldHistoryBudgetInstallation,
    RuntimeWorldObservationBudgetInstallation, RuntimeWorldPublicationBudgetInstallation,
    RuntimeWorldRecoveryBudgetInstallation, RuntimeWorldRetentionBudgetInstallation,
};
use crate::lifecycle::owner::RuntimeWorldOwnerConstructionContract;

fn branch_limit(maximum: u64) -> crate::budget::RuntimeWorldBudgetLimit {
    RuntimeWorldBudgets::install(RuntimeWorldBudgetInstallation {
        branches: RuntimeWorldBranchBudgetInstallation {
            live_product_branches: maximum,
        },
        history: RuntimeWorldHistoryBudgetInstallation {
            retained_composite_commits: 1,
            history_metadata_bytes: 1,
        },
        observations: RuntimeWorldObservationBudgetInstallation {
            active_observations: 1,
        },
        publication: RuntimeWorldPublicationBudgetInstallation {
            active_publication_attempts: 1,
        },
        recovery: RuntimeWorldRecoveryBudgetInstallation {
            retained_product_unpublished_records: 1,
            retained_partial_metadata_bytes: 1,
        },
        retention: RuntimeWorldRetentionBudgetInstallation {
            unique_exact_component_pins: 1,
            in_flight_pin_acquisition_reservations: 1,
        },
        custody: RuntimeWorldCustodyBudgetInstallation {
            owner_created_component_custody_records: 1,
        },
    })
    .expect("test budget")
    .live_product_branches()
}

#[test]
fn named_reservations_are_bounded_and_drop_restores_the_slot_and_name() {
    let construction = RuntimeWorldOwnerConstructionContract::new().expect("owner identity");
    let owner = construction.owner_identity();
    let registry = ProductBranchRegistry::new(owner, branch_limit(1));
    let name = ProductBranchName::try_new("child").expect("valid name");
    let reservation = registry
        .reserve_branch(owner, name.clone())
        .expect("one branch slot");

    assert_eq!(registry.branch_count(), 0);
    assert_eq!(registry.reserved_branch_count(), 1);
    assert!(matches!(
        registry.reserve_branch(owner, ProductBranchName::try_new("other").unwrap()),
        Err(ProductBranchRegistryDenial::CapacityExhausted)
    ));
    assert!(matches!(
        registry.reserve_branch(owner, name.clone()),
        Err(ProductBranchRegistryDenial::NameAlreadyReserved)
    ));

    drop(reservation);
    assert_eq!(registry.reserved_branch_count(), 0);
    let replacement = registry
        .reserve_branch(owner, name)
        .expect("dropped reservation restores both bounds");
    drop(replacement);
}

#[test]
fn foreign_owner_cannot_consume_branch_capacity() {
    let first = RuntimeWorldOwnerConstructionContract::new().expect("first owner");
    let second = RuntimeWorldOwnerConstructionContract::new().expect("second owner");
    let registry = ProductBranchRegistry::new(first.owner_identity(), branch_limit(1));

    assert!(matches!(
        registry.reserve_branch(
            second.owner_identity(),
            ProductBranchName::try_new("foreign").unwrap()
        ),
        Err(ProductBranchRegistryDenial::ForeignOwner)
    ));
    assert_eq!(registry.reserved_branch_count(), 0);
}
