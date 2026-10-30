#[allow(unused_imports)]
use crate::court::ordinary_context::branches::*;
pub(crate) use worth_relational::facade::history::BranchId;
pub(crate) use worth_runtime_world::facade::*;
pub(crate) use worth_signal::facade::branch::validate_signal_branch_name;
#[test]
fn omitted_plans_are_denied_before_owner_work() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let mut court = crate::court::ordinary_context::CompositeSupplyChainCourt::compile();
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
        request_execution,
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
            request_execution,
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
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let mut court = crate::court::ordinary_context::CompositeSupplyChainCourt::compile();
    let root = court.bootstrap();
    let token = RuntimeWorldCancellationSource::new().token();
    let branch = court.world.branch_port();
    let RuntimeWorldBranchCreationOutcome::Performed(existing) = branch
        .create_product_branch(
            request_execution,
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
        .create_product_branch(request_execution, root.clone(), attempt, &token)
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
