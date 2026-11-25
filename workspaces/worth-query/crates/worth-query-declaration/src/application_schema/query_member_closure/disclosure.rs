use crate::application_query::ErasedApplicationQueryDefinition;

use super::{ApplicationSchemaDeclarationDenial, ApplicationSchemaMember};

pub(super) fn validate_dependencies(
    definition: &ErasedApplicationQueryDefinition,
    members: &[ApplicationSchemaMember],
) -> Result<(), ApplicationSchemaDeclarationDenial> {
    validate_governing_capability(definition, members)?;
    if definition.disclosure().rules().iter().all(|rule| {
        let selector = rule.selector();
        !selector.is_internal_computation()
            || selector
                .field_contract()
                .is_some_and(|field| internal_field_is_installed(field, members))
            || selector
                .internal_relation_contract()
                .is_some_and(|(relation, from, to, _)| {
                    super::super::schema_read_closure::relation_exists(members, relation, from, to)
                })
    }) {
        Ok(())
    } else {
        Err(ApplicationSchemaDeclarationDenial::InvalidApplicationQuery)
    }
}

fn validate_governing_capability(
    definition: &ErasedApplicationQueryDefinition,
    members: &[ApplicationSchemaMember],
) -> Result<(), ApplicationSchemaDeclarationDenial> {
    let disclosure = definition.disclosure();
    let (Some(name), Some(capability_type)) =
        (disclosure.capability_name(), disclosure.capability_type())
    else {
        return Ok(());
    };
    if members.iter().any(|member| {
        matches!(
            member,
            ApplicationSchemaMember::ApplicationCapability { contract }
                if contract.name() == name && contract.capability_type() == capability_type
        )
    }) {
        Ok(())
    } else {
        Err(ApplicationSchemaDeclarationDenial::MissingApplicationQueryDependency)
    }
}

fn internal_field_is_installed(
    (entity, aspect, field): (&str, &str, &str),
    members: &[ApplicationSchemaMember],
) -> bool {
    members.iter().any(|member| {
        matches!(
            member,
            ApplicationSchemaMember::Field {
                entity: installed_entity,
                aspect: installed_aspect,
                field: installed_field,
                ..
            } if installed_entity == entity
                && installed_aspect == aspect
                && installed_field == field
        )
    })
}
