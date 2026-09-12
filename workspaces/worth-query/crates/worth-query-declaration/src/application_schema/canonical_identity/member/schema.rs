use crate::application_schema::canonical_basis::ApplicationSchemaCanonicalBasis;
use crate::application_schema::ApplicationSchemaMember;

use super::field::append_schema_field;
use super::principal_binding::append_principal_binding;

pub(super) fn append_schema_member(
    basis: &mut ApplicationSchemaCanonicalBasis,
    prefix: &str,
    member: &ApplicationSchemaMember,
) {
    match member {
        ApplicationSchemaMember::Entity { entity } => {
            basis.text(format!("{prefix}.kind"), "entity");
            basis.text(format!("{prefix}.entity"), entity);
        }
        ApplicationSchemaMember::Aspect {
            entity,
            aspect,
            identity,
            revision,
        } => {
            basis.text(format!("{prefix}.kind"), "aspect");
            basis.text(format!("{prefix}.entity"), entity);
            basis.text(format!("{prefix}.aspect"), aspect);
            basis.u64(format!("{prefix}.identity"), identity.0);
            basis.u64(format!("{prefix}.revision"), revision.0);
        }
        ApplicationSchemaMember::Field { .. } => append_schema_field(basis, prefix, member),
        ApplicationSchemaMember::Relation {
            relation,
            from,
            to,
            integrity,
        } => {
            basis.text(format!("{prefix}.kind"), "relation");
            basis.text(format!("{prefix}.relation"), relation);
            basis.text(format!("{prefix}.from"), from);
            basis.text(format!("{prefix}.to"), to);
            append_relation_integrity(basis, prefix, *integrity);
        }
        ApplicationSchemaMember::PrincipalBinding { .. } => {
            append_principal_binding(basis, prefix, member)
        }
        _ => unreachable!("schema member router supplied a non-schema member"),
    }
}

fn append_relation_integrity(
    basis: &mut ApplicationSchemaCanonicalBasis,
    prefix: &str,
    integrity: crate::application_schema::ApplicationRelationIntegrity,
) {
    use crate::application_schema::{
        ApplicationRelationCrossContextPolicy as CrossContext,
        ApplicationRelationDeletionPolicy as Deletion,
    };
    basis.text(
        format!("{prefix}.endpoints.cross_context"),
        match integrity.endpoints.cross_context_policy {
            CrossContext::AllowExplicit => "allow-explicit",
            CrossContext::SchemaControlled => "schema-controlled",
            CrossContext::Forbid => "forbid",
        },
    );
    basis.u64(
        format!("{prefix}.endpoints.self_edges_allowed"),
        u64::from(integrity.endpoints.self_edges_allowed),
    );
    for (name, value) in [
        ("source_min", integrity.cardinality.source_min),
        ("source_max", integrity.cardinality.source_max),
        ("target_min", integrity.cardinality.target_min),
        ("target_max", integrity.cardinality.target_max),
        ("pair_min", integrity.cardinality.pair_min),
        ("pair_max", integrity.cardinality.pair_max),
    ] {
        basis.text(
            format!("{prefix}.cardinality.{name}.presence"),
            if value.is_some() { "some" } else { "none" },
        );
        if let Some(value) = value {
            basis.u64(format!("{prefix}.cardinality.{name}.value"), value);
        }
    }
    basis.text(
        format!("{prefix}.deletion"),
        match integrity.deletion {
            Deletion::RetainDanglingForAudit => "retain-dangling-for-audit",
            Deletion::CascadeDeleteRelations => "cascade-delete-relations",
            Deletion::RejectDeleteWithLiveRelations => "reject-delete-with-live-relations",
            Deletion::RequireRelationDeletionInSameCommit => {
                "require-relation-deletion-in-same-commit"
            }
            Deletion::RequireRelationRetirement => "require-relation-retirement",
        },
    );
}
