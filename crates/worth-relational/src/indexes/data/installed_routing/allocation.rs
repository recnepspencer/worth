//! Before-growth geometry for the actual routing collections being compiled.
use super::super::DerivedIndexMaintenanceDenialKind as Denial;
use std::collections::{BTreeMap, BTreeSet};
use worth_foundational::facade::AspectFieldLocator;
pub(super) type Payment<'a> = dyn FnMut(u64, u64) -> Result<(), Denial> + 'a;
pub(super) fn tree<K, V>(entries: usize) -> Result<u64, Denial> {
    crate::branch::retained_compilation_tree_bytes::<K, V>(entries)
        .map_err(|_| Denial::WorkBudgetExceeded)
}
fn new_entry<K, V>(entries: usize, width: usize, pay: &mut Payment<'_>) -> Result<(), Denial> {
    let bytes = if entries == 0 {
        tree::<K, V>(1)?
    } else {
        tree::<K, V>(1)?
            .checked_sub(tree::<K, V>(0)?)
            .ok_or(Denial::WorkBudgetExceeded)?
    };
    navigation(entries, width, pay)?;
    pay(1, bytes)
}
pub(super) fn navigation(
    entries: usize,
    width: usize,
    pay: &mut Payment<'_>,
) -> Result<(), Denial> {
    let levels = u64::from(usize::BITS - entries.leading_zeros()) + 1;
    let work = levels
        .checked_mul(22)
        .and_then(|n| n.checked_mul(width as u64))
        .ok_or(Denial::WorkBudgetExceeded)?;
    pay(work, 0)
}
pub(super) fn set<K: Ord>(
    set: &mut BTreeSet<K>,
    key: K,
    width: usize,
    pay: &mut Payment<'_>,
) -> Result<(), Denial> {
    navigation(set.len(), width, pay)?;
    if !set.contains(&key) {
        new_entry::<K, ()>(set.len(), width, pay)?;
    }
    set.insert(key);
    Ok(())
}
pub(super) fn map<'a, K: Ord, V: Default>(
    map: &'a mut BTreeMap<K, V>,
    key: K,
    width: usize,
    pay: &mut Payment<'_>,
) -> Result<&'a mut V, Denial> {
    navigation(map.len(), width, pay)?;
    if !map.contains_key(&key) {
        new_entry::<K, V>(map.len(), width, pay)?;
    }
    Ok(map.entry(key).or_default())
}
pub(super) fn grow<T>(values: &mut Vec<T>, pay: &mut Payment<'_>) -> Result<(), Denial> {
    if values.len() < values.capacity() {
        return Ok(());
    }
    let next = values
        .capacity()
        .checked_mul(2)
        .ok_or(Denial::WorkBudgetExceeded)?
        .max(4);
    let bytes = std::alloc::Layout::array::<T>(next)
        .map_err(|_| Denial::WorkBudgetExceeded)?
        .size();
    // The existing reservation still covers the old vector during reallocation.
    pay(values.len() as u64 + 1, bytes as u64)?;
    values.try_reserve_exact(next - values.len()).map_err(|_| {
        Denial::Execution(crate::execution::RelationalExecutionDenialCause::AllocationFailed)
    })?;
    Ok(())
}
pub(super) fn push<K: Ord, V>(
    map: &mut BTreeMap<K, Vec<V>>,
    key: K,
    value: V,
    width: usize,
    pay: &mut Payment<'_>,
) -> Result<(), Denial> {
    let values = self::map(map, key, width, pay)?;
    grow(values, pay)?;
    values.push(value);
    Ok(())
}
pub(super) fn locator(
    locator: &AspectFieldLocator,
    pay: &mut Payment<'_>,
) -> Result<usize, Denial> {
    let width = locator
        .field_path()
        .fields()
        .iter()
        .try_fold(
            locator.aspect().aspect_key().as_str().len() + 1,
            |n, field| n.checked_add(field.as_str().len() + 1),
        )
        .ok_or(Denial::WorkBudgetExceeded)?;
    pay(
        width as u64,
        locator.owned_allocation_capacity_bytes() as u64,
    )?;
    Ok(width)
}
pub(super) fn scope(
    plan: &crate::schema::data::LoweredAspectContractPlan,
    locator: &AspectFieldLocator,
    pay: &mut Payment<'_>,
) -> Result<(), Denial> {
    let width = locator.aspect().aspect_key().as_str().len() + 1;
    pay(
        (plan.executable_bindings.len() as u64)
            .checked_mul(width as u64)
            .ok_or(Denial::WorkBudgetExceeded)?,
        0,
    )?;
    let width = locator
        .field_path()
        .fields()
        .first()
        .map_or(1, |field| field.as_str().len() + 1);
    for binding in &plan.executable_bindings {
        pay(width as u64, 0)?;
        if let worth_foundational::AspectShape::Struct(shape) = binding.contract.shape() {
            pay(
                (shape.fields().len() as u64)
                    .checked_mul(width as u64)
                    .ok_or(Denial::WorkBudgetExceeded)?,
                0,
            )?;
        }
    }
    Ok(())
}
