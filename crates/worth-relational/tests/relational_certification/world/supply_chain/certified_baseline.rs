use super::{
    audit_supply_chain_baseline, compare, compile_supply_chain_baseline,
    CompiledSupplyChainProgram, ExpectedSupplyChainObservation, ProductionSeededSupplyChainWorld,
    SupplyChainScale, SupplyChainWorldDefinition,
};
use worth_relational::facade::runtime::{RelationalRuntime, RelationalRuntimeApi};

pub(crate) fn certified_supply_chain_world(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    scale: SupplyChainScale,
) -> (
    ProductionSeededSupplyChainWorld,
    ExpectedSupplyChainObservation,
) {
    let definition =
        SupplyChainWorldDefinition::operating(scale).expect("Supply Chain definition is valid");
    let program =
        CompiledSupplyChainProgram::compile(definition).expect("Supply Chain program compiles");
    let certified = audit_supply_chain_baseline(
        execution,
        compile_supply_chain_baseline(program).expect("baseline installs"),
    )
    .expect("baseline matches the independent Supply Chain oracle");
    (certified.world, certified.expected)
}

pub(crate) fn canonical_empty_supply_chain_runtime(scale: SupplyChainScale) -> RelationalRuntime {
    let definition = SupplyChainWorldDefinition::empty(empty_supply_chain_scale(scale));
    let program = CompiledSupplyChainProgram::compile(definition)
        .expect("empty Supply Chain definition compiles");
    let mut runtime = RelationalRuntimeApi::builder().build();
    runtime
        .prepare_initial_schema_installation()
        .expect("empty Supply Chain schema installation prepares")
        .install(program.schema_registry().clone())
        .expect("empty Supply Chain schema installation succeeds");
    runtime
}

pub(crate) fn assert_oracle_matches(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &ProductionSeededSupplyChainWorld,
    expected: &ExpectedSupplyChainObservation,
) {
    let observed = super::observe_supply_chain(execution, world)
        .expect("production Supply Chain observation remains available");
    compare(expected, &observed).expect("production observation matches the independent oracle");
}

fn empty_supply_chain_scale(mut scale: SupplyChainScale) -> SupplyChainScale {
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
