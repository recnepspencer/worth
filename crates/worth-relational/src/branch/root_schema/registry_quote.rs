//! Concrete container and moved-leaf custody of one owned schema registry.
use super::compilation_quote::{add, array, checked, size, tree, Denial};
use crate::identity::data::KindId;
use crate::schema::data::*;
use std::sync::Arc;

pub(super) fn retained(
    registry: &RelationalSchemaRegistry,
    control: crate::schema::SchemaCompilationControl<'_, '_>,
) -> Result<u64, Denial> {
    let mut bytes = tree::<KindId, EntityKindRegistration>(registry.entity_kinds.len())?;
    add(
        &mut bytes,
        tree::<KindId, RelationKindRegistration>(registry.relation_kinds.len())?,
    )?;
    add(
        &mut bytes,
        tree::<KindId, ()>(registry.native_private_kinds.len())?,
    )?;
    if let Some(storage) = &registry.current_publication_storage {
        control.checkpoint().map_err(Denial::Request)?;
        add(
            &mut bytes,
            array::<KindId>(storage.entity_kinds.capacity())?,
        )?;
    }
    add(
        &mut bytes,
        tree::<
            crate::derived_relation::DerivedRelationIdentity,
            Arc<crate::derived_relation::definition::RegisteredDerivedRelation>,
        >(registry.derived_relations.len())?,
    )?;
    add(
        &mut bytes,
        tree::<
            crate::projected_member_domain::ProjectedMemberDomainIdentity,
            Arc<crate::projected_member_domain::RegisteredProjectedMemberDomain>,
        >(registry.projected_member_domains.len())?,
    )?;
    for kind in registry.entity_kinds.values() {
        control.checkpoint().map_err(Denial::Request)?;
        add(&mut bytes, size(kind.kind_name.capacity())?)?;
        add(&mut bytes, size(kind.schema_id.0.capacity())?)?;
        add(
            &mut bytes,
            declarations(&kind.aspect_contract_declarations, control)?,
        )?;
    }
    for kind in registry.relation_kinds.values() {
        control.checkpoint().map_err(Denial::Request)?;
        add(&mut bytes, size(kind.kind_name.capacity())?)?;
        add(&mut bytes, size(kind.schema_id.0.capacity())?)?;
        add(
            &mut bytes,
            declarations(&kind.aspect_contract_declarations, control)?,
        )?;
        add(
            &mut bytes,
            size(checked(
                kind.relation_integrity
                    .retained_allocation_bytes(control)
                    .map_err(Denial::Request)?,
            )?)?,
        )?;
    }
    // Registered recipe/domain values already have shared owners. Only these
    // map keys are moved owned text; do not recursively bill the Arc payloads.
    for key in registry.derived_relations.keys() {
        control.checkpoint().map_err(Denial::Request)?;
        add(
            &mut bytes,
            size(checked(
                key.schema.0.capacity().checked_add(key.name.capacity()),
            )?)?,
        )?;
    }
    for key in registry.projected_member_domains.keys() {
        control.checkpoint().map_err(Denial::Request)?;
        add(
            &mut bytes,
            size(checked(
                key.schema.0.capacity().checked_add(key.name.capacity()),
            )?)?,
        )?;
    }
    let content = registry
        .content_formats
        .copy_geometry(control)
        .map_err(Denial::Request)?;
    use worth_foundational::facade::CanonicalDigestId;
    if content.formats != 0 {
        add(
            &mut bytes,
            tree::<CanonicalDigestId, crate::content::RelationalContentFormat>(content.formats)?,
        )?;
    }
    if content.names != 0 {
        add(
            &mut bytes,
            tree::<String, std::collections::BTreeMap<u32, CanonicalDigestId>>(content.names)?,
        )?;
    }
    if content.bindings != 0 {
        add(
            &mut bytes,
            tree::<CanonicalDigestId, CanonicalDigestId>(content.bindings)?,
        )?;
    }
    // The reviewed tree quote is linear in its admitted node bound. Sum the
    // actual per-name version roots rather than inventing a full catalog width.
    add(
        &mut bytes,
        checked(tree::<u32, CanonicalDigestId>(0)?.checked_mul(size(content.version_node_bound)?))?,
    )?;
    add(&mut bytes, size(content.name_bytes)?)?;
    Ok(bytes)
}

fn declarations(
    value: &KindAspectContractDeclarations,
    control: crate::schema::SchemaCompilationControl<'_, '_>,
) -> Result<u64, Denial> {
    let mut bytes = array::<DeclaredAspectContractBinding>(value.aspects.capacity())?;
    add(
        &mut bytes,
        array::<crate::merge::data::IdentityBasisDeclaration>(
            value.identity_declarations.capacity(),
        )?,
    )?;
    add(
        &mut bytes,
        array::<crate::merge::data::AspectMergePolicyDeclaration>(
            value.merge_policy_declarations.capacity(),
        )?,
    )?;
    for aspect in &value.aspects {
        control.checkpoint().map_err(Denial::Request)?;
        // These are already allocated, adopted leaves. Growth/Clone elsewhere
        // uses the separate checked clone quote, never this capacity observer.
        add(
            &mut bytes,
            size(aspect.binding.owned_allocation_capacity_bytes())?,
        )?;
        add(
            &mut bytes,
            size(aspect.contract.owned_allocation_capacity_bytes())?,
        )?;
    }
    for identity in &value.identity_declarations {
        control.checkpoint().map_err(Denial::Request)?;
        use crate::merge::data::{IdentityBasisKind, IdentityBasisScope};
        if let IdentityBasisScope::AspectKey(key) = &identity.scope {
            add(&mut bytes, size(key.owned_allocation_capacity_bytes())?)?;
        }
        match &identity.basis {
            IdentityBasisKind::StorageIdentity
            | IdentityBasisKind::LineageIdentity
            | IdentityBasisKind::StructuralFingerprint
            | IdentityBasisKind::DeclaredKeySet(_)
            | IdentityBasisKind::Custom(_) => {} // Actual payloads are shared owners.
        }
    }
    for policy in &value.merge_policy_declarations {
        control.checkpoint().map_err(Denial::Request)?;
        add(
            &mut bytes,
            size(policy.aspect_key.owned_allocation_capacity_bytes())?,
        )?;
    }
    Ok(bytes)
}
