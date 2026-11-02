use std::collections::BTreeSet;

use crate::merge::data::{IdentityBasisDeclaration, IdentityBasisKind, IdentityBasisScope};
use worth_foundational::{AspectShape, ReferenceAspectType, ScalarAspectType};

use crate::schema::data::{
    AspectBinding, DeclaredAspectContractBinding, KindAspectContractDeclarations,
    SchemaRegistryError,
};

use super::derive_kind_plan_revision;
use super::merge_policy_declarations::canonicalize_merge_policy_declarations;

pub(super) enum RegistrationDomain {
    Entity,
    Relation,
}

pub(super) fn canonicalize_kind_aspect_declarations(
    kind_id: crate::identity::data::KindId,
    declarations: KindAspectContractDeclarations,
    domain: RegistrationDomain,
) -> Result<KindAspectContractDeclarations, SchemaRegistryError> {
    let mut seen = BTreeSet::new();
    let mut aspects = declarations.aspects;
    let mut identity_declarations = canonicalize_identity_declarations(
        kind_id,
        declarations.identity_declarations,
        &aspects,
        &domain,
    )?;
    let merge_policy_declarations = canonicalize_merge_policy_declarations(
        kind_id,
        declarations.merge_policy_declarations,
        &aspects,
    )?;
    aspects.sort_by_key(DeclaredAspectContractBinding::aspect_key);
    for aspect in &aspects {
        if !seen.insert(aspect.aspect_key()) {
            return Err(SchemaRegistryError::duplicate_aspect_key(
                kind_id,
                aspect.aspect_key(),
            ));
        }
        validate_declared_aspect(kind_id, aspect, &domain)?;
    }
    identity_declarations.sort();
    let plan_revision = derive_kind_plan_revision(
        kind_id,
        &aspects,
        &identity_declarations,
        &merge_policy_declarations,
    )?;
    Ok(KindAspectContractDeclarations {
        plan_revision,
        aspects,
        identity_declarations,
        merge_policy_declarations,
    })
}

fn canonicalize_identity_declarations(
    kind_id: crate::identity::data::KindId,
    mut declarations: Vec<IdentityBasisDeclaration>,
    aspects: &[DeclaredAspectContractBinding],
    domain: &RegistrationDomain,
) -> Result<Vec<IdentityBasisDeclaration>, SchemaRegistryError> {
    if declarations.is_empty() {
        declarations = default_identity_declarations(kind_id, domain);
    }
    declarations.sort();
    let mut seen = BTreeSet::new();
    let aspect_keys = aspects
        .iter()
        .map(DeclaredAspectContractBinding::aspect_key)
        .collect::<BTreeSet<_>>();
    for declaration in &declarations {
        if !seen.insert(declaration.clone()) {
            return Err(SchemaRegistryError::invalid_aspect_declaration(
                kind_id,
                format!(
                    "duplicate identity basis declaration for scope {:?}",
                    declaration.scope
                ),
            ));
        }
        match &declaration.scope {
            IdentityBasisScope::EntityKind(scope_kind) => {
                if *scope_kind != kind_id || !matches!(domain, RegistrationDomain::Entity) {
                    return Err(SchemaRegistryError::invalid_aspect_declaration(
                        kind_id,
                        "entity identity declarations must target the registering entity kind",
                    ));
                }
            }
            IdentityBasisScope::RelationKind(scope_kind) => {
                if *scope_kind != kind_id || !matches!(domain, RegistrationDomain::Relation) {
                    return Err(SchemaRegistryError::invalid_aspect_declaration(
                        kind_id,
                        "relation identity declarations must target the registering relation kind",
                    ));
                }
            }
            IdentityBasisScope::AspectKey(aspect_key) => {
                if !aspect_keys.contains(aspect_key) {
                    return Err(SchemaRegistryError::invalid_aspect_declaration(
                        kind_id,
                        format!(
                            "identity basis declaration references undeclared aspect key {:?}",
                            aspect_key
                        ),
                    ));
                }
                if !matches!(declaration.basis, IdentityBasisKind::DeclaredKeySet(_)) {
                    return Err(SchemaRegistryError::invalid_aspect_declaration(
                        kind_id,
                        "aspect-key-scoped identity declarations must use DeclaredKeySet",
                    ));
                }
            }
        }
        if matches!(domain, RegistrationDomain::Relation)
            && matches!(declaration.basis, IdentityBasisKind::LineageIdentity)
        {
            return Err(SchemaRegistryError::invalid_aspect_declaration(
                kind_id,
                "relation kinds cannot currently declare LineageIdentity",
            ));
        }
        if let IdentityBasisKind::DeclaredKeySet(keys) = &declaration.basis {
            if keys.is_empty() {
                return Err(SchemaRegistryError::invalid_aspect_declaration(
                    kind_id,
                    "DeclaredKeySet identity declarations must contain at least one aspect key",
                ));
            }
            for key in keys.iter() {
                if !aspect_keys.contains(key) {
                    return Err(SchemaRegistryError::invalid_aspect_declaration(
                        kind_id,
                        format!(
                            "DeclaredKeySet identity declaration references undeclared aspect key {:?}",
                            key
                        ),
                    ));
                }
            }
        }
        match &declaration.basis {
            IdentityBasisKind::StorageIdentity
            | IdentityBasisKind::LineageIdentity
            | IdentityBasisKind::DeclaredKeySet(_) => {}
            IdentityBasisKind::StructuralFingerprint => {
                return Err(SchemaRegistryError::invalid_aspect_declaration(
                    kind_id,
                    "merge planning does not yet support StructuralFingerprint identity declarations",
                ));
            }
            IdentityBasisKind::Custom(_) => {
                return Err(SchemaRegistryError::invalid_aspect_declaration(
                    kind_id,
                    "merge planning does not yet support custom identity declarations",
                ));
            }
        }
    }
    Ok(declarations)
}

