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
        ApplicationSchemaMember::ComponentSet { definition } => {
            basis.text(format!("{prefix}.kind"), "component-set");
            basis.text(format!("{prefix}.name"), definition.name());
            basis.text(format!("{prefix}.member"), definition.member());
            basis.u64(
                format!("{prefix}.sources.len"),
                definition.relations().len() as u64,
            );
            for (index, relation) in definition.relations().iter().enumerate() {
                basis.text(format!("{prefix}.sources[{index}]"), relation);
            }
        }
        ApplicationSchemaMember::Entity { entity } => {
            basis.text(format!("{prefix}.kind"), "entity");
            basis.text(format!("{prefix}.entity"), entity);
        }
        ApplicationSchemaMember::Aspect {
            entity,
            aspect,
            identity,
            revision,
            absence,
        } => {
            basis.text(format!("{prefix}.kind"), "aspect");
            basis.text(format!("{prefix}.entity"), entity);
            basis.text(format!("{prefix}.aspect"), aspect);
            basis.u64(format!("{prefix}.identity"), identity.0);
            basis.u64(format!("{prefix}.revision"), revision.0);
            basis.text(
                format!("{prefix}.absence"),
                match absence {
                    worth_foundational::facade::AbsenceLaw::Required => "required",
                    worth_foundational::facade::AbsenceLaw::Optional => "optional",
                    worth_foundational::facade::AbsenceLaw::Defaulted => "defaulted",
                },
            );
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
            append_relation_integrity(basis, prefix, integrity);
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
    integrity: &crate::application_schema::ApplicationRelationIntegrityRecord,
) {
    use crate::application_schema::ApplicationRelationDeletionPolicy as Deletion;
    super::relation_graph::append(basis, prefix, integrity.graph_laws(), integrity.unique);
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
