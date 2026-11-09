//! Before-copy admission for the canonical carried descriptor and its Arc.
use crate::execution::RelationalExecutionDenialCause as Denial;
use crate::schema::data::*;

pub(super) fn retained(
    registry: &RelationalSchemaRegistry,
    control: crate::schema::SchemaCompilationControl<'_, '_>,
) -> Result<u64, Denial> {
    let mut bytes = SchemaAuthoritySnapshot::projection_allocation_bytes()?;
    add(
        &mut bytes,
        array::<SchemaAuthorityKindSnapshot>(registry.entity_kinds.len())?,
    )?;
    add(
        &mut bytes,
        array::<SchemaAuthorityRelationSnapshot>(registry.relation_kinds.len())?,
    )?;
    let mut basis = registry
        .entity_kinds
        .values()
        .map(|kind| (&kind.schema_id, kind.schema_version_id))
        .chain(
            registry
                .relation_kinds
                .values()
                .map(|kind| (&kind.schema_id, kind.schema_version_id)),
        );
    let first = basis.next();
    let mut consistent = true;
    for candidate in basis {
        control.checkpoint()?;
        consistent &= Some(candidate) == first;
    }
    if let Some((id, _)) = first.filter(|_| consistent) {
        add(&mut bytes, size(id.0.len())?)?;
    }
    for kind in registry.entity_kinds.values() {
        control.checkpoint()?;
        add(&mut bytes, size(kind.kind_name.len())?)?;
        add(&mut bytes, size(kind.schema_id.0.len())?)?;
    }
    for kind in registry.relation_kinds.values() {
        control.checkpoint()?;
        add(&mut bytes, size(kind.kind_name.len())?)?;
        add(&mut bytes, size(kind.schema_id.0.len())?)?;
    }
    add(
        &mut bytes,
        array::<crate::derived_relation::DerivedRelationDefinitionParts>(
            registry.derived_relations.len(),
        )?,
    )?;
    for definition in registry.derived_relations.values() {
        control.checkpoint()?;
        add(
            &mut bytes,
            size(checked(definition.parts.clone_allocation_bytes(control)?)?)?,
        )?;
    }
    add(
        &mut bytes,
        array::<crate::projected_member_domain::ProjectedMemberDomainParts>(
            registry.projected_member_domains.len(),
        )?,
    )?;
    for domain in registry.projected_member_domains.values() {
        control.checkpoint()?;
        add(
            &mut bytes,
            size(domain.parts.clone_allocation_bytes(control)?)?,
        )?;
    }
    Ok(bytes)
}

fn checked<T>(value: Option<T>) -> Result<T, Denial> {
    value.ok_or(Denial::MemoryOverflow)
}
fn size(bytes: usize) -> Result<u64, Denial> {
    u64::try_from(bytes).map_err(|_| Denial::MemoryOverflow)
}
fn add(total: &mut u64, bytes: u64) -> Result<(), Denial> {
    *total = checked(total.checked_add(bytes))?;
    Ok(())
}
fn array<T>(count: usize) -> Result<u64, Denial> {
    size(
        std::alloc::Layout::array::<T>(count)
            .map_err(|_| Denial::MemoryOverflow)?
            .size(),
    )
}
