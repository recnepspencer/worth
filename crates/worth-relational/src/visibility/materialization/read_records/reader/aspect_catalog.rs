use crate::runtime::RelationalRuntime;
use worth_foundational::facade::AspectKey;

pub(super) fn declared_aspects_for_entity_kind(
    runtime: &RelationalRuntime,
    kind_id: crate::identity::data::KindId,
    charge: &mut dyn FnMut(u64) -> Result<(), worth_execution::MapKernelStop>,
    memory: &mut worth_execution::ExecutionResultMemory<'_>,
) -> Result<Vec<AspectKey>, crate::execution::RelationalExecutionDenialCause> {
    runtime
        .schema_contract_runtime
        .aspect_contract_plans
        .entity_plans
        .get(&kind_id)
        .map(|plan| plan_aspect_keys(plan, charge, memory))
        .unwrap_or_else(|| Ok(Vec::new()))
}

pub(super) fn declared_aspects_for_relation_kind(
    runtime: &RelationalRuntime,
    kind_id: crate::identity::data::KindId,
    charge: &mut dyn FnMut(u64) -> Result<(), worth_execution::MapKernelStop>,
    memory: &mut worth_execution::ExecutionResultMemory<'_>,
) -> Result<Vec<AspectKey>, crate::execution::RelationalExecutionDenialCause> {
    runtime
        .schema_contract_runtime
        .aspect_contract_plans
        .relation_plans
        .get(&kind_id)
        .map(|plan| plan_aspect_keys(plan, charge, memory))
        .unwrap_or_else(|| Ok(Vec::new()))
}

fn plan_aspect_keys(
    plan: &crate::schema::data::LoweredAspectContractPlan,
    charge: &mut dyn FnMut(u64) -> Result<(), worth_execution::MapKernelStop>,
    memory: &mut worth_execution::ExecutionResultMemory<'_>,
) -> Result<Vec<AspectKey>, crate::execution::RelationalExecutionDenialCause> {
    let mut aspects = Vec::new();
    for binding in &plan.executable_bindings {
        charge(super::super::read_prices::ASPECT_BINDING)
            .map_err(crate::execution::denial::kernel_stop)?;
        let quoted = binding.aspect_key().owned_allocation_capacity_bytes() as u64;
        super::super::read_work::record_memory::prepare_push(memory, &mut aspects, quoted)?;
        let key = binding.aspect_key().clone();
        super::super::read_work::record_memory::settle_clone(
            memory,
            quoted,
            key.owned_allocation_capacity_bytes() as u64,
        )?;
        aspects.push(key);
    }
    charge(aspects.len().saturating_sub(1) as u64 * super::super::read_prices::ASPECT_COMPARISON)
        .map_err(crate::execution::denial::kernel_stop)?;
    if !aspects.windows(2).all(|window| window[0] < window[1]) {
        let n = aspects.len();
        let comparisons = if n <= 1 {
            0
        } else {
            n as u64 * u64::from(usize::BITS - (n - 1).leading_zeros())
        };
        charge(comparisons * super::super::read_prices::ASPECT_COMPARISON)
            .map_err(crate::execution::denial::kernel_stop)?;
        aspects.sort_unstable();
        aspects.dedup();
    }
    Ok(aspects)
}
