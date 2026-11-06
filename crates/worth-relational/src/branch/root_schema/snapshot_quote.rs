//! Before-copy quote of the canonical schema descriptor's owned temporary leaves.
use super::compilation_quote::{add, array, checked, size, Denial};
use crate::schema::data::*;

pub(super) fn temporary(
    registry: &RelationalSchemaRegistry,
    control: crate::schema::SchemaCompilationControl<'_, '_>,
) -> Result<u64, Denial> {
    let mut bytes = array::<SchemaAuthorityKindSnapshot>(registry.entity_kinds.len())?;
    add(
        &mut bytes,
        array::<SchemaAuthorityRelationSnapshot>(registry.relation_kinds.len())?,
    )?;
    // A common primary id is copied once. Summing every id is a conservative
    // bound without allocating a schema-basis result just to compute the quote.
    for kind in registry.entity_kinds.values() {
        control.checkpoint().map_err(Denial::Request)?;
        add(&mut bytes, size(kind.kind_name.len())?)?;
        add(
            &mut bytes,
            size(checked(kind.schema_id.0.len().checked_mul(2))?)?,
        )?;
    }
    for kind in registry.relation_kinds.values() {
        control.checkpoint().map_err(Denial::Request)?;
        add(&mut bytes, size(kind.kind_name.len())?)?;
        add(
            &mut bytes,
            size(checked(kind.schema_id.0.len().checked_mul(2))?)?,
        )?;
    }
    add(
        &mut bytes,
        array::<crate::derived_relation::DerivedRelationDefinitionParts>(
            registry.derived_relations.len(),
        )?,
    )?;
    for definition in registry.derived_relations.values() {
        control.checkpoint().map_err(Denial::Request)?;
        add(
            &mut bytes,
            size(checked(
                definition
                    .parts
                    .clone_allocation_bytes(control)
                    .map_err(Denial::Request)?,
            )?)?,
        )?;
    }
    add(
        &mut bytes,
        array::<crate::projected_member_domain::ProjectedMemberDomainParts>(
            registry.projected_member_domains.len(),
        )?,
    )?;
    for domain in registry.projected_member_domains.values() {
        control.checkpoint().map_err(Denial::Request)?;
        add(
            &mut bytes,
            size(
                domain
                    .parts
                    .clone_allocation_bytes(control)
                    .map_err(Denial::Request)?,
            )?,
        )?;
    }
    add(
        &mut bytes,
        registry.content_formats.snapshot().owned_bytes(),
    )?;
    Ok(bytes)
}
