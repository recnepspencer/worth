use super::*;
use worth_relational::facade::history::BranchId;
use worth_signal::facade::branch::validate_signal_branch_name;

fn plans(relational: bool, signal: bool, name: &str) -> ProductBranchCreationPlans {
    ProductBranchCreationPlans::new(
        if relational {
            RelationalBranchCreationPlan::ForkExact {
                target: BranchId(name.into()),
            }
        } else {
            RelationalBranchCreationPlan::ReuseExact
        },
        if signal {
            SignalBranchCreationPlan::ForkExact {
                target: validate_signal_branch_name(name).unwrap(),
            }
        } else {
            SignalBranchCreationPlan::ReuseExact
        },
    )
}
#[test]
fn reuse_fork_matrix_retirement_and_aba_preserve_exact_owner_bindings() {
    for (relational, signal) in [(false, false), (true, false), (false, true), (true, true)] {
        let mut court = CompositeSupplyChainCourt::compile();
        let root = court.bootstrap();
        let source_records = court.records.read(root.basis().relational_basis());
        let signal_port = court
            .signal
            .owner_component_services()
            .unwrap()
            .basis_port();
        let before = signal_port.owner_service_cost_snapshot().unwrap();
        let token = RuntimeWorldCancellationSource::new().token();
        let branch_port = court.world.branch_port();
        let intent = ProductBranchCreationIntent::from_source(
            "child",
            plans(relational, signal, "component-child"),
        )
        .unwrap();
        let RuntimeWorldBranchCreationOutcome::Performed(child) = branch_port
            .create_product_branch(root.clone(), intent, &token)
            .unwrap()
        else {
            panic!("healthy fork matrix")
        };
        assert_eq!(
            court.records.read(child.basis().relational_basis()),
            source_records,
            "fork preserves semantic content"
        );
        assert_eq!(
            child.basis().relational_basis().admission_identity()
                == root.basis().relational_basis().admission_identity(),
            !relational
        );
        assert_eq!(
            child.basis().signal_basis().descriptor() == root.basis().signal_basis().descriptor(),
            !signal
        );
        assert_eq!(
            signal_port
                .owner_service_cost_snapshot()
                .unwrap()
                .fork_destination_installations()
                - before.fork_destination_installations(),
            u64::from(signal)
        );
        if relational {
            let identity = court
                .records
                .runtime
                .branch_identity(&BranchId("component-child".into()))
                .unwrap();
            assert_eq!(
                court
                    .records
                    .runtime
                    .observe_branch(&identity)
                    .unwrap()
                    .1
                    .admission_identity(),
                child.basis().relational_basis().admission_identity()
            );
        }
        let reference = signal_port
            .issue_managed_branch_reference(child.basis().signal_basis())
            .unwrap();
        assert_eq!(
            signal_port
                .observe_current(&reference)
                .unwrap()
                .descriptor(),
            child.basis().signal_basis().descriptor()
        );
        assert_eq!(
            court.observe(&root).selected_commit(),
            root.selected_commit(),
            "creation does not publish source"
        );
        let report = branch_port.retire_product_branch(&child).unwrap();
        assert_eq!(
            report.owner_retirement_work().len(),
            usize::from(relational) + usize::from(signal),
            "retirement reports exact fork custody for owner close"
        );
        assert!(court
            .world
            .observation_port()
            .observe_product_branch(child.branch_identity())
            .is_err());
        let RuntimeWorldBranchCreationOutcome::Performed(recreated) = branch_port
            .create_product_branch(
                root.clone(),
                ProductBranchCreationIntent::from_source("child", plans(false, false, "unused"))
                    .unwrap(),
                &token,
            )
            .unwrap()
        else {
            panic!("recreate retired name")
        };
        assert_ne!(
            child.lifecycle_incarnation(),
            recreated.lifecycle_incarnation()
        );
        assert!(
            branch_port.retire_product_branch(&child).is_err(),
            "old incarnation cannot retire new child"
        );
        let receipt = court.publish_cargo(&recreated, "5");
        assert_ne!(
            receipt.new_product_head().selected_commit(),
            recreated.selected_commit(),
            "create then publish is separate"
        );
        drop((root, child, recreated, receipt, report, reference));
        court.finish();
    }
}
#[test]
fn omitted_plans_are_denied_before_owner_work() {
    let mut court = CompositeSupplyChainCourt::compile();
    let root = court.bootstrap();
    let signal = court
        .signal
        .owner_component_services()
        .unwrap()
        .basis_port();
    let before = signal
        .owner_service_cost_snapshot()
        .unwrap()
        .fork_destination_installations();
    let denied = court.world.branch_port().create_product_branch(
        root.clone(),
        ProductBranchCreationIntent::named("child").unwrap(),
        &RuntimeWorldCancellationSource::new().token(),
    );
    assert!(matches!(
        denied,
        Err(RuntimeWorldServiceDenial::Denied(
            RuntimeWorldBranchAdmissionDenial::PlansOmitted
        ))
    ));
    assert_eq!(
        signal
            .owner_service_cost_snapshot()
            .unwrap()
            .fork_destination_installations(),
        before
    );
    let healthy = court
        .world
        .branch_port()
        .create_product_branch(
            root.clone(),
            ProductBranchCreationIntent::from_source("child", plans(false, false, "unused"))
                .unwrap(),
            &RuntimeWorldCancellationSource::new().token(),
        )
        .unwrap();
    drop((root, healthy));
    court.finish();
}

