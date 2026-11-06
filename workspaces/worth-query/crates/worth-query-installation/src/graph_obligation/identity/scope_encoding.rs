//! Exact scope encoding; shared aspect contracts enter through one identity-local table.
use super::aspect_contract_table::AspectContractTable;
use crate::canonical_digest_derivation::InstallationCanonicalIdentityBasis;
use crate::domain_operation::{
    WorthQueryOperationGraphReadScope, WorthQueryOperationTouchContract,
    WorthQueryOperationTouchScope,
};
use worth_foundational::facade::CanonicalDigestDerivationDenial;

pub(super) fn encode_touch<'a>(
    basis: &mut InstallationCanonicalIdentityBasis,
    table: &mut AspectContractTable<'a>,
    prefix: &str,
    contract: &'a WorthQueryOperationTouchContract,
) -> Result<(), CanonicalDigestDerivationDenial> {
    let WorthQueryOperationTouchContract::Declared {
        graph_roles,
        scopes,
    } = contract
    else {
        return Ok(());
    };
    basis.unsigned_usize(format!("{prefix}.graph-role-count"), graph_roles.len())?;
    for (index, role) in graph_roles.iter().enumerate() {
        basis.text(format!("{prefix}.graph-role[{index}]"), role)?;
    }
    basis.unsigned_usize(format!("{prefix}.scope-count"), scopes.len())?;
    for (index, scope) in scopes.iter().enumerate() {
        encode_touch_scope(basis, table, &format!("{prefix}.scope[{index}]"), scope)?;
    }
    Ok(())
}

pub(super) fn encode_read_scope<'a>(
    basis: &mut InstallationCanonicalIdentityBasis,
    table: &mut AspectContractTable<'a>,
    prefix: &str,
    scope: &'a WorthQueryOperationGraphReadScope,
) -> Result<(), CanonicalDigestDerivationDenial> {
    match scope {
        WorthQueryOperationGraphReadScope::DerivedSource(scope) => {
            basis.text(format!("{prefix}.kind"), "derived-source-v1")?;
            encode_binding(basis, prefix, scope.schema())?;
            basis.text(format!("{prefix}.relation"), scope.relation())?;
        }
        WorthQueryOperationGraphReadScope::Components(scope) => {
            basis.text(format!("{prefix}.kind"), "components")?;
            encode_binding(basis, prefix, scope.schema())?;
            basis.text(
                format!("{prefix}.definition"),
                scope.meaning().canonical_record(),
            )?;
        }
        WorthQueryOperationGraphReadScope::Entity(entity) => {
            basis.text(format!("{prefix}.kind"), "entity")?;
            encode_binding(basis, prefix, entity.schema())?;
            basis.text(format!("{prefix}.entity"), entity.semantic_key())?;
        }
        WorthQueryOperationGraphReadScope::NativeProjection(projection) => {
            basis.text(format!("{prefix}.kind"), "native-projection")?;
            encode_binding(basis, prefix, projection.schema())?;
            basis.text(
                format!("{prefix}.entity"),
                projection.entity().semantic_key(),
            )?;
            basis.text(format!("{prefix}.aspect"), projection.aspect().as_str())?;
            table.encode_reference(
                basis,
                &format!("{prefix}.contract"),
                projection.projection().canonical_contract_basis(),
            )?;
            super::mask_encoding::encode_mask(basis, prefix, projection.projection())?;
        }
        WorthQueryOperationGraphReadScope::Relation(relation) => {
            basis.text(format!("{prefix}.kind"), "relation")?;
            encode_binding(basis, prefix, relation.schema())?;
            basis.text(format!("{prefix}.relation"), relation.relation())?;
            basis.text(format!("{prefix}.from"), relation.from())?;
            basis.text(format!("{prefix}.to"), relation.to())?;
        }
    }
    Ok(())
}

