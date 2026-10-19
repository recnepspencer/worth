//! Bind complete body meaning and codecs once at native installation.
#[cfg(test)]
mod tests;
use super::super::{
    WorthQueryPrimaryGraphInstallationDenial, WorthQueryPrimaryGraphInstallationDenialKind,
};
use super::OperationBodyHandler;
use std::marker::PhantomData;
use worth_query_declaration::facade::application_operation::ApplicationMutationBinding;
use worth_query_declaration::facade::application_schema::ApplicationStructuredValueBinding;
use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryInstalledApplicationMutationBinding,
    WorthQueryInstalledApplicationSchema,
};

fn require_candidate_capacity(
    body: &worth_query_declaration::facade::application_schema::ApplicationOperationBodyProgram,
    requirements: &worth_query_declaration::facade::application_operation::ApplicationCandidateRequirements,
    binding_identity: &str,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    let denied = || {
        WorthQueryPrimaryGraphInstallationDenial::new(
            WorthQueryPrimaryGraphInstallationDenialKind::OperationBodyUnavailable,
            binding_identity,
        )
    };
    let required_writes = body
        .scope_fields()
        .iter()
        .filter(|step| step.write_target().is_some())
        .count()
        .checked_add(body.path_value_assignments().len())
        .ok_or_else(denied)?
        .checked_add(
            body.creation()
                .into_iter()
                .flat_map(|c| c.initialization_sites())
                .try_fold(0usize, |sum, site| sum.checked_add(site.fields().len()))
                .ok_or_else(denied)?,
        )
        .ok_or_else(denied)?;
    let cardinality = requirements.cardinality();
    let retirement_capacity = body
        .retirements()
        .iter()
        .try_fold(0usize, |sum, rule| {
            sum.checked_add(rule.maximum_owners.get())
        })
        .ok_or_else(denied)?;
    let required_unlinks = body.creation().map_or(0, |creation| {
        creation
            .links()
            .iter()
            .filter(|link| link.posture.replaces_selection())
            .count()
    });
    let required_deletes = retirement_capacity
        .checked_add(body.creation().map_or(0, |creation| {
            creation
                .links()
                .iter()
                .filter(|link| link.posture.retires_predecessor())
                .count()
        }))
        .ok_or_else(denied)?;
    let required_unlinks = required_unlinks
        .checked_add(retirement_capacity)
        .ok_or_else(denied)?;
    if cardinality.maximum_emits() < usize::from(body.external_emission().is_some())
        || cardinality.maximum_deletes() < required_deletes
        || cardinality.maximum_unlinks() < required_unlinks
        || cardinality.maximum_writes() < required_writes
        || cardinality.maximum_creates() < body.creation().map_or(0, |c| 1 + c.children().len())
        || cardinality.maximum_links()
            < body
                .creation()
                .into_iter()
                .flat_map(|c| c.initialization_sites())
                .try_fold(0usize, |sum, site| sum.checked_add(site.links().len()))
                .ok_or_else(denied)?
    {
        return Err(denied());
    }
    Ok(())
}

