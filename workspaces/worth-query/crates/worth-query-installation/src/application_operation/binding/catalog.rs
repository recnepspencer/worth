use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

use worth_query_declaration::facade::{
    application_operation::ApplicationMutationBindingDescriptor,
    application_schema::{ApplicationSchema, ApplicationSchemaMember},
};

use crate::application_schema::WorthQueryInstalledApplicationSchema;

use super::WorthQueryCompiledApplicationMutationBinding;
use crate::application_operation::{
    WorthQueryApplicationOperationInstallationDenial,
    WorthQueryApplicationOperationInstallationDenialKind as DenialKind,
};

#[derive(Default)]
pub(crate) struct WorthQueryInstalledApplicationMutationCatalog {
    bindings: BTreeMap<String, Arc<WorthQueryCompiledApplicationMutationBinding>>,
}

impl WorthQueryInstalledApplicationMutationCatalog {
    pub(crate) fn get_binding(
        &self,
        identity: &str,
    ) -> Option<Arc<WorthQueryCompiledApplicationMutationBinding>> {
        self.bindings.get(identity).map(Arc::clone)
    }

    pub(crate) fn descriptors(
        &self,
    ) -> impl ExactSizeIterator<Item = &ApplicationMutationBindingDescriptor> {
        self.bindings.values().map(|binding| binding.descriptor())
    }
}

pub(crate) fn compile_application_mutation_catalog<Schema>(
    schema: &WorthQueryInstalledApplicationSchema<Schema>,
) -> Result<
    WorthQueryInstalledApplicationMutationCatalog,
    WorthQueryApplicationOperationInstallationDenial,
>
where
    Schema: ApplicationSchema,
{
    let mut bindings = BTreeMap::new();
    for descriptor in schema.member_provenance.mutation_bindings() {
        validate_identities(descriptor)?;
        validate_operation(schema, descriptor)?;
        if !schema
            .installed_declaration()
            .members()
            .iter()
            .any(|member| {
                matches!(
                    member, ApplicationSchemaMember::ApplicationMutation { description }
                        if description == descriptor.description()
                )
            })
        {
            return Err(denial(
                DenialKind::MutationBindingMeaningChanged,
                descriptor.identity(),
            ));
        }
        validate_scope(schema, descriptor)?;
        validate_principal(schema, descriptor)?;
        validate_outputs(schema, descriptor)?;
        let compiled = Arc::new(WorthQueryCompiledApplicationMutationBinding::new(
            descriptor.clone(),
        ));
        if let Some(existing) = bindings.insert(descriptor.identity().to_owned(), compiled) {
            let kind = if existing.descriptor() == descriptor {
                DenialKind::MutationBindingIdentityCollision
            } else {
                DenialKind::MutationBindingMeaningChanged
            };
            return Err(denial(kind, descriptor.identity()));
        }
    }
    Ok(WorthQueryInstalledApplicationMutationCatalog { bindings })
}

fn validate_identities(
    descriptor: &ApplicationMutationBindingDescriptor,
) -> Result<(), WorthQueryApplicationOperationInstallationDenial> {
    if !valid_identity(descriptor.identity()) {
        return Err(denial(
            DenialKind::InvalidMutationBindingIdentity,
            descriptor.identity(),
        ));
    }
    if !valid_identity(descriptor.handler().identity()) {
        return Err(denial(
            DenialKind::InvalidMutationHandlerIdentity,
            descriptor.identity(),
        ));
    }
    if !valid_identity(descriptor.idempotency().identity()) {
        return Err(denial(
            DenialKind::InvalidMutationIdempotencyIdentity,
            descriptor.identity(),
        ));
    }
    Ok(())
}

fn valid_identity(identity: &str) -> bool {
    !identity.is_empty()
        && identity.trim() == identity
        && !identity
            .chars()
            .any(|character| character.is_control() || character.is_whitespace())
}

fn validate_operation<Schema>(
    schema: &WorthQueryInstalledApplicationSchema<Schema>,
    descriptor: &ApplicationMutationBindingDescriptor,
) -> Result<(), WorthQueryApplicationOperationInstallationDenial>
where
    Schema: ApplicationSchema,
{
    let matching_name = schema
        .installed_declaration()
        .members()
        .iter()
        .find_map(|member| match member {
            ApplicationSchemaMember::Operation {
                operation,
                input_type,
            } if operation == descriptor.operation_name() => Some(input_type),
            _ => None,
        });
    let Some(input_type) = matching_name else {
        return Err(denial(
            DenialKind::MutationBindingOperationNotInstalled,
            descriptor.identity(),
        ));
    };
    let portable_contract_exists = schema
        .portable_operation_contracts()
        .iter()
        .any(|contract| {
            contract.operation() == descriptor.operation_name()
                && contract.input_type() == descriptor.input_identity()
        });
    if input_type != descriptor.input_identity()
        || !schema
            .member_provenance
            .admits_mutation_binding_operation(descriptor)
        || !portable_contract_exists
    {
        return Err(denial(
            DenialKind::MutationBindingOperationMeaningChanged,
            descriptor.identity(),
        ));
    }
    Ok(())
}

