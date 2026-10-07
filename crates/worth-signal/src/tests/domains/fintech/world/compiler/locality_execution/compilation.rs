use crate::data::error::SignalError;
use crate::data::graph::SignalGraph;
use crate::facade::{DiagnosticsTier, SignalRuntime, SignalRuntimePolicy};
use crate::tests::domains::fintech::execution_tier::FintechTier;

use super::super::super::FinancialWorldDefinition;
use super::super::locality_evaluation::runtime_baseline_values;
use super::super::locality_topology::build_locality_topology;
use super::CompiledFinancialLocalityWorld;

pub(crate) fn compile_financial_locality_world(
    definition: FinancialWorldDefinition,
) -> Result<super::super::CompiledFinancialWorld, SignalError> {
    compile_financial_locality_world_at_tier(definition, DiagnosticsTier::Development)
}

pub(crate) fn compile_financial_locality_world_at_tier(
    definition: FinancialWorldDefinition,
    diagnostics_tier: DiagnosticsTier,
) -> Result<super::super::CompiledFinancialWorld, SignalError> {
    compile_financial_locality_world_with_policy(
        definition,
        SignalRuntimePolicy::for_tier(diagnostics_tier),
    )
}

pub(crate) fn compile_financial_locality_world_with_policy(
    definition: FinancialWorldDefinition,
    policy: SignalRuntimePolicy,
) -> Result<super::super::CompiledFinancialWorld, SignalError> {
    let locality = definition.locality().cloned().ok_or_else(|| {
        SignalError::invalid_input("financial locality compiler requires a locality courtroom")
    })?;
    locality.validate_generator_invariants();
    let mut runtime = SignalRuntime::builder(SignalGraph::new())
        .with_kernel_defaults()
        .with_tiers::<FintechTier>()
        .runtime_policy(policy)
        .build();
    let handles = build_locality_topology(&mut runtime.graph_mut(), &locality)?;
    let baseline_values = runtime_baseline_values(&locality)?;
    let mut compiled = CompiledFinancialLocalityWorld {
        runtime,
        definition,
        handles,
        baseline_values,
        baseline_preparation_report: None,
    };
    compiled.establish_causally_complete_baseline()?;
    compiled.seal_baseline()?;
    Ok(super::super::CompiledFinancialWorld::from_locality(
        compiled,
    ))
}
