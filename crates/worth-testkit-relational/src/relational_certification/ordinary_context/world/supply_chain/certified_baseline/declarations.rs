pub use {
    crate::relational_certification::ordinary_context::world::supply_chain::baseline_audit::audit as audit_supply_chain_baseline,
    crate::relational_certification::ordinary_context::world::supply_chain::comparison::compare,
    crate::relational_certification::ordinary_context::world::supply_chain::compiler::compile_supply_chain_baseline,
    crate::relational_certification::ordinary_context::world::supply_chain::definition::SupplyChainWorldDefinition,
    crate::relational_certification::ordinary_context::world::supply_chain::expected_observation::ExpectedSupplyChainObservation,
    crate::relational_certification::ordinary_context::world::supply_chain::production_world::ProductionSeededSupplyChainWorld,
    crate::relational_certification::ordinary_context::world::supply_chain::program::CompiledSupplyChainProgram,
    crate::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale,
};
pub use {
    worth_relational::facade::runtime::RelationalRuntime,
    worth_relational::facade::runtime::RelationalRuntimeApi,
};

pub fn certified_supply_chain_world(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    scale: SupplyChainScale,
) -> (
    ProductionSeededSupplyChainWorld,
    ExpectedSupplyChainObservation,
) {
    let definition =
        crate::relational_certification::ordinary_context::world::supply_chain::definition::SupplyChainWorldDefinition::operating(scale).expect("Supply Chain definition is valid");
    let program =
        crate::relational_certification::ordinary_context::world::supply_chain::program::CompiledSupplyChainProgram::compile(definition).expect("Supply Chain program compiles");
    let certified = audit_supply_chain_baseline(
        execution,
        compile_supply_chain_baseline(execution, program).expect("baseline installs"),
    )
    .expect("baseline matches the independent Supply Chain oracle");
    (certified.world, certified.expected)
}

pub fn canonical_empty_supply_chain_runtime(scale: SupplyChainScale) -> RelationalRuntime {
    let definition = crate::relational_certification::ordinary_context::world::supply_chain::definition::SupplyChainWorldDefinition::empty(empty_supply_chain_scale(scale));
    let program = crate::relational_certification::ordinary_context::world::supply_chain::program::CompiledSupplyChainProgram::compile(definition)
        .expect("empty Supply Chain definition compiles");
    let mut runtime = worth_relational::facade::runtime::RelationalRuntimeApi::builder()
        .build()
        .expect("valid native runtime configuration");
    runtime
        .prepare_initial_schema_installation()
        .expect("empty Supply Chain schema installation prepares")
        .install(program.schema_registry().clone())
        .expect("empty Supply Chain schema installation succeeds");
    runtime
}

pub fn assert_oracle_matches(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &ProductionSeededSupplyChainWorld,
    expected: &ExpectedSupplyChainObservation,
) {
    let observed = crate::relational_certification::ordinary_context::world::supply_chain::observation::observe(execution, world)
        .expect("production Supply Chain observation remains available");
    compare(expected, &observed).expect("production observation matches the independent oracle");
}

pub fn empty_supply_chain_scale(mut scale: SupplyChainScale) -> SupplyChainScale {
    scale.ports = 0;
    scale.terminals = 0;
    scale.berths = 0;
    scale.vessels = 0;
    scale.voyages = 0;
    scale.port_calls = 0;
    scale.cargo_lots = 0;
    scale.regions = 1;
    scale
}
