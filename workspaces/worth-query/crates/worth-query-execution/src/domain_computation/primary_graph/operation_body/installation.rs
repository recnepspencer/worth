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
        let mut input_bindings = Vec::new();
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
        Ok(Self {
            body: body.clone(),
            marker: PhantomData,
            input_bindings,
        })
    }
}