fn default_identity_declarations(
    kind_id: crate::identity::data::KindId,
    domain: &RegistrationDomain,
) -> Vec<IdentityBasisDeclaration> {
    match domain {
        RegistrationDomain::Entity => vec![
            IdentityBasisDeclaration {
                scope: IdentityBasisScope::EntityKind(kind_id),
                basis: IdentityBasisKind::StorageIdentity,
            },
            IdentityBasisDeclaration {
                scope: IdentityBasisScope::EntityKind(kind_id),
                basis: IdentityBasisKind::LineageIdentity,
            },
        ],
        RegistrationDomain::Relation => vec![IdentityBasisDeclaration {
            scope: IdentityBasisScope::RelationKind(kind_id),
            basis: IdentityBasisKind::StorageIdentity,
        }],
    }
}

fn validate_declared_aspect(
    kind_id: crate::identity::data::KindId,
    aspect: &DeclaredAspectContractBinding,
    domain: &RegistrationDomain,
) -> Result<(), SchemaRegistryError> {
    validate_declared_aspect_key(kind_id, "aspect key", aspect)?;
    if matches!(
        aspect.contract.shape(),
        AspectShape::Content | AspectShape::Scalar(ScalarAspectType::ContentRef)
    ) {
        return Err(SchemaRegistryError::invalid_aspect_declaration(
            kind_id,
            "content references require a format-bound struct field",
        ));
    }
    match &aspect.binding {
        AspectBinding::EntityField { .. } => {
            if !matches!(domain, RegistrationDomain::Entity) {
                return Err(SchemaRegistryError::invalid_aspect_declaration(
                    kind_id,
                    "entity field aspect bindings are valid only for entity kinds",
                ));
            }
            if !matches!(
                aspect.contract.shape(),
                AspectShape::Scalar(_) | AspectShape::Struct(_)
            ) {
                return Err(SchemaRegistryError::invalid_aspect_declaration(
                    kind_id,
                    "entity field aspects require scalar or struct foundational contracts",
                ));
            }
        }
        AspectBinding::RelationField { .. } => {
            let RegistrationDomain::Relation = domain else {
                return Err(SchemaRegistryError::invalid_aspect_declaration(
                    kind_id,
                    "relation field aspect bindings are valid only for relation kinds",
                ));
            };
            if !matches!(
                aspect.contract.shape(),
                AspectShape::Scalar(_) | AspectShape::Struct(_)
            ) {
                return Err(SchemaRegistryError::invalid_aspect_declaration(
                    kind_id,
                    "relation field aspects require scalar or struct foundational contracts",
                ));
            }
        }
        AspectBinding::RelationSourceEndpoint | AspectBinding::RelationTargetEndpoint => {
            if !matches!(domain, RegistrationDomain::Relation) {
                return Err(SchemaRegistryError::invalid_aspect_declaration(
                    kind_id,
                    "relation endpoint bindings are valid only for relation kinds",
                ));
            }
            if !matches!(
                aspect.contract.shape(),
                AspectShape::Reference(ReferenceAspectType::Entity)
            ) {
                return Err(SchemaRegistryError::invalid_aspect_declaration(
                    kind_id,
                    "relation endpoint bindings require entity-reference foundational contracts",
                ));
            }
        }
        AspectBinding::LifecycleTransition => {
            if !matches!(
                aspect.contract.shape(),
                AspectShape::Scalar(ScalarAspectType::String)
            ) {
                return Err(SchemaRegistryError::invalid_aspect_declaration(
                    kind_id,
                    "lifecycle bindings require scalar string foundational contracts",
                ));
            }
        }
        AspectBinding::StructuralRegion
        | AspectBinding::StructuralPartition
        | AspectBinding::StructuralFacet => {
            if !matches!(
                aspect.contract.shape(),
                AspectShape::Scalar(ScalarAspectType::String)
            ) {
                return Err(SchemaRegistryError::invalid_aspect_declaration(
                    kind_id,
                    "structural bindings require scalar string foundational contracts",
                ));
            }
        }
        _ => {
            return Err(SchemaRegistryError::invalid_aspect_declaration(
                kind_id,
                "unsupported authoritative aspect binding",
            ));
        }
    }
    Ok(())
}

fn validate_declared_aspect_key(
    kind_id: crate::identity::data::KindId,
    contract_key_label: &str,
    aspect: &DeclaredAspectContractBinding,
) -> Result<(), SchemaRegistryError> {
    let declared_key = aspect.aspect_key();
    let foundational_key = aspect.foundational_key().as_str();
    if foundational_key.trim().is_empty() {
        return Err(SchemaRegistryError::invalid_aspect_declaration(
            kind_id,
            format!("{contract_key_label} must not be empty"),
        ));
    }
    if declared_key.as_str() != foundational_key {
        return Err(SchemaRegistryError::invalid_aspect_declaration(
            kind_id,
            "declared aspect contract key must be the foundational aspect key",
        ));
    }
    Ok(())
}
