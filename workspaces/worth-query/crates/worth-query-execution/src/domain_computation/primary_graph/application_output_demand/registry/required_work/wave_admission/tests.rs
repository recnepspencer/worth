//! Only owner admission opens required-wave compute.
use super::*;
use crate::domain_computation::primary_graph::{
    application_contribution::computation::prepared_request::PreparedCompute,
    application_contribution::OwnerPlan,
};
use worth_relational::facade::mvcc::CompanionPreflightBudget;
fn admission() -> InvalidationEditAdmission {
    InvalidationEditAdmission::new(CompanionPreflightBudget {
        maximum_work_visits: 1_000_000,
        maximum_preparation_bytes: 1 << 20,
    })
}
fn selected() -> std::sync::Arc<PositionedRelationalSnapshot> {
    let world =
        crate::domain_computation::primary_graph::tests::fixture::installed_authorization_world(
            true,
        );
    world
        .application
        .with_host_advancement(|phase| {
            let execution = phase
                .execution_request_for(&world.application.product_runtime)
                .unwrap();
            world
                .application
                .runtime
                .primary_graph()
                .unwrap()
                .with_runtime(&execution, |runtime| {
                    let basis = runtime
                        .admit_branch_basis(&runtime.main_branch_identity())
                        .unwrap();
                    let snapshot = runtime
                        .snapshots()
                        .snapshot_for_observation(&basis.observation())
                        .unwrap();
                    std::sync::Arc::new(
                        runtime.read_truth().positioned_snapshot(&snapshot).unwrap(),
                    )
                })
        })
        .unwrap()
}
fn compute_plan() -> OwnerPlan<'static, (), ()> {
    OwnerPlan::new(
        vec![PreparedCompute::new(0, 0, |_| (), |_| 0)],
        |slots, _| {
            assert_eq!(slots.len(), 1);
            Ok(())
        },
    )
}
#[test]
fn computing_members_without_owner_input_admission_are_refused() {
    let registry = WorthQueryOutputDemandRegistry::default();
    let basis = selected();
    let denied = registry
        .admit_prepared_wave(
            &basis,
            worth_proof::CanonicalUniqueVec::try_from_sorted_unique(vec![0]).unwrap(),
            vec![compute_plan()],
            &mut admission(),
        )
        .err()
        .unwrap();
    assert_eq!(
        denied.kind(),
        WorthQueryOutputDemandDenialKind::ForeignSettlement
    );
}
#[test]
fn wave_admission_requires_the_same_owner_and_native_basis() {
    let registry = WorthQueryOutputDemandRegistry::default();
    let basis = selected();
    for (owner, root, accepted) in [
        (registry.clone(), basis.root_id(), true),
        (
            registry.clone(),
            basis.root_id().checked_add(1).unwrap(),
            false,
        ),
        (
            WorthQueryOutputDemandRegistry::default(),
            basis.root_id(),
            false,
        ),
    ] {
        let inputs = PreparedRequiredInputs {
            runtime: basis.runtime_instance_id(),
            root,
            owner,
        };
        let outcome = registry.admit_prepared_wave(
            &basis,
            worth_proof::CanonicalUniqueVec::try_from_sorted_unique(vec![0]).unwrap(),
            vec![compute_plan().with_inputs(inputs)],
            &mut admission(),
        );
        assert_eq!(
            outcome.is_ok(),
            accepted,
            "registry and native basis both bind admission"
        );
    }
}
