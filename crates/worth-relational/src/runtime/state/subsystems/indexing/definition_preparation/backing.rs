//! Before-growth backing admission retained by the actual immutable catalog.
use super::*;
use RelationalIndexDefinitionPreparationDenial as Denial;

use crate::indexes::{IndexDefinitionBacking, RetainedIndexDefinitionMeaning};

fn add(total: &mut u64, bytes: u64) -> Result<(), Denial> {
    *total = total.checked_add(bytes).ok_or(Denial::CapacityOverflow)?;
    Ok(())
}
fn tree<K, V>(entries: usize) -> Result<u64, Denial> {
    crate::branch::retained_compilation_tree_bytes::<K, V>(entries)
        .map_err(|_| Denial::CapacityOverflow)
}
fn arc<T>() -> Result<u64, Denial> {
    std::alloc::Layout::new::<[usize; 2]>()
        .extend(std::alloc::Layout::new::<T>())
        .map(|(layout, _)| layout.pad_to_align().size() as u64)
        .map_err(|_| Denial::CapacityOverflow)
}
fn retain(
    catalog: &mut IndexDefinitionCatalog,
    request: &ExecutionRequest<'_, '_>,
    mut bytes: u64,
) -> Result<(), Denial> {
    add(&mut bytes, arc::<IndexDefinitionBacking>()?)?;
    let memory = request
        .reserve_retained_memory(bytes)
        .map_err(Denial::Request)?;
    catalog.backing = Some(Arc::new(IndexDefinitionBacking {
        _prior: catalog.backing.clone(),
        _memory: memory,
    }));
    Ok(())
}

pub(super) fn copy_catalog(
    original: &IndexDefinitionCatalog,
    request: &ExecutionRequest<'_, '_>,
) -> Result<IndexDefinitionCatalog, Denial> {
    let mut bytes = tree::<DerivedIndexId, crate::indexes::RetainedDerivedIndexDefinition>(
        original.definitions.len(),
    )?;
    add(
        &mut bytes,
        tree::<DerivedIndexId, u64>(original.definitions.len())?,
    )?;
    add(
        &mut bytes,
        tree::<AspectKey, BTreeMap<FieldKey, Vec<crate::indexes::RetainedDerivedIndexDefinition>>>(
            original.field_definitions.len(),
        )?,
    )?;
    for (aspect, fields) in &original.field_definitions {
        request.consult().map_err(Denial::Request)?;
        add(&mut bytes, aspect.as_str().len() as u64)?;
        add(
            &mut bytes,
            tree::<FieldKey, Vec<crate::indexes::RetainedDerivedIndexDefinition>>(fields.len())?,
        )?;
        for (field, definitions) in fields {
            request.consult().map_err(Denial::Request)?;
            add(&mut bytes, field.as_str().len() as u64)?;
            add(
                &mut bytes,
                std::alloc::Layout::array::<crate::indexes::RetainedDerivedIndexDefinition>(
                    definitions.len(),
                )
                .map_err(|_| Denial::CapacityOverflow)?
                .size() as u64,
            )?;
        }
    }
    add(&mut bytes, arc::<IndexDefinitionBacking>()?)?;
    let memory = request
        .reserve_retained_memory(bytes)
        .map_err(Denial::Request)?;
    let mut catalog = original.clone();
    catalog.backing = Some(Arc::new(IndexDefinitionBacking {
        _prior: original.backing.clone(),
        _memory: memory,
    }));
    Ok(catalog)
}

pub(super) fn prepare_insert(
    catalog: &mut IndexDefinitionCatalog,
    definition: &DerivedIndexDefinition,
    request: &ExecutionRequest<'_, '_>,
) -> Result<(), Denial> {
    // The reviewed standard tree retains at most n+1 nodes for n entries.
    // Paying two maximal nodes per insertion also covers a transient split;
    // the preceding catalog copy paid its entire original node population.
    let mut bytes = tree::<DerivedIndexId, crate::indexes::RetainedDerivedIndexDefinition>(1)?;
    add(&mut bytes, tree::<DerivedIndexId, u64>(1)?)?;
    add(&mut bytes, arc::<RetainedIndexDefinitionMeaning>()?)?;
    // This definition is moved, not copied. Retain the original caller backing
    // under this request before adopting it into the runtime catalog.
    add(&mut bytes, definition.owned_allocation_capacity_bytes())?;
    if let Some((aspect, field)) = field_index_locator(&definition.kind) {
        add(
            &mut bytes,
            tree::<
                AspectKey,
                BTreeMap<FieldKey, Vec<crate::indexes::RetainedDerivedIndexDefinition>>,
            >(1)?,
        )?;
        add(
            &mut bytes,
            tree::<FieldKey, Vec<crate::indexes::RetainedDerivedIndexDefinition>>(1)?,
        )?;
        add(&mut bytes, aspect.as_str().len() as u64)?;
        add(&mut bytes, field.as_str().len() as u64)?;
        let previous = catalog
            .field_definitions
            .get(&aspect)
            .and_then(|fields| fields.get(&field))
            .map_or(0, Vec::len);
        let capacity = previous
            .checked_add(1)
            .and_then(|n| n.checked_mul(2))
            .ok_or(Denial::CapacityOverflow)?
            .max(4);
        add(
            &mut bytes,
            std::alloc::Layout::array::<crate::indexes::RetainedDerivedIndexDefinition>(capacity)
                .map_err(|_| Denial::CapacityOverflow)?
                .size() as u64,
        )?;
    }
    retain(catalog, request, bytes)
}

pub(super) fn retain_arc(
    catalog: &mut IndexDefinitionCatalog,
    request: &ExecutionRequest<'_, '_>,
) -> Result<(), Denial> {
    {
        let bytes = arc::<IndexDefinitionCatalog>()?
            .checked_add(arc::<PreparedRelationalIndexDefinitions>()?)
            .ok_or(Denial::CapacityOverflow)?;
        retain(catalog, request, bytes)
    }
}