impl<Schema: ApplicationSchema, Binding: ApplicationMutationBinding<Schema>>
    OperationBodyHandler<Schema, Binding>
{
    pub(in crate::domain_computation::primary_graph) fn install(
        schema: &WorthQueryInstalledApplicationSchema<Schema>,
        installed: &WorthQueryInstalledApplicationMutationBinding<Schema, Binding>,
    ) -> Result<Self, WorthQueryPrimaryGraphInstallationDenial> {
        let denied = || {
            WorthQueryPrimaryGraphInstallationDenial::new(
                WorthQueryPrimaryGraphInstallationDenialKind::OperationBodyUnavailable,
                Binding::IDENTITY,
            )
        };
        if installed.operation().binding_identity() != &schema.binding_identity() {
            return Err(denied());
        }
        let body = installed.operation().body().ok_or_else(denied)?;
        if !body.has_supported_retiring_replacement()
            || !body.has_supported_creation_reuse()
            || !body.has_supported_child_creations()
        {
            return Err(denied());
        }
        let retirement_inputs = body
            .retirements()
            .iter()
            .map(|rule| {
                if !rule.fits_body(body) {
                    return Err(denied());
                }
                rule.source_input_index(body).ok_or_else(denied)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let external_emission = super::emission::install::<Schema, Binding>(schema, body)?;
        if body.scope_entity() != Some(Binding::scope_field().entity()) {
            return Err(denied());
        }
        if body.path_value_assignments().iter().any(|step| {
            !step.target.has_supported_shape()
                || Some(step.target.path.start_entity()) != body.scope_entity()
        }) {
            return Err(denied());
        }
        require_candidate_capacity(body, &Binding::CANDIDATES, Binding::IDENTITY)?;
        let mut input_bindings = Vec::new();
        for source in body.observation_operands().map(|operand| operand.source()) {
            use worth_query_declaration::facade::application_schema::ApplicationOperationBodyRequirementSource as Source;
            if let Source::PrincipalIdentity {
                scope,
                mutation_binding,
                principal_binding,
                field,
                binding,
                scalar_family,
            } = source
            {
                let principal = Binding::principal_binding();
                if scope != Binding::scope_field().entity()
                    || mutation_binding != Binding::IDENTITY
                    || principal_binding != principal.name()
                    || field.entity() != principal.principal_entity()
                    || field.aspect() != principal.principal_identity_aspect()
                    || field.field() != principal.principal_identity_field()
                    || binding.as_str() != principal.principal_identity_value_type()
                    || *scalar_family != principal.principal_identity_scalar_family()
                {
                    return Err(denied());
                }
                let installed_binding = schema
                    .value_bindings()
                    .field(field.entity(), field.aspect(), field.field())
                    .ok_or_else(denied)?;
                if installed_binding.unit().is_some() {
                    return Err(denied());
                }
            }
            if let Source::Field {
                field,
                binding,
                scalar_family,
                ..
            }
            | Source::PrincipalIdentity {
                field,
                binding,
                scalar_family,
                ..
            } = source
            {
                let installed_binding = schema
                    .value_bindings()
                    .field(field.entity(), field.aspect(), field.field())
                    .ok_or_else(denied)?;
                if installed_binding.identity() != binding
                    || installed_binding.scalar_family() != *scalar_family
                {
                    return Err(denied());
                }
            }
        }
        for step in body.scope_fields() {
            let locus = step.field();
            let binding = schema
                .value_bindings()
                .field(locus.entity(), locus.aspect(), locus.field())
                .ok_or_else(denied)?;
            if binding.identity() != step.binding()
                || binding.scalar_family() != step.scalar_family()
                || step
                    .literal_value()
                    .is_some_and(|value| binding.validate_encoded(value).is_err())
            {
                return Err(denied());
            }
            if step.input_member().is_some() {
                if !Binding::InputBinding::RETAIN_CANONICAL_INPUT
                    || !binding.is_readable()
                    || !binding.admits_native_input()
                {
                    return Err(denied());
                }
                input_bindings.push(binding.clone());
            }
        }
        for creation in body
            .creation()
            .into_iter()
            .flat_map(|root| root.initialization_sites())
        {
            for link in creation.links() {
                super::super::WorthQueryApplicationEntityKey::<Schema, ()>::new(&link.key)
                    .map_err(|_| denied())?;
            }
            super::super::WorthQueryApplicationEntityKey::<Schema, ()>::new(creation.key())
                .map_err(|_| denied())?;
            for step in creation.fields() {
                let locus = step.field();
                if locus.entity() != creation.entity() {
                    return Err(denied());
                }
                let binding = schema
                    .value_bindings()
                    .field(locus.entity(), locus.aspect(), locus.field())
                    .ok_or_else(denied)?;
                if binding.identity() != step.binding()
                    || binding.scalar_family() != step.scalar_family()
                    || step
                        .literal_value()
                        .is_some_and(|value| binding.validate_encoded(value).is_err())
                {
                    return Err(denied());
                }
                if let Some(source) = step.source_field() {
                    if step.related_source().is_none()
                        && step.path_source().is_none()
                        && source.entity() != Binding::scope_field().entity()
                    {
                        return Err(denied());
                    }
                    if let Some(path) = step.path_source()
                        && (!path.has_supported_shape()
                            || path.path.start_entity() != Binding::scope_field().entity())
                    {
                        return Err(denied());
                    }
                    let source_binding = schema
                        .value_bindings()
                        .field(source.entity(), source.aspect(), source.field())
                        .ok_or_else(denied)?;
                    if source_binding.identity() != step.binding()
                        || source_binding.scalar_family() != step.scalar_family()
                        || source_binding.unit() != binding.unit()
                    {
                        return Err(denied());
                    }
                }
                if let worth_query_declaration::facade::application_schema::ApplicationOperationBodyCreatedField::PrincipalIdentity {
                    mutation_binding, principal_binding, source, ..
                } = step {
                    let principal = Binding::principal_binding();
                    if mutation_binding != Binding::IDENTITY || principal_binding != principal.name()
                        || source.entity() != principal.principal_entity() || source.aspect() != principal.principal_identity_aspect()
                        || source.field() != principal.principal_identity_field() || step.binding().as_str() != principal.principal_identity_value_type()
                        || step.scalar_family() != principal.principal_identity_scalar_family() || binding.unit().is_some()
                    { return Err(denied()); }
                    let identity_binding = schema.value_bindings().field(source.entity(), source.aspect(), source.field()).ok_or_else(denied)?;
                    if identity_binding.identity() != step.binding() || identity_binding.scalar_family() != step.scalar_family() || identity_binding.unit() != binding.unit() { return Err(denied()); }
                }
                if step.input_member().is_some() {
                    if !Binding::InputBinding::RETAIN_CANONICAL_INPUT
                        || !binding.is_readable()
                        || !binding.admits_native_input()
                    {
                        return Err(denied());
                    }
                    input_bindings.push(binding.clone());
                }
            }
        }
        for guard in body.path_input_expectations() {
            let source = &guard.source;
            if !source.has_supported_shape()
                || source.path.start_entity() != Binding::scope_field().entity()
            {
                return Err(denied());
            }
            let binding = schema
                .value_bindings()
                .field(
                    source.field.entity(),
                    source.field.aspect(),
                    source.field.field(),
                )
                .ok_or_else(denied)?;
            if binding.identity() != &guard.binding
                || binding.scalar_family() != guard.scalar_family
                || !Binding::InputBinding::RETAIN_CANONICAL_INPUT
                || !binding.is_readable()
                || !binding.admits_native_input()
            {
                return Err(denied());
            }
            input_bindings.push(binding.clone());
        }
        Ok(Self {
            values: super::value::install(schema, body, Binding::IDENTITY)?,
            selection_reads: super::selection::installed_reads(body),
            outputs: super::output::install_outputs::<Schema, Binding>(schema, body)?,
            body: body.clone(),
            marker: PhantomData,
            input_bindings,
            external_emission,
            retirement_inputs,
            child_parents: super::children::parent_indices(body),
        })
    }
}
