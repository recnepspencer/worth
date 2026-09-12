use std::{collections::BTreeMap, num::NonZeroUsize, sync::Arc};

use worth_query_declaration::facade::{
    application_query::ApplicationQueryBindingDescriptor,
    application_schema::{ApplicationSchema, ApplicationSchemaMember},
};

use crate::{
    application_query::{
        WorthQueryApplicationQueryInstallationDenial,
        WorthQueryApplicationQueryInstallationDenialKind as DenialKind,
    },
    application_schema::WorthQueryInstalledApplicationSchema,
};

use super::{
    ApplicationQueryBindingKey, WorthQueryCompiledApplicationQuery,
    WorthQueryCompiledApplicationQueryBinding, WorthQueryInstalledApplicationQueryLimits,
};

#[derive(Default)]
pub(crate) struct WorthQueryInstalledApplicationQueryCatalog {
    #[cfg(any(test, feature = "certification-query-lookup"))]
    queries: BTreeMap<ApplicationQueryBindingKey, Arc<WorthQueryCompiledApplicationQuery>>,
    #[cfg(any(test, feature = "certification-query-lookup"))]
    queries_by_name: BTreeMap<String, Arc<WorthQueryCompiledApplicationQuery>>,
    bindings: BTreeMap<String, Arc<WorthQueryCompiledApplicationQueryBinding>>,
}

impl WorthQueryInstalledApplicationQueryCatalog {
    pub(crate) fn descriptors(
        &self,
    ) -> impl ExactSizeIterator<Item = &ApplicationQueryBindingDescriptor> {
        self.bindings.values().map(|binding| binding.descriptor())
    }

    #[cfg(any(test, feature = "certification-query-lookup"))]
    pub(crate) fn get_query(
        &self,
        key: &ApplicationQueryBindingKey,
    ) -> Option<Arc<WorthQueryCompiledApplicationQuery>> {
        self.queries.get(key).map(Arc::clone)
    }

    #[cfg(any(test, feature = "certification-query-lookup"))]
    pub(crate) fn get_query_by_name(
        &self,
        name: &str,
    ) -> Option<Arc<WorthQueryCompiledApplicationQuery>> {
        self.queries_by_name.get(name).map(Arc::clone)
    }

    pub(crate) fn get_binding(
        &self,
        identity: &str,
    ) -> Option<Arc<WorthQueryCompiledApplicationQueryBinding>> {
        self.bindings.get(identity).map(Arc::clone)
    }
}

pub(crate) fn compile_application_query_catalog<Schema>(
    schema: &WorthQueryInstalledApplicationSchema<Schema>,
) -> Result<WorthQueryInstalledApplicationQueryCatalog, WorthQueryApplicationQueryInstallationDenial>
where
    Schema: ApplicationSchema,
{
    let (queries, queries_by_name) = compile_queries(schema)?;
    let mut bindings = BTreeMap::new();
    for descriptor in schema.member_provenance.query_bindings() {
        validate_binding_identity(descriptor)?;
        let query = require_exact_query(descriptor, &queries, &queries_by_name)?;
        validate_scope_field(schema, descriptor)?;
        validate_principal_binding(schema, descriptor)?;
        let limits = install_limits(descriptor)?;
        let compiled = Arc::new(WorthQueryCompiledApplicationQueryBinding::new(
            descriptor.clone(),
            query,
            limits,
        ));
        if let Some(existing) = bindings.insert(descriptor.identity().to_owned(), compiled) {
            let kind = if existing.descriptor() == descriptor {
                DenialKind::BindingIdentityCollision
            } else {
                DenialKind::BindingMeaningChanged
            };
            return Err(denial(kind, descriptor.identity()));
        }
    }
    Ok(WorthQueryInstalledApplicationQueryCatalog {
        #[cfg(any(test, feature = "certification-query-lookup"))]
        queries,
        #[cfg(any(test, feature = "certification-query-lookup"))]
        queries_by_name,
        bindings,
    })
}

type QueryMaps = (
    BTreeMap<ApplicationQueryBindingKey, Arc<WorthQueryCompiledApplicationQuery>>,
    BTreeMap<String, Arc<WorthQueryCompiledApplicationQuery>>,
);

fn compile_queries<Schema>(
    schema: &WorthQueryInstalledApplicationSchema<Schema>,
) -> Result<QueryMaps, WorthQueryApplicationQueryInstallationDenial>
where
    Schema: ApplicationSchema,
{
    let mut queries = BTreeMap::new();
    let mut names = BTreeMap::new();
    for definition in schema
        .installed_declaration()
        .members()
        .iter()
        .filter_map(|member| match member {
            ApplicationSchemaMember::ApplicationQuery { definition } => Some(definition),
            _ => None,
        })
    {
        let key = ApplicationQueryBindingKey::from_definition(definition);
        let compiled = Arc::new(WorthQueryCompiledApplicationQuery::compile(
            schema, definition,
        )?);
        if queries.insert(key, Arc::clone(&compiled)).is_some()
            || names
                .insert(definition.name().to_owned(), compiled)
                .is_some()
        {
            return Err(denial(DenialKind::QueryMeaningChanged, definition.name()));
        }
    }
    Ok((queries, names))
}

