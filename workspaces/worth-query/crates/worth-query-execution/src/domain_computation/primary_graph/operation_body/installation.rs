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
        .checked_add(
            body.creation()
                .map_or(0, |creation| creation.fields().len()),
        )
        .ok_or_else(denied)?;
    let cardinality = requirements.cardinality();
    let required_unlinks = body.creation().map_or(0, |creation| creation.links().iter().filter(|link|
        link.posture == worth_query_declaration::facade::application_schema::ApplicationOperationBodyCreationLinkPosture::ReplaceSingle).count());
    if cardinality.maximum_emits() < usize::from(body.external_emission().is_some())
        || cardinality.maximum_unlinks() < required_unlinks
        || cardinality.maximum_writes() < required_writes
        || cardinality.maximum_creates() < usize::from(body.creation().is_some())
        || cardinality.maximum_links()
            < body.creation().map_or(0, |creation| creation.links().len())
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
        let external_emission = super::emission::install::<Schema, Binding>(schema, body)?;
        if body.scope_entity() != Some(Binding::scope_field().entity()) {
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
        if let Some(creation) = body.creation() {
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
                        && source.entity() != Binding::scope_field().entity()
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
        Ok(Self {
            values: super::value::install(schema, body, Binding::IDENTITY)?,
            selection_reads: super::selection::installed_reads(body),
            outputs: super::output::install_outputs::<Schema, Binding>(schema, body)?,
            body: body.clone(),
            marker: PhantomData,
            input_bindings,
            external_emission,
        })
    }
}