fn encode_touch_scope<'a>(
    basis: &mut InstallationCanonicalIdentityBasis,
    table: &mut AspectContractTable<'a>,
    prefix: &str,
    scope: &'a WorthQueryOperationTouchScope,
) -> Result<(), CanonicalDigestDerivationDenial> {
    match scope {
        WorthQueryOperationTouchScope::RemoveAspect(scope) => {
            basis.text(format!("{prefix}.kind"), "remove-aspect")?;
            encode_binding(basis, prefix, scope.schema())?;
            basis.text(format!("{prefix}.entity"), scope.entity())?;
            table.encode_reference(
                basis,
                &format!("{prefix}.contract"),
                scope.canonical_contract_basis(),
            )?;
        }
        WorthQueryOperationTouchScope::InitializeRelationField(field) => {
            basis.text(format!("{prefix}.kind"), "initialize-relation-field")?;
            let relation = field.relation();
            encode_binding(basis, prefix, relation.schema())?;
            basis.text(format!("{prefix}.relation"), relation.relation())?;
            basis.text(format!("{prefix}.from"), relation.from())?;
            basis.text(format!("{prefix}.to"), relation.to())?;
            table.encode_reference(
                basis,
                &format!("{prefix}.contract"),
                field.canonical_contract_basis(),
            )?;
            for (index, part) in field.field_path().fields().iter().enumerate() {
                basis.text(format!("{prefix}.field[{index}]"), part.as_str())?;
            }
        }
        WorthQueryOperationTouchScope::CreateEntity(entity)
        | WorthQueryOperationTouchScope::DeleteEntity(entity) => {
            let kind = if matches!(scope, WorthQueryOperationTouchScope::CreateEntity(_)) {
                "create-entity"
            } else {
                "delete-entity"
            };
            basis.text(format!("{prefix}.kind"), kind)?;
            encode_binding(basis, prefix, entity.schema())?;
            basis.text(format!("{prefix}.entity"), entity.entity())?;
        }
        WorthQueryOperationTouchScope::WriteField(field) => {
            basis.text(format!("{prefix}.kind"), "write-field")?;
            encode_binding(basis, prefix, field.schema())?;
            basis.text(format!("{prefix}.entity"), field.entity())?;
            table.encode_reference(
                basis,
                &format!("{prefix}.contract"),
                field.canonical_contract_basis(),
            )?;
            for (index, part) in field.field_path().fields().iter().enumerate() {
                basis.text(format!("{prefix}.field[{index}]"), part.as_str())?;
            }
        }
        WorthQueryOperationTouchScope::LinkRelation(relation)
        | WorthQueryOperationTouchScope::UnlinkRelation(relation) => {
            let kind = if matches!(scope, WorthQueryOperationTouchScope::LinkRelation(_)) {
                "link-relation"
            } else {
                "unlink-relation"
            };
            basis.text(format!("{prefix}.kind"), kind)?;
            encode_binding(basis, prefix, relation.schema())?;
            basis.text(format!("{prefix}.relation"), relation.relation())?;
            basis.text(format!("{prefix}.from"), relation.from())?;
            basis.text(format!("{prefix}.to"), relation.to())?;
        }
        WorthQueryOperationTouchScope::DeclaredDomain(identity) => {
            basis.text(format!("{prefix}.kind"), "declared-domain")?;
            basis.text(format!("{prefix}.identity"), identity.as_str())?;
        }
    }
    Ok(())
}

fn encode_binding(
    basis: &mut InstallationCanonicalIdentityBasis,
    prefix: &str,
    binding: &worth_query_declaration::facade::application_schema::ApplicationSchemaBindingIdentity,
) -> Result<(), CanonicalDigestDerivationDenial> {
    basis.unsigned_u64(format!("{prefix}.runtime"), binding.runtime_ordinal())?;
    basis.unsigned_u64(format!("{prefix}.generation"), binding.generation())?;
    basis.digest(format!("{prefix}.package"), *binding.package_identity())?;
    basis.digest(format!("{prefix}.schema"), *binding.schema_identity())?;
    Ok(())
}
