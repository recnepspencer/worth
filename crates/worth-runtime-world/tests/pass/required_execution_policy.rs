use worth_runtime_world::facade::*;
fn construct(
    bridge: RuntimeWorldCorrespondencePort,
    relational: RelationalOwnerServicePorts,
    signal: SignalOwnerServicePorts<(), (), (), (), ()>,
    publication: worth_signal::facade::branch::SignalConditionalDefinitionPublicationPort<
        (),
        (),
        (),
        (),
        (),
    >,
    budgets: RuntimeWorldBudgets,
    clock: RuntimeWorldClock,
    policy: worth_foundational::ExecutionRequestPolicy,
) {
    RuntimeWorldOwner::builder()
        .with_bridge_correspondence(bridge)
        .with_relational_services(relational)
        .with_signal_services(signal)
        .with_signal_definition_publication(publication)
        .with_budgets(budgets)
        .with_clock(clock)
        .with_execution_policy(policy)
        .build()
        .unwrap();
}
fn main() {}
