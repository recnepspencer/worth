use worth_foundational::facade::CanonicalDigestDerivationDenial;

use crate::canonical_digest_derivation::InstallationCanonicalIdentityBasis;
use crate::domain_operation::{
    WorthQueryOperationGraphAccess, WorthQueryOperationGraphParticipation,
    WorthQueryOperationGraphReadScope, WorthQueryOperationTouchContract,
    WorthQueryOperationTouchScope,
};

use super::super::{
    WorthQueryInstalledGraphObligationContract, WorthQueryInstalledGraphObligationResourcePosture,
};

pub(super) fn encode_contract(
    basis: &mut InstallationCanonicalIdentityBasis,
    index: usize,
    contract: &WorthQueryInstalledGraphObligationContract,
) -> Result<(), CanonicalDigestDerivationDenial> {
    let prefix = format!("obligation[{index}]");
    match contract {
        WorthQueryInstalledGraphObligationContract::QueryGraphRead { graph } => {
            basis.text(format!("{prefix}.kind"), "query-graph-read")?;
            basis.digest(format!("{prefix}.graph"), *graph.digest())?;
        }
        WorthQueryInstalledGraphObligationContract::OperationGraphRead { role } => {
            basis.text(format!("{prefix}.kind"), "operation-graph-read")?;
            basis.text(format!("{prefix}.role"), role.role())?;
            match role.participation() {
                WorthQueryOperationGraphParticipation::PrimaryLogicalGraph => {
                    basis.text(format!("{prefix}.participation"), "primary")?;
                }
                WorthQueryOperationGraphParticipation::SeparateAuthority { role } => {
                    basis.text(format!("{prefix}.participation"), "separate-authority")?;
                    basis.text(format!("{prefix}.authority-role"), role)?;
                }
            }
            basis.text(
                format!("{prefix}.access"),
                match role.access() {
                    WorthQueryOperationGraphAccess::Observe => "observe",
                    WorthQueryOperationGraphAccess::Project => "project",
                },
            )?;
            basis.unsigned_usize(
                format!("{prefix}.semantic-read-count"),
                role.read_scopes().len(),
            )?;
            for (read_index, read) in role.read_scopes().iter().enumerate() {
                encode_read_scope(basis, &format!("{prefix}.read[{read_index}]"), read)?;
            }
        }
        WorthQueryInstalledGraphObligationContract::PrincipalAuthorization => {
            basis.text(format!("{prefix}.kind"), "principal-authorization")?;
        }
        WorthQueryInstalledGraphObligationContract::AbilityAuthorization { requirements } => {
            basis.text(format!("{prefix}.kind"), "ability-authorization")?;
            basis.unsigned_usize(format!("{prefix}.count"), requirements.len())?;
            for (requirement_index, requirement) in requirements.iter().enumerate() {
                basis.digest(
                    format!("{prefix}.requirement[{requirement_index}]"),
                    *requirement.identity(),
                )?;
            }
        }
        WorthQueryInstalledGraphObligationContract::CapabilityAuthorization { requirements } => {
            basis.text(format!("{prefix}.kind"), "capability-authorization")?;
            basis.unsigned_usize(format!("{prefix}.count"), requirements.len())?;
            for (requirement_index, requirement) in requirements.iter().enumerate() {
                basis.digest(
                    format!("{prefix}.requirement[{requirement_index}]"),
                    *requirement.identity().digest(),
                )?;
            }
        }
        WorthQueryInstalledGraphObligationContract::MutationTouch { contract } => {
            basis.text(format!("{prefix}.kind"), "mutation-touch")?;
            encode_touch(basis, &prefix, contract)?;
        }
        WorthQueryInstalledGraphObligationContract::EffectApplication { family } => {
            basis.text(format!("{prefix}.kind"), "effect-application")?;
            basis.text(format!("{prefix}.family"), family.as_str())?;
        }
        WorthQueryInstalledGraphObligationContract::InvariantExecution { requirement } => {
            basis.text(format!("{prefix}.kind"), "invariant-execution")?;
            basis.text(format!("{prefix}.slot"), requirement.slot())?;
            basis.text(format!("{prefix}.family"), requirement.family())?;
            basis.unsigned_u32(format!("{prefix}.version"), requirement.version().get())?;
            basis.text(
                format!("{prefix}.enforcement"),
                requirement.enforcement().as_str(),
            )?;
            basis.text(
                format!("{prefix}.executor-role"),
                requirement.executor_role(),
            )?;
            basis.unsigned_usize(
                format!("{prefix}.state-load-count"),
                requirement.state_load_families().len(),
            )?;
            for (load_index, family) in requirement.state_load_families().iter().enumerate() {
                basis.text(format!("{prefix}.state-load[{load_index}]"), family)?;
            }
            match (requirement.max_state_facts(), requirement.max_work_units()) {
                (Some(state), Some(work)) if state > 0 && work > 0 => {
                    basis.unsigned_usize(format!("{prefix}.maximum-state-facts"), state)?;
                    basis.unsigned_u64(format!("{prefix}.maximum-work-units"), work)?;
                }
                (state, work) => {
                    basis.text(format!("{prefix}.bounds-form"), "optional-v1")?;
                    basis.text(
                        format!("{prefix}.state-presence"),
                        if state.is_some() { "some" } else { "none" },
                    )?;
                    if let Some(state) = state {
                        basis.unsigned_usize(format!("{prefix}.maximum-state-facts"), state)?;
                    }
                    basis.text(
                        format!("{prefix}.work-presence"),
                        if work.is_some() { "some" } else { "none" },
                    )?;
                    if let Some(work) = work {
                        basis.unsigned_u64(format!("{prefix}.maximum-work-units"), work)?;
                    }
                }
            }
        }
    }
    Ok(())
}

