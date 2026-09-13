use std::collections::BTreeSet;

use worth_foundational::facade::ScalarAspectType;

use crate::application_query::{
    ApplicationQueryAuthorizationRequirement, ApplicationQueryResultShape,
    ApplicationQueryResultTraversalDirection, ErasedApplicationQueryDefinition,
};

use super::{ApplicationSchemaDeclarationDenial, ApplicationSchemaMember};

mod disclosure;

pub(super) fn validate_application_query_members(
    members: &[ApplicationSchemaMember],
) -> Result<(), ApplicationSchemaDeclarationDenial> {
    let mut names = BTreeSet::new();
    for definition in members.iter().filter_map(query_definition) {
        if !names.insert(definition.name()) {
            return Err(ApplicationSchemaDeclarationDenial::DuplicateApplicationQuery);
        }
        validate_query(definition, members)?;
    }
    Ok(())
}

fn query_definition(member: &ApplicationSchemaMember) -> Option<&ErasedApplicationQueryDefinition> {
    match member {
        ApplicationSchemaMember::ApplicationQuery { definition } => Some(definition),
        _ => None,
    }
}

fn validate_query(
    definition: &ErasedApplicationQueryDefinition,
    members: &[ApplicationSchemaMember],
) -> Result<(), ApplicationSchemaDeclarationDenial> {
    if !entity_exists(members, definition.root_entity())
        || !entity_exists(members, definition.scope_entity())
    {
        return Err(ApplicationSchemaDeclarationDenial::MissingApplicationQueryDependency);
    }
    validate_authorization_requirement(definition, members)?;
    disclosure::validate_dependencies(definition, members)?;
    validate_root_selection(definition, members)?;
    validate_result_shape(definition, members)?;
    validate_predicates(definition, members)?;
    validate_ordering(definition, members)
}

fn validate_root_selection(
    definition: &ErasedApplicationQueryDefinition,
    members: &[ApplicationSchemaMember],
) -> Result<(), ApplicationSchemaDeclarationDenial> {
    if definition.root_paths().iter().any(|path| {
        path.start_entity() != definition.scope_entity()
            || path.terminal_entity() != definition.root_entity()
            || path.steps().is_empty()
            || path
                .steps()
                .iter()
                .any(|step| !relation_exists(members, step.relation(), step.from(), step.to()))
            || path.guards().iter().any(|guard| {
                !field_matches(
                    members,
                    guard.entity(),
                    guard.aspect(),
                    guard.field(),
                    guard.scalar_family(),
                    Some(guard.value_type()),
                    true,
                )
            })
    }) {
        return Err(ApplicationSchemaDeclarationDenial::InvalidApplicationQuery);
    }
    Ok(())
}

fn validate_result_shape(
    definition: &ErasedApplicationQueryDefinition,
    members: &[ApplicationSchemaMember],
) -> Result<(), ApplicationSchemaDeclarationDenial> {
    if definition.result_shape().result_type() != definition.result_type()
        || definition.result_shape().query_type() != definition.query_type()
        || definition.result_shape().root_entity() != definition.root_entity()
        || !shape_is_closed(members, definition.query_type(), definition.result_shape())
    {
        return Err(ApplicationSchemaDeclarationDenial::InvalidApplicationQuery);
    }
    Ok(())
}

fn validate_predicates(
    definition: &ErasedApplicationQueryDefinition,
    members: &[ApplicationSchemaMember],
) -> Result<(), ApplicationSchemaDeclarationDenial> {
    for predicate in definition.predicates() {
        let (entity, aspect, field) = predicate.field();
        if !field_matches(
            members,
            entity,
            aspect,
            field,
            predicate.scalar_family(),
            None,
            true,
        ) || !definition.parameters().iter().any(|parameter| {
            parameter.name() == predicate.parameter()
                && parameter.scalar_family() == predicate.scalar_family()
        }) {
            return Err(ApplicationSchemaDeclarationDenial::InvalidApplicationQuery);
        }
    }
    Ok(())
}

fn validate_ordering(
    definition: &ErasedApplicationQueryDefinition,
    members: &[ApplicationSchemaMember],
) -> Result<(), ApplicationSchemaDeclarationDenial> {
    for ordering in definition.ordering() {
        let (entity, aspect, field) = ordering.field();
        if !field_matches(
            members,
            entity,
            aspect,
            field,
            ordering.scalar_family(),
            None,
            false,
        ) {
            return Err(ApplicationSchemaDeclarationDenial::InvalidApplicationQuery);
        }
    }
    Ok(())
}

