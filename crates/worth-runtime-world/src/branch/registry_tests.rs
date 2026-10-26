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
        Err(ProductBranchRegistryDenial::CapacityExhausted {
            limit: 1,
            occupancy: 1
        })
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

#[test]
fn defensive_branch_counter_overflow_and_allocation_have_distinct_producers() {
    use crate::branch::RuntimeWorldBranchAdmissionDenial;
    // A consistent live registry cannot reach these representational sizes.
    // Inject only its bookkeeping counter; no authority or branch is fabricated.
    for (reserved, expected) in [
        (
            usize::MAX,
            RuntimeWorldBranchAdmissionDenial::BranchOccupancyOverflow,
        ),
        (
            usize::MAX - 1,
            RuntimeWorldBranchAdmissionDenial::BranchAllocationRejected,
        ),
    ] {
        let construction = RuntimeWorldOwnerConstructionContract::new().unwrap();
        let owner = construction.owner_identity();
        let registry = ProductBranchRegistry::new(
            owner,
            RuntimeWorldBudgets::without_policy_limits().live_product_branches(),
        );
        registry.state.lock().unwrap().reserved_branches = reserved;
        let name = ProductBranchName::try_new("defensive-reservation").unwrap();
        assert_eq!(
            registry
                .reserve_branch(owner, name.clone())
                .map_err(RuntimeWorldBranchAdmissionDenial::from)
                .err(),
            Some(expected)
        );
        {
            let state = registry.state.lock().unwrap();
            assert_eq!(state.reserved_branches, reserved);
            assert!(state.reserved_names.is_empty());
            assert_eq!(state.entries.len(), 0);
        }
        registry.state.lock().unwrap().reserved_branches = 0;
        drop(registry.reserve_branch(owner, name).unwrap());
        assert_eq!(registry.reserved_branch_count(), 0);
    }
}

#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "branch capacity occupancy exceeds its installed limit")]
fn branch_capacity_desynchronization_is_loud_in_debug_builds() {
    let construction = RuntimeWorldOwnerConstructionContract::new().unwrap();
    let owner = construction.owner_identity();
    let registry = ProductBranchRegistry::new(owner, branch_limit(1));
    registry.state.lock().unwrap().reserved_branches = 2;
    let _ = registry.reserve_root(owner);
}