pub(super) fn encode_resources(
    basis: &mut InstallationCanonicalIdentityBasis,
    resources: &WorthQueryInstalledGraphObligationResourcePosture,
) -> Result<(), CanonicalDigestDerivationDenial> {
    match resources {
        WorthQueryInstalledGraphObligationResourcePosture::ApplicationQuery {
            maximum_traversal_depth,
            maximum_result_count,
            maximum_authorization_facts,
        } => {
            basis.text("resources.kind", "application-query")?;
            basis.unsigned_usize(
                "resources.maximum-traversal-depth",
                *maximum_traversal_depth,
            )?;
            basis.unsigned_usize("resources.maximum-result-count", *maximum_result_count)?;
            basis.unsigned_usize(
                "resources.maximum-authorization-facts",
                *maximum_authorization_facts,
            )?;
        }
        WorthQueryInstalledGraphObligationResourcePosture::ApplicationOperation(contract) => {
            basis.text("resources.kind", "application-operation")?;
            basis.text("resources.contract", contract.canonical_identity())?;
        }
    }
    Ok(())
}

fn encode_touch(
    basis: &mut InstallationCanonicalIdentityBasis,
    prefix: &str,
    contract: &WorthQueryOperationTouchContract,
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
        encode_touch_scope(basis, &format!("{prefix}.scope[{index}]"), scope)?;
    }
    Ok(())
}

fn encode_read_scope(
    basis: &mut InstallationCanonicalIdentityBasis,
    prefix: &str,
    scope: &WorthQueryOperationGraphReadScope,
) -> Result<(), CanonicalDigestDerivationDenial> {
    match scope {
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
            basis.embedded_basis(
                &format!("{prefix}.contract"),
                projection.projection().canonical_contract_basis(),
            )?;
            basis.embedded_basis(
                &format!("{prefix}.mask"),
                projection.projection().canonical_mask_basis(),
            )?;
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

fn encode_touch_scope(
    basis: &mut InstallationCanonicalIdentityBasis,
    prefix: &str,
    scope: &WorthQueryOperationTouchScope,
) -> Result<(), CanonicalDigestDerivationDenial> {
    match scope {
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
            basis.embedded_basis(
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