fn validate_authorization_requirement(
    definition: &ErasedApplicationQueryDefinition,
    members: &[ApplicationSchemaMember],
) -> Result<(), ApplicationSchemaDeclarationDenial> {
    let ApplicationQueryAuthorizationRequirement::Ability {
        ability,
        scope_entity,
    } = definition.authorization()
    else {
        return Ok(());
    };
    if *scope_entity != definition.scope_entity()
        || !members.iter().any(|member| {
            matches!(
                member,
                ApplicationSchemaMember::Ability {
                    ability: installed,
                    scope_entity: installed_scope,
                } if installed == ability && installed_scope == scope_entity
            )
        })
    {
        return Err(ApplicationSchemaDeclarationDenial::MissingAbilityDependency);
    }
    if members.iter().any(|member| {
        matches!(
            member,
            ApplicationSchemaMember::AbilityPolicy {
                ability: installed,
                scope_entity: installed_scope,
                ..
            } if installed == ability && installed_scope == scope_entity
        )
    }) {
        Ok(())
    } else {
        Err(ApplicationSchemaDeclarationDenial::MissingAbilityPolicyDependency)
    }
}

fn shape_is_closed(
    members: &[ApplicationSchemaMember],
    query_type: &str,
    shape: &ApplicationQueryResultShape,
) -> bool {
    shape.query_type() == query_type
        && entity_exists(members, shape.root_entity())
        && shape.fields().iter().all(|field| {
            field.query_type() == query_type
                && field.entity() == shape.root_entity()
                && result_field_matches(members, field)
        })
        && shape.relations().iter().all(|relation| {
            relation.query_type() == query_type
                && relation_shape_endpoints_match(relation, shape)
                && relation_exists(members, relation.relation(), relation.from(), relation.to())
                && shape_is_closed(members, query_type, relation.nested_shape())
        })
}

fn result_field_matches(
    members: &[ApplicationSchemaMember],
    expected: &crate::application_query::ApplicationQueryResultField,
) -> bool {
    members.iter().any(|member| {
        matches!(
            member,
            ApplicationSchemaMember::Field {
                entity,
                aspect,
                field,
                scalar_family,
                value_type,
                presence,
                ..
            } if entity == expected.entity()
                && aspect == expected.aspect()
                && field == expected.field()
                && *scalar_family == expected.scalar_family()
                && value_type == expected.value_type()
                && *presence == expected.presence()
        )
    })
}

fn relation_shape_endpoints_match(
    relation: &crate::application_query::ApplicationQueryResultRelation,
    parent: &ApplicationQueryResultShape,
) -> bool {
    let child = relation.nested_shape();
    match relation.direction() {
        ApplicationQueryResultTraversalDirection::Forward => {
            relation.from() == parent.root_entity() && relation.to() == child.root_entity()
        }
        ApplicationQueryResultTraversalDirection::Reverse => {
            relation.to() == parent.root_entity() && relation.from() == child.root_entity()
        }
    }
}

fn entity_exists(members: &[ApplicationSchemaMember], expected: &str) -> bool {
    members.iter().any(
        |member| matches!(member, ApplicationSchemaMember::Entity { entity } if entity == expected),
    )
}

#[allow(clippy::too_many_arguments)]
fn field_matches(
    members: &[ApplicationSchemaMember],
    expected_entity: &str,
    expected_aspect: &str,
    expected_field: &str,
    expected_scalar: ScalarAspectType,
    expected_value_type: Option<&str>,
    equality_required: bool,
) -> bool {
    members.iter().any(|member| {
        matches!(
            member,
            ApplicationSchemaMember::Field {
                entity,
                aspect,
                field,
                scalar_family,
                value_type,
                equality_queryable,
                ..
            } if entity == expected_entity
                && aspect == expected_aspect
                && field == expected_field
                && *scalar_family == expected_scalar
                && expected_value_type.is_none_or(|expected| value_type == expected)
                && (!equality_required || *equality_queryable)
        )
    })
}

fn relation_exists(
    members: &[ApplicationSchemaMember],
    expected_relation: &str,
    expected_from: &str,
    expected_to: &str,
) -> bool {
    members.iter().any(|member| {
        matches!(
            member,
            ApplicationSchemaMember::Relation { relation, from, to, .. }
                if relation == expected_relation
                    && from == expected_from
                    && to == expected_to
        )
    })
}

#[cfg(test)]
#[path = "query_member_closure_tests.rs"]
mod tests;
