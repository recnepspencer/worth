use crate::runtime::RelationalRuntime;
use worth_foundational::facade::AspectKey;

pub(super) fn declared_aspects_for_entity_kind(
    runtime: &RelationalRuntime,
    kind_id: crate::identity::data::KindId,
    charge: &mut dyn FnMut(u64) -> Result<(), worth_execution::MapKernelStop>,
) -> Result<Vec<AspectKey>, worth_execution::MapKernelStop> {
    runtime
        .schema_contract_runtime
        .aspect_contract_plans
        .entity_plans
        .get(&kind_id)
        .map(|plan| plan_aspect_keys(plan, charge))
        .unwrap_or_else(|| Ok(Vec::new()))
}

pub(super) fn declared_aspects_for_relation_kind(
    runtime: &RelationalRuntime,
    kind_id: crate::identity::data::KindId,
    charge: &mut dyn FnMut(u64) -> Result<(), worth_execution::MapKernelStop>,
) -> Result<Vec<AspectKey>, worth_execution::MapKernelStop> {
    runtime
        .schema_contract_runtime
        .aspect_contract_plans
        .relation_plans
        .get(&kind_id)
        .map(|plan| plan_aspect_keys(plan, charge))
        .unwrap_or_else(|| Ok(Vec::new()))
}

fn plan_aspect_keys(
    plan: &crate::schema::data::LoweredAspectContractPlan,
    charge: &mut dyn FnMut(u64) -> Result<(), worth_execution::MapKernelStop>,
) -> Result<Vec<AspectKey>, worth_execution::MapKernelStop> {
    let mut aspects = Vec::new();
    for binding in &plan.executable_bindings {
        charge(super::super::read_prices::ASPECT_BINDING)?;
        aspects.push(binding.aspect_key().clone());
    }
    charge(aspects.len().saturating_sub(1) as u64 * super::super::read_prices::ASPECT_COMPARISON)?;
    if !aspects.windows(2).all(|window| window[0] < window[1]) {
        let n = aspects.len();
        let comparisons = if n <= 1 {
            0
        } else {
            n as u64 * u64::from(usize::BITS - (n - 1).leading_zeros())
        };
        charge(comparisons * super::super::read_prices::ASPECT_COMPARISON)?;
        aspects.sort();
        aspects.dedup();
    }
    Ok(aspects)
}
