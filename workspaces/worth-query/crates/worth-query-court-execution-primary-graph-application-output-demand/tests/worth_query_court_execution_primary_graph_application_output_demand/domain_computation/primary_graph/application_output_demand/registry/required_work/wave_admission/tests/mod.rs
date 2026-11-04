//! Only owner admission opens required-wave compute.
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::producer::execution::prepared_plan::OwnerPlan;
use worth_relational::facade::runtime::PositionedRelationalSnapshot;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::required_work::wave_admission::PreparedRequiredInputs;
use worth_query_execution::facade::domain_computation::WorthQueryOutputDemandDenialKind;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::WorthQueryOutputDemandRegistry;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::computation::prepared_request::PreparedCompute;

pub fn selected() -> std::sync::Arc<PositionedRelationalSnapshot> {
    let world =
        worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world(
            true,
        );
    world
        .application
        .with_host_advancement(|phase| {
            (*world
                .application
                .runtime_for_test())
                .primary_graph()
                .unwrap()
                .with_runtime(
                    &phase
                        .execution_request_for_for_test(&(*world.application.product_runtime_for_test()))
                        .unwrap(),
                    |runtime| {
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
                    },
                )
        })
        .expect("the wave-admission proof owns its request")
}
pub fn compute_plan() -> OwnerPlan<'static, (), ()> {
    OwnerPlan::new_for_test(vec![PreparedCompute::new_for_test(|_| ())], |slots| {
        assert_eq!(slots.len(), 1);
        Ok(())
    })
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
            vec![compute_plan().with_inputs_for_test(inputs)],
        );
        assert_eq!(
            outcome.is_ok(),
            accepted,
            "registry and native basis both bind admission"
        );
    }
}
