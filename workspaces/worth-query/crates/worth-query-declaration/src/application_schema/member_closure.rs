use std::collections::BTreeSet;

use crate::authentication::{
    WorthQueryExternalPrincipalIdentity, WorthQueryPrincipalMappingStatus,
};

use super::authorization_policy::{
    ApplicationAuthorizationPath, ApplicationAuthorizationPathEffect,
    ApplicationAuthorizationTraversalDirection,
};
use super::{ApplicationSchemaDeclarationDenial, ApplicationSchemaMember};

mod decision_read_budgets;
mod member_collections;
mod principal_binding;
mod target_closure;

use super::capability_member_closure::validate_application_capability_members;
use super::query_member_closure::validate_application_query_members;
use decision_read_budgets::validate_decision_fact_budgets;
use member_collections::{
    collect_abilities, collect_aspects, collect_effects, collect_entities, collect_fields,
    collect_operations, collect_policies, collect_principal_entities, collect_relations,
    collect_units,
};
use principal_binding::{
    PrincipalBindingClosureRequirements, PrincipalBindingEqualityPosture,
    PrincipalBindingFieldRequirement, PrincipalBindingRelationRequirement,
    PrincipalBindingWritePosture,
};

pub(super) fn validate_member_closure(
    members: &[ApplicationSchemaMember],
) -> Result<(), ApplicationSchemaDeclarationDenial> {
    let index = ClosureIndex::new(members);
    for member in members {
        index.validate(member)?;
    }
    super::mutation_description_validation::validate_dependencies(members)?;
    validate_application_query_members(members)?;
    validate_application_capability_members(members)?;
    validate_decision_fact_budgets(members, &index.operations)?;
    Ok(())
}

