//! The shared allocation shape of compiled aspect plans, independent of roots.
use super::standard_tree_quote::retained_tree_bytes;
use crate::execution::RelationalExecutionDenialCause as Denial;
use crate::identity::data::KindId;
use crate::schema::{data::*, SchemaCompilationControl};
use std::alloc::Layout;

pub(super) fn retained_bytes(
    registry: &RelationalSchemaRegistry,
    control: SchemaCompilationControl<'_, '_>,
) -> Result<usize, Denial> {
    let mut bytes = arc_bytes::<AspectContractPlanCatalog>()?;
    add(
        &mut bytes,
        usize::try_from(
            retained_tree_bytes::<KindId, LoweredAspectContractPlan>(registry.entity_kinds.len())
                .map_err(|_| Denial::MemoryOverflow)?,
        )
        .map_err(|_| Denial::MemoryOverflow)?,
    )?;
    add(
        &mut bytes,
        usize::try_from(
            retained_tree_bytes::<KindId, LoweredAspectContractPlan>(registry.relation_kinds.len())
                .map_err(|_| Denial::MemoryOverflow)?,
        )
        .map_err(|_| Denial::MemoryOverflow)?,
    )?;
    for declarations in registry
        .entity_kinds
        .values()
        .map(|kind| &kind.aspect_contract_declarations)
        .chain(
            registry
                .relation_kinds
                .values()
                .map(|kind| &kind.aspect_contract_declarations),
        )
    {
        control.checkpoint()?;
        if declarations.aspects.len() > 8 {
            add(
                &mut bytes,
                declarations
                    .aspects
                    .len()
                    .checked_mul(size_of::<LoweredAspectContractBinding>())
                    .ok_or(Denial::MemoryOverflow)?,
            )?;
        }
        for aspect in &declarations.aspects {
            control.checkpoint()?;
            add(
                &mut bytes,
                aspect
                    .contract
                    .clone_allocation_bytes()
                    .ok_or(Denial::MemoryOverflow)?,
            )?;
            add(&mut bytes, aspect.binding.clone_allocation_bytes())?;
        }
    }
    Ok(bytes)
}
pub(super) fn arc_bytes<T>() -> Result<usize, Denial> {
    arc_layout(Layout::new::<T>())
}
fn arc_layout(data: Layout) -> Result<usize, Denial> {
    Ok(Layout::new::<[usize; 2]>()
        .extend(data)
        .map_err(|_| Denial::MemoryOverflow)?
        .0
        .pad_to_align()
        .size())
}
fn add(total: &mut usize, bytes: usize) -> Result<(), Denial> {
    *total = total.checked_add(bytes).ok_or(Denial::MemoryOverflow)?;
    Ok(())
}
