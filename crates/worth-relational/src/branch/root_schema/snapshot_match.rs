//! Compare the original immutable schema to a snapshot without cloning its heaps.
use super::*;

pub(super) fn retain_revisions(
    expected: &SchemaAuthoritySnapshot,
    control: crate::schema::SchemaCompilationControl<'_, '_>,
) -> Result<
    Arc<[crate::schema::data::RelationIntegrityPlanRevision]>,
    crate::execution::RelationalExecutionDenialCause,
> {
    let mut revisions = Vec::with_capacity(expected.relation_kinds.len());
    for kind in &expected.relation_kinds {
        control.checkpoint()?;
        revisions.push(kind.relation_integrity_plan_revision);
    }
    Ok(revisions.into())
}

pub(super) fn matches(
    authority: &RelationalBranchRootSchemaAuthority,
    expected: &SchemaAuthoritySnapshot,
) -> bool {
    let registry = &authority.registry;
    let mut registrations = registry
        .entity_kinds
        .values()
        .map(|kind| (&kind.schema_id, kind.schema_version_id))
        .chain(
            registry
                .relation_kinds
                .values()
                .map(|kind| (&kind.schema_id, kind.schema_version_id)),
        );
    let basis = registrations.next();
    let basis = if registrations.all(|candidate| Some(candidate) == basis) {
        basis
    } else {
        None
    };
    if expected.primary_schema_id.as_ref() != basis.map(|(id, _)| id)
        || expected.primary_schema_version_id != basis.map(|(_, version)| version)
        || registry.entity_kinds.len() != expected.entity_kinds.len()
        || registry.relation_kinds.len() != expected.relation_kinds.len()
        || authority.relation_integrity_revisions.len() != expected.relation_kinds.len()
    {
        return false;
    }
    let entities_match = registry
        .entity_kinds
        .iter()
        .zip(&expected.entity_kinds)
        .all(|((id, actual), expected)| {
            *id == expected.kind_id
                && actual.kind_name == expected.kind_name
                && actual.schema_id == expected.schema_id
                && actual.schema_version_id == expected.schema_version_id
                && actual.aspect_contract_declarations.plan_revision
                    == expected.aspect_plan_revision
        });
    let relations_match = registry
        .relation_kinds
        .iter()
        .zip(authority.relation_integrity_revisions.iter())
        .zip(&expected.relation_kinds)
        .all(|(((id, actual), revision), expected)| {
            *id == expected.kind_id
                && actual.kind_name == expected.kind_name
                && actual.schema_id == expected.schema_id
                && actual.schema_version_id == expected.schema_version_id
                && actual.aspect_contract_declarations.plan_revision
                    == expected.aspect_plan_revision
                && *revision == expected.relation_integrity_plan_revision
                && actual.cross_context_policy == expected.cross_context_policy
                && actual.cascade_delete_policy == expected.cascade_delete_policy
        });
    entities_match
        && relations_match
        && registry
            .derived_relations
            .values()
            .map(|entry| &entry.parts)
            .eq(expected.derived_relations.iter())
        && registry
            .projected_member_domains
            .values()
            .map(|entry| &entry.parts)
            .eq(expected.projected_member_domains.iter())
}
