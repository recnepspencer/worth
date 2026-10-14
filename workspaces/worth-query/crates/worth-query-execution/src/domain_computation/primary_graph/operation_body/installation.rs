//! Bind complete body meaning and codecs once at native installation.
use super::super::{
    WorthQueryPrimaryGraphInstallationDenial, WorthQueryPrimaryGraphInstallationDenialKind,
};
use super::OperationBodyHandler;
use std::marker::PhantomData;
use worth_query_declaration::facade::application_operation::ApplicationMutationBinding;
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
        for step in body.scope_fields() {
            let locus = step.field();
            let binding = schema
                .value_bindings()
                .field(locus.entity(), locus.aspect(), locus.field())
                .ok_or_else(denied)?;
            if binding.identity() != step.binding()
                || binding.validate_encoded(step.value()).is_err()
            {
                return Err(denied());
            }
        }
        Ok(Self {
            body: body.clone(),
            marker: PhantomData,
        })
    }
}