fn require_exact_query(
    descriptor: &ApplicationQueryBindingDescriptor,
    queries: &BTreeMap<ApplicationQueryBindingKey, Arc<WorthQueryCompiledApplicationQuery>>,
    names: &BTreeMap<String, Arc<WorthQueryCompiledApplicationQuery>>,
) -> Result<Arc<WorthQueryCompiledApplicationQuery>, WorthQueryApplicationQueryInstallationDenial> {
    let key = ApplicationQueryBindingKey::from_binding(descriptor);
    if let Some(query) = queries.get(&key) {
        return Ok(Arc::clone(query));
    }
    let kind = if names.contains_key(descriptor.query_name()) {
        DenialKind::BindingQueryMeaningChanged
    } else {
        DenialKind::BindingQueryNotInstalled
    };
    Err(denial(kind, descriptor.identity()))
}

fn validate_binding_identity(
    descriptor: &ApplicationQueryBindingDescriptor,
) -> Result<(), WorthQueryApplicationQueryInstallationDenial> {
    let valid = |identity: &str| {
        !identity.is_empty()
            && identity.trim() == identity
            && !identity
                .chars()
                .any(|character| character.is_control() || character.is_whitespace())
    };
    if !valid(descriptor.identity()) || !valid(descriptor.input_identity().as_str()) {
        return Err(denial(
            DenialKind::InvalidBindingIdentity,
            descriptor.identity(),
        ));
    }
    Ok(())
}

fn validate_scope_field<Schema>(
    schema: &WorthQueryInstalledApplicationSchema<Schema>,
    descriptor: &ApplicationQueryBindingDescriptor,
) -> Result<(), WorthQueryApplicationQueryInstallationDenial>
where
    Schema: ApplicationSchema,
{
    let recipe = descriptor.scope().field();
    let exact_member = schema
        .installed_declaration()
        .members()
        .iter()
        .any(|member| {
            let ApplicationSchemaMember::Field {
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
            } = member
            else {
                return false;
            };
            recipe.locus().entity() == entity
                && recipe.locus().aspect() == aspect
                && recipe.locus().field() == field
                && recipe.scalar_family() == *scalar_family
                && recipe.binding_identity().as_str() == value_type
                && recipe.unit().map(|value| value.as_str()) == unit.as_deref()
                && recipe.frame().map(|value| value.as_str()) == frame.as_deref()
                && descriptor.scope().writable() == *writable
                && *equality_queryable
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
            DenialKind::BindingScopeMeaningChanged,
            descriptor.identity(),
        ))
    }
}

fn validate_principal_binding<Schema>(
    schema: &WorthQueryInstalledApplicationSchema<Schema>,
    descriptor: &ApplicationQueryBindingDescriptor,
) -> Result<(), WorthQueryApplicationQueryInstallationDenial>
where
    Schema: ApplicationSchema,
{
    let contract = descriptor.principal();
    let exact = schema.installed_declaration().members().iter().any(|member| matches!(
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
            && principal_identity_aspect == contract.principal_identity_aspect()
            && principal_identity_field == contract.principal_identity_field()
            && *principal_identity_scalar_family == contract.principal_identity_scalar_family()
            && principal_identity_value_type == contract.principal_identity_value_type()
    ));
    let recipe = contract.principal_identity_binding();
    let exact_value_binding = schema
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
    if exact && exact_value_binding {
        Ok(())
    } else {
        Err(denial(
            DenialKind::BindingPrincipalMeaningChanged,
            descriptor.identity(),
        ))
    }
}

fn install_limits(
    descriptor: &ApplicationQueryBindingDescriptor,
) -> Result<WorthQueryInstalledApplicationQueryLimits, WorthQueryApplicationQueryInstallationDenial>
{
    let declared = descriptor.limits();
    install_declared_limits(
        declared.maximum_results(),
        declared.maximum_work(),
        descriptor.identity(),
    )
}

fn install_declared_limits(
    declared_results: usize,
    declared_work: usize,
    subject: &str,
) -> Result<WorthQueryInstalledApplicationQueryLimits, WorthQueryApplicationQueryInstallationDenial>
{
    let maximum_results = NonZeroUsize::new(declared_results)
        .ok_or_else(|| denial(DenialKind::BindingResultLimitIsZero, subject))?;
    let maximum_work = NonZeroUsize::new(declared_work)
        .ok_or_else(|| denial(DenialKind::BindingWorkLimitIsZero, subject))?;
    Ok(WorthQueryInstalledApplicationQueryLimits::new(
        maximum_results,
        maximum_work,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_request_limits_are_denied_before_installation() {
        assert_eq!(
            install_declared_limits(0, 1, "binding").unwrap_err().kind(),
            DenialKind::BindingResultLimitIsZero
        );
        assert_eq!(
            install_declared_limits(1, 0, "binding").unwrap_err().kind(),
            DenialKind::BindingWorkLimitIsZero
        );
    }
}

fn denial(kind: DenialKind, subject: &str) -> WorthQueryApplicationQueryInstallationDenial {
    WorthQueryApplicationQueryInstallationDenial::new(kind, subject)
}