pub(super) struct ClosureIndex<'a> {
    members: &'a [ApplicationSchemaMember],
    entities: BTreeSet<&'a str>,
    aspects: BTreeSet<(&'a str, &'a str)>,
    units: BTreeSet<&'a str>,
    operations: BTreeSet<&'a str>,
    abilities: BTreeSet<(&'a str, &'a str)>,
    policies: BTreeSet<&'a str>,
    principal_entities: BTreeSet<&'a str>,
    fields: BTreeSet<(&'a str, &'a str, &'a str)>,
    relations: BTreeSet<(&'a str, &'a str, &'a str)>,
    effects: BTreeSet<&'a str>,
}

impl<'a> ClosureIndex<'a> {
    pub(super) fn new(members: &'a [ApplicationSchemaMember]) -> Self {
        Self {
            members,
            entities: collect_entities(members),
            aspects: collect_aspects(members),
            units: collect_units(members),
            operations: collect_operations(members),
            abilities: collect_abilities(members),
            policies: collect_policies(members),
            principal_entities: collect_principal_entities(members),
            fields: collect_fields(members),
            relations: collect_relations(members),
            effects: collect_effects(members),
        }
    }

    fn validate(
        &self,
        member: &ApplicationSchemaMember,
    ) -> Result<(), ApplicationSchemaDeclarationDenial> {
        match member {
            ApplicationSchemaMember::Aspect { entity, .. }
                if !self.entities.contains(entity.as_str()) =>
            {
                Err(ApplicationSchemaDeclarationDenial::MissingEntity)
            }
            ApplicationSchemaMember::Field { entity, aspect, .. }
                if !self.aspects.contains(&(entity.as_str(), aspect.as_str())) =>
            {
                Err(ApplicationSchemaDeclarationDenial::MissingAspect)
            }
            ApplicationSchemaMember::Field {
                unit: Some(unit),
                ..
            } if !self.units.contains(unit.as_str()) => {
                Err(ApplicationSchemaDeclarationDenial::MissingUnit)
            }
            ApplicationSchemaMember::Relation { from, to, .. }
                if !self.entities.contains(from.as_str())
                    || !self.entities.contains(to.as_str()) =>
            {
                Err(ApplicationSchemaDeclarationDenial::MissingEntity)
            }
            ApplicationSchemaMember::PrincipalBinding {
                mapping_entity,
                identity_aspect,
                identity_field,
                status_aspect,
                status_field,
                target_relation,
                principal_entity,
                principal_identity_aspect,
                principal_identity_field,
                principal_identity_scalar_family,
                principal_identity_value_type,
                ..
            } if !self.principal_binding_dependencies_exist(PrincipalBindingClosureRequirements {
                mapping_identity: PrincipalBindingFieldRequirement {
                    entity: mapping_entity,
                    aspect: identity_aspect,
                    field: identity_field,
                    scalar_family: worth_foundational::facade::ScalarAspectType::String,
                    value_type: <WorthQueryExternalPrincipalIdentity as crate::portable_identity::WorthQueryPortableType>::PORTABLE_TYPE_IDENTITY.as_str(),
                    write: PrincipalBindingWritePosture::ReadOnly,
                    equality: PrincipalBindingEqualityPosture::Required,
                },
                mapping_status: PrincipalBindingFieldRequirement {
                    entity: mapping_entity,
                    aspect: status_aspect,
                    field: status_field,
                    scalar_family: worth_foundational::facade::ScalarAspectType::Bool,
                    value_type: <WorthQueryPrincipalMappingStatus as crate::portable_identity::WorthQueryPortableType>::PORTABLE_TYPE_IDENTITY.as_str(),
                    write: PrincipalBindingWritePosture::Writable,
                    equality: PrincipalBindingEqualityPosture::Unconstrained,
                },
                target: PrincipalBindingRelationRequirement {
                    relation: target_relation,
                    from: mapping_entity,
                    to: principal_entity,
                },
                principal_identity: PrincipalBindingFieldRequirement {
                    entity: principal_entity,
                    aspect: principal_identity_aspect,
                    field: principal_identity_field,
                    scalar_family: *principal_identity_scalar_family,
                    value_type: principal_identity_value_type,
                    write: PrincipalBindingWritePosture::ReadOnly,
                    equality: PrincipalBindingEqualityPosture::Required,
                },
            }) =>
            {
                Err(ApplicationSchemaDeclarationDenial::MissingPrincipalBindingDependency)
            }
            ApplicationSchemaMember::OperationProgram { operation, target }
                if !self.operations.contains(operation.as_str())
                    || !self.program_target_exists(target) =>
            {
                Err(ApplicationSchemaDeclarationDenial::MissingOperationProgramDependency)
            }
            ApplicationSchemaMember::OperationDecisionRead { operation, target }
                if !self.operations.contains(operation.as_str())
                    || !self.decision_read_target_exists(target) =>
            {
                Err(ApplicationSchemaDeclarationDenial::MissingOperationDecisionReadDependency)
            }
            ApplicationSchemaMember::OperationMutationPrecondition { operation, target }
                if !self.operations.contains(operation.as_str())
                    || !self.precondition_target_is_decision_read(operation, target) =>
            {
                Err(
                    ApplicationSchemaDeclarationDenial::MissingOperationMutationPreconditionDependency,
                )
            }
            ApplicationSchemaMember::OperationDecisionFactBudget { operation, .. }
                if !self.operations.contains(operation.as_str()) =>
            {
                Err(ApplicationSchemaDeclarationDenial::MissingOperationDecisionReadDependency)
            }
            ApplicationSchemaMember::OperationProjectionWorkBudget { operation, .. }
                if !self.operations.contains(operation.as_str()) =>
            {
                Err(ApplicationSchemaDeclarationDenial::MissingOperationDecisionReadDependency)
            }
            ApplicationSchemaMember::OperationExternalEffect {
                operation,
                effect,
                rust_payload_type,
                maximum_payload_bytes,
                ..
            } if *maximum_payload_bytes == 0
                || !self.external_effect_dependencies_exist(operation, effect, rust_payload_type) =>
            {
                Err(ApplicationSchemaDeclarationDenial::MissingOperationProgramDependency)
            }
            ApplicationSchemaMember::OperationAftermath { operation, .. }
                if !self.operations.contains(operation.as_str()) =>
            {
                Err(ApplicationSchemaDeclarationDenial::MissingOperationDecisionReadDependency)
            }
            ApplicationSchemaMember::Ability { scope_entity, .. }
                if !self.entities.contains(scope_entity.as_str()) =>
            {
                Err(ApplicationSchemaDeclarationDenial::MissingAbilityDependency)
            }
            ApplicationSchemaMember::OperationAbility {
                operation,
                ability,
                scope_entity,
            } if !self.operations.contains(operation.as_str())
                || !self
                    .abilities
                    .contains(&(ability.as_str(), scope_entity.as_str())) =>
            {
                Err(ApplicationSchemaDeclarationDenial::MissingAbilityDependency)
            }
            ApplicationSchemaMember::AbilityPolicy {
                ability,
                scope_entity,
                policy,
                paths,
            } if !self
                .abilities
                .contains(&(ability.as_str(), scope_entity.as_str()))
                || !self.policies.contains(policy.as_str()) =>
            {
                Err(ApplicationSchemaDeclarationDenial::MissingAbilityPolicyDependency)
            }
            ApplicationSchemaMember::AbilityPolicy {
                scope_entity,
                paths,
                ..
            } if !self.authorization_paths_are_closed(scope_entity, paths) => {
                Err(ApplicationSchemaDeclarationDenial::InvalidAbilityPolicy)
            }
            _ => Ok(()),
        }
    }

    fn authorization_paths_are_closed(
        &self,
        scope_entity: &str,
        paths: &[ApplicationAuthorizationPath],
    ) -> bool {
        !paths.is_empty()
            && paths
                .iter()
                .any(|path| path.effect() == ApplicationAuthorizationPathEffect::Allow)
            && paths
                .iter()
                .all(|path| self.authorization_path_is_closed(scope_entity, path))
    }

    pub(super) fn authorization_path_is_closed(
        &self,
        scope_entity: &str,
        path: &ApplicationAuthorizationPath,
    ) -> bool {
        if path.scope_entity() != scope_entity
            || !self.principal_entities.contains(path.principal_entity())
        {
            return false;
        }
        let mut current = path.principal_entity();
        let mut entity_by_ordinal = vec![current];
        for traversal in path.traversals() {
            if !self
                .relations
                .contains(&(traversal.relation(), traversal.from(), traversal.to()))
            {
                return false;
            }
            current = match traversal.direction() {
                ApplicationAuthorizationTraversalDirection::Forward
                    if current == traversal.from() =>
                {
                    traversal.to()
                }
                ApplicationAuthorizationTraversalDirection::Reverse
                    if current == traversal.to() =>
                {
                    traversal.from()
                }
                _ => return false,
            };
            entity_by_ordinal.push(current);
        }
        current == scope_entity
            && path.predicates().iter().all(|predicate| {
                entity_by_ordinal
                    .get(predicate.traversal_ordinal())
                    .is_some_and(|entity| *entity == predicate.entity())
                    && self.fields.contains(&(
                        predicate.entity(),
                        predicate.aspect(),
                        predicate.field(),
                    ))
                    && self.field_is_equality_queryable(
                        predicate.entity(),
                        predicate.aspect(),
                        predicate.field(),
                    )
            })
    }

    fn field_is_equality_queryable(&self, entity: &str, aspect: &str, field: &str) -> bool {
        self.members.iter().any(|member| {
            matches!(
                member,
                ApplicationSchemaMember::Field {
                    entity: candidate_entity,
                    aspect: candidate_aspect,
                    field: candidate_field,
                    equality_queryable: true,
                    ..
                } if candidate_entity == entity
                    && candidate_aspect == aspect
                    && candidate_field == field
            )
        })
    }
}
