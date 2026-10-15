//! Bind complete body meaning and codecs once at native installation.
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
        if body.scope_entity() != Some(Binding::scope_field().entity()) {
            return Err(denied());
        }
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
        let cardinality = Binding::CANDIDATES.cardinality();
        if cardinality.maximum_writes() < required_writes
            || cardinality.maximum_creates() < usize::from(body.creation().is_some())
            || cardinality.maximum_links()
                < body.creation().map_or(0, |creation| creation.links().len())
        {
            return Err(denied());
        }
        let mut input_bindings = Vec::new();
        for source in body
            .requirements()
            .iter()
            .flat_map(|requirement| requirement.operands())
            .map(|operand| operand.source())
        {
            if let worth_query_declaration::facade::application_schema::ApplicationOperationBodyRequirementSource::Field { field, binding, scalar_family, .. } = source {
                let installed_binding = schema.value_bindings().field(field.entity(), field.aspect(), field.field()).ok_or_else(denied)?;
                if installed_binding.identity() != binding || installed_binding.scalar_family() != *scalar_family { return Err(denied()); }
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
                    if source.entity() != Binding::scope_field().entity() {
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
            outputs: super::output::install_outputs::<Schema, Binding>(schema, body)?,
            body: body.clone(),
            marker: PhantomData,
            input_bindings,
        })
    }
}
