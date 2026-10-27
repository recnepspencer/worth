use crate::application_query::{
    ApplicationQueryAuthorizationRequirement, ApplicationQueryResultShape,
    ErasedApplicationQueryDefinition,
};

use super::{validate_simple_identifier, ApplicationSchemaDeclarationDenial};

pub(super) fn validate(
    definition: &ErasedApplicationQueryDefinition,
) -> Result<(), ApplicationSchemaDeclarationDenial> {
    validate_identifiers([
        definition.name(),
        definition.root_entity(),
        definition.scope_entity(),
    ])?;
    for parameter in definition.parameters() {
        validate_simple_identifier(parameter.name())?;
    }
    validate_root_selection(definition)?;
    validate_result_shape(definition.result_shape())?;
    validate_predicates_and_ordering(definition)?;
    validate_lifecycle(definition)?;
    validate_authorization_and_disclosure(definition)
}

fn validate_root_selection(
    definition: &ErasedApplicationQueryDefinition,
) -> Result<(), ApplicationSchemaDeclarationDenial> {
    if let Some(orbit) = definition.orbit() {
        validate_identifiers([
            orbit.relation(),
            orbit.from(),
            orbit.to(),
            orbit.output_name(),
        ])?;
    }
    for path in definition.root_paths() {
        validate_identifiers([path.start_entity(), path.terminal_entity()])?;
        for step in path.steps() {
            validate_identifiers([step.relation(), step.parent_entity(), step.child_entity()])?;
        }
        for guard in path.guards() {
            validate_identifiers([guard.entity(), guard.aspect(), guard.field()])?;
        }
    }
    Ok(())
}

fn validate_result_shape(
    shape: &ApplicationQueryResultShape,
) -> Result<(), ApplicationSchemaDeclarationDenial> {
    validate_simple_identifier(shape.root_entity())?;
    for field in shape.fields() {
        validate_identifiers([
            field.entity(),
            field.aspect(),
            field.field(),
            field.output_name(),
        ])?;
    }
    for relation in shape.relations() {
        validate_identifiers([
            relation.relation(),
            relation.from(),
            relation.to(),
            relation.output_name(),
        ])?;
        validate_result_shape(relation.nested_shape())?;
    }
    Ok(())
}

fn validate_predicates_and_ordering(
    definition: &ErasedApplicationQueryDefinition,
) -> Result<(), ApplicationSchemaDeclarationDenial> {
    for predicate in definition.predicates() {
        let (entity, aspect, field) = predicate.field();
        validate_identifiers([entity, aspect, field, predicate.parameter()])?;
    }
    for ordering in definition.ordering() {
        let (entity, aspect, field) = ordering.field();
        validate_identifiers([entity, aspect, field, ordering.output_name()])?;
    }
    Ok(())
}

fn validate_lifecycle(
    definition: &ErasedApplicationQueryDefinition,
) -> Result<(), ApplicationSchemaDeclarationDenial> {
    if let Some(continuation) = definition.continuation() {
        validate_identifiers([
            continuation.relation(),
            continuation.parent_entity(),
            continuation.child_entity(),
        ])?;
    }
    if let Some(live) = definition.live_cause() {
        validate_simple_identifier(live.effect())?;
        let (scope_entity, scope_aspect, scope_field) = live.scope_field();
        let (target_entity, target_aspect, target_field) = live.target_field();
        validate_identifiers([
            scope_entity,
            scope_aspect,
            scope_field,
            target_entity,
            target_aspect,
            target_field,
        ])?;
    }
    Ok(())
}

fn validate_authorization_and_disclosure(
    definition: &ErasedApplicationQueryDefinition,
) -> Result<(), ApplicationSchemaDeclarationDenial> {
    if let ApplicationQueryAuthorizationRequirement::Ability {
        ability,
        scope_entity,
    } = definition.authorization()
    {
        validate_identifiers([ability.as_str(), scope_entity.as_str()])?;
    }
    let disclosure = definition.disclosure();
    if let Some(capability_name) = disclosure.capability_name() {
        validate_simple_identifier(capability_name)?;
    }
    for rule in disclosure.rules() {
        let selector = rule.selector();
        if let Some((entity, aspect, field)) = selector.field_contract() {
            validate_identifiers([entity, aspect, field])?;
        }
        if let Some((relation, from, to, _, _)) = selector.relation_contract() {
            validate_identifiers([relation, from, to])?;
        }
    }
    Ok(())
}

fn validate_identifiers<'a>(
    identifiers: impl IntoIterator<Item = &'a str>,
) -> Result<(), ApplicationSchemaDeclarationDenial> {
    for identifier in identifiers {
        validate_simple_identifier(identifier)?;
    }
    Ok(())
}
