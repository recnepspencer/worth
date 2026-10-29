pub(crate) use {worth_runtime_world::facade::*, crate::court::ordinary_context::CompositeSupplyChainCourt};




pub(crate) fn signal_fork_child(
    court: &CompositeSupplyChainCourt,
    source: &ProductBranchObservation,
    name: &str,
) -> ProductBranchObservation {
    let host = crate::test_execution_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&host);
    let RuntimeWorldBranchCreationOutcome::Performed(child) = court
        .world
        .branch_port()
        .create_product_branch(
            &execution,
            source.clone(),
            ProductBranchCreationIntent::from_source(
                name,
                ProductBranchCreationPlans::new(
                    RelationalBranchCreationPlan::ReuseExact,
                    SignalBranchCreationPlan::ForkExact {
                        target: worth_signal::facade::branch::validate_signal_branch_name(name)
                            .unwrap(),
                    },
                ),
            )
            .unwrap(),
            &RuntimeWorldCancellationSource::new().token(),
        )
        .unwrap()
    else {
        panic!("genuine branch creation must perform");
    };
    child
}