#[test]
fn sibling_fork_denial_retains_only_real_relational_custody_until_cleanup() {
    let mut court = CompositeSupplyChainCourt::compile();
    let root = court.bootstrap();
    let token = RuntimeWorldCancellationSource::new().token();
    let branch = court.world.branch_port();
    let RuntimeWorldBranchCreationOutcome::Performed(existing) = branch
        .create_product_branch(
            root.clone(),
            ProductBranchCreationIntent::from_source("existing", plans(false, true, "occupied"))
                .unwrap(),
            &token,
        )
        .unwrap()
    else {
        panic!("healthy Signal fork twin")
    };
    let signal = court
        .signal
        .owner_component_services()
        .unwrap()
        .basis_port();
    let before = signal
        .owner_service_cost_snapshot()
        .unwrap()
        .fork_destination_installations();
    let attempt = ProductBranchCreationIntent::from_source(
        "failed",
        ProductBranchCreationPlans::new(
            RelationalBranchCreationPlan::ForkExact {
                target: BranchId("new-relational".into()),
            },
            SignalBranchCreationPlan::ForkExact {
                target: validate_signal_branch_name("occupied").unwrap(),
            },
        ),
    )
    .unwrap();
    let RuntimeWorldBranchCreationOutcome::ProductUnpublished(effects) = branch
        .create_product_branch(root.clone(), attempt, &token)
        .unwrap()
    else {
        panic!("occupied Signal sibling must retain Relational fork")
    };
    assert_eq!(effects.owner_effect_count(), 1);
    assert!(court
        .records
        .runtime
        .branch_identity(&BranchId("new-relational".into()))
        .is_ok());
    assert_eq!(
        signal
            .owner_service_cost_snapshot()
            .unwrap()
            .fork_destination_installations(),
        before
    );
    assert_eq!(
        court.observe(&root).selected_commit(),
        root.selected_commit()
    );
    let handle = effects.recovery_handle();
    drop(effects);
    let cleanup = court
        .world
        .recovery_port()
        .release_effects(&handle, 0)
        .unwrap();
    assert_eq!(cleanup.unpublished_history_candidates().len(), 1);
    assert_eq!(
        cleanup.owner_retirement_work().len(),
        1,
        "one real Relational fork is reported for component owner close"
    );
    let retired = branch.retire_product_branch(&existing).unwrap();
    assert_eq!(retired.owner_retirement_work().len(), 1);
    drop((root, existing, retired, cleanup));
    court.finish();
}