fn validate_scope<Schema>(
    schema: &WorthQueryInstalledApplicationSchema<Schema>,
    descriptor: &ApplicationMutationBindingDescriptor,
) -> Result<(), WorthQueryApplicationOperationInstallationDenial>
where
    Schema: ApplicationSchema,
{
    let recipe = descriptor.scope().field();
    let exact_member = schema
        .installed_declaration()
        .members()
        .iter()
        .any(|member| match member {
            ApplicationSchemaMember::Field {
                entity,
                aspect,
                field,
                scalar_family,
                value_type,
                unit,
                frame,
                writable,
                equality_queryable,
                ..
            } => {
                recipe.locus().entity() == entity
                    && recipe.locus().aspect() == aspect
                    && recipe.locus().field() == field
                    && recipe.scalar_family() == *scalar_family
                    && recipe.binding_identity().as_str() == value_type
                    && recipe.unit().map(|value| value.as_str()) == unit.as_deref()
                    && recipe.frame().map(|value| value.as_str()) == frame.as_deref()
                    && descriptor.scope().writable() == *writable
                    && *equality_queryable
            }
            _ => false,
        });
    let exact_binding = schema
        .value_bindings()
        .field(
            recipe.locus().entity(),
            recipe.locus().aspect(),
            recipe.locus().field(),
        )
        .is_some_and(|binding| {
            binding.identity() == recipe.binding_identity()
                && binding.binding_type() == recipe.binding_type()
                && binding.value_type() == recipe.value_type()
        });
    if exact_member && exact_binding {
        Ok(())
    } else {
        Err(denial(
            DenialKind::MutationBindingScopeMeaningChanged,
            descriptor.identity(),
        ))
    }
}

fn validate_principal<Schema>(
    schema: &WorthQueryInstalledApplicationSchema<Schema>,
    descriptor: &ApplicationMutationBindingDescriptor,
) -> Result<(), WorthQueryApplicationOperationInstallationDenial>
where
    Schema: ApplicationSchema,
{
    let contract = descriptor.principal();
    let exact_member = schema.installed_declaration().members().iter().any(|member| matches!(
        member,
        ApplicationSchemaMember::PrincipalBinding {
            binding, mapping_entity, identity_aspect, identity_field, status_aspect,
            status_field, target_relation, principal_entity, principal_identity_aspect,
            principal_identity_field, principal_identity_scalar_family,
            principal_identity_value_type,
        } if binding == contract.name()
            && mapping_entity == contract.mapping_entity()
            && identity_aspect == contract.identity_aspect()
            && identity_field == contract.identity_field()
            && status_aspect == contract.status_aspect()
            && status_field == contract.status_field()
            && target_relation == contract.target_relation()
            && principal_entity == contract.principal_entity()
            && principal_identity_aspect == contract.principal_identity_binding().locus().aspect()
            && principal_identity_field == contract.principal_identity_field()
            && *principal_identity_scalar_family == contract.principal_identity_scalar_family()
            && principal_identity_value_type == contract.principal_identity_value_type()
    ));
    let recipe = contract.principal_identity_binding();
    let exact_binding = schema
        .value_bindings()
        .field(
            recipe.locus().entity(),
            recipe.locus().aspect(),
            recipe.locus().field(),
        )
        .is_some_and(|binding| {
            binding.identity() == recipe.binding_identity()
                && binding.binding_type() == recipe.binding_type()
                && binding.value_type() == recipe.value_type()
                && binding.is_identity()
        });
    if exact_member && exact_binding {
        Ok(())
    } else {
        Err(denial(
            DenialKind::MutationBindingPrincipalMeaningChanged,
            descriptor.identity(),
        ))
    }
}

fn validate_outputs<Schema>(
    schema: &WorthQueryInstalledApplicationSchema<Schema>,
    descriptor: &ApplicationMutationBindingDescriptor,
) -> Result<(), WorthQueryApplicationOperationInstallationDenial>
where
    Schema: ApplicationSchema,
{
    let mut names = BTreeSet::new();
    let mut creates = 0usize;
    let mut retires = 0usize;
    for role in descriptor.output_roles() {
        if !valid_identity(role.name()) || !names.insert(role.name()) {
            return Err(denial(
                DenialKind::InvalidMutationOutputRole,
                descriptor.identity(),
            ));
        }
        let entity_exists = schema
            .installed_declaration()
            .members()
            .iter()
            .any(|member| matches!(member, ApplicationSchemaMember::Entity { entity } if entity == role.entity()));
        if !entity_exists {
            return Err(denial(
                DenialKind::MutationOutputEntityNotInstalled,
                descriptor.identity(),
            ));
        }
        match role.posture() {
            worth_query_declaration::facade::application_operation::ApplicationMutationOutputPosture::Create => creates += 1,
            worth_query_declaration::facade::application_operation::ApplicationMutationOutputPosture::Retire => retires += 1,
            worth_query_declaration::facade::application_operation::ApplicationMutationOutputPosture::Preserve => {}
        }
    }
    let candidates = descriptor.candidates().cardinality();
    if creates > candidates.maximum_creates() || retires > candidates.maximum_deletes() {
        return Err(denial(
            DenialKind::MutationOutputCandidateMismatch,
            descriptor.identity(),
        ));
    }
    Ok(())
}

fn denial(kind: DenialKind, subject: &str) -> WorthQueryApplicationOperationInstallationDenial {
    WorthQueryApplicationOperationInstallationDenial::new(kind, subject)
}
