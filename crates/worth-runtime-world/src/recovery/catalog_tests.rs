use crate::budget::{
    RuntimeWorldBranchBudgetInstallation, RuntimeWorldBudgetInstallation, RuntimeWorldBudgets,
    RuntimeWorldCustodyBudgetInstallation, RuntimeWorldHistoryBudgetInstallation,
    RuntimeWorldObservationBudgetInstallation, RuntimeWorldPublicationBudgetInstallation,
    RuntimeWorldRecoveryBudgetInstallation, RuntimeWorldRetentionBudgetInstallation,
};
use crate::identity::RuntimeWorldOwnerIdentity;
use crate::lifecycle::owner::RuntimeWorldOwnerConstructionContract;

use super::{ProductUnpublishedRecoveryCatalog, RecoveryCatalogDenial};

fn owner() -> RuntimeWorldOwnerIdentity {
    RuntimeWorldOwnerConstructionContract::new()
        .expect("recovery test owner")
        .owner_identity()
}

fn limit(value: u64) -> crate::budget::RuntimeWorldBudgetLimit {
    RuntimeWorldBudgets::install(RuntimeWorldBudgetInstallation {
        branches: RuntimeWorldBranchBudgetInstallation {
            live_product_branches: 1,
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
            retained_product_unpublished_records: value,
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
    .expect("positive recovery test budgets")
    .retained_product_unpublished_records()
}

#[test]
fn recovery_slot_drop_restores_capacity_without_owner_contact() {
    let owner = owner();
    let catalog = ProductUnpublishedRecoveryCatalog::new(owner, limit(1));
    let slot = catalog
        .reserve_product_unpublished(owner)
        .expect("one recovery slot");
    assert_eq!(catalog.reserved_slots(), 1);
    drop(slot);
    assert_eq!(catalog.reserved_slots(), 0);
    assert_eq!(catalog.maximum_slots(), 1);
}

#[test]
fn recovery_slot_denials_preserve_the_real_counter() {
    let catalog_owner = owner();
    let foreign = owner();
    let catalog = ProductUnpublishedRecoveryCatalog::new(catalog_owner, limit(1));
    assert!(matches!(
        catalog.reserve_product_unpublished(foreign),
        Err(RecoveryCatalogDenial::ForeignOwner { .. })
    ));
    let first = catalog
        .reserve_product_unpublished(catalog_owner)
        .expect("the only recovery slot");
    assert!(matches!(
        catalog.reserve_product_unpublished(catalog_owner),
        Err(RecoveryCatalogDenial::CapacityExhausted { maximum: 1 })
    ));
    assert_eq!(catalog.reserved_slots(), 1);
    drop(first);
    assert_eq!(catalog.reserved_slots(), 0);
}
