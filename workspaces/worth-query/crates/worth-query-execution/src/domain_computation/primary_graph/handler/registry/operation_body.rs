//! Central implementation selection: authored bodies cannot be replaced.
use super::super::super::operation_body::{
    OperationBodyHandler, WorthQueryOperationBodyDecision, WorthQueryOperationBodyDenial,
};
use super::{
    denial, ApplicationMutationBinding, ApplicationSchema, ApplicationSchemaBindingIdentity,
    DenialKind, OperationHandler, PendingMutationHandlerRegistry,
    WorthQueryInstalledApplicationMutationBinding, WorthQueryPrimaryGraphInstallationDenial,
};
use worth_query_declaration::facade::application_operation::ApplicationMutationScopeBinding;

type Scope<Schema, Binding> = <<Binding as ApplicationMutationBinding<Schema>>::ScopeBinding
    as ApplicationMutationScopeBinding<Schema>>::Scope;

impl<Schema: ApplicationSchema> PendingMutationHandlerRegistry<Schema> {
    pub(super) fn require_operation_body_coverage(
        &self,
        schema: &worth_query_installation::facade::WorthQueryInstalledApplicationSchema<Schema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        use std::collections::BTreeSet;
        use worth_query_declaration::facade::application_schema::ApplicationSchemaMember;

        let covered: BTreeSet<_> = schema
            .installed_mutation_binding_inventory()
            .filter(|binding| {
                !binding.workflow_control() && self.entries.contains_key(binding.identity())
            })
            .map(|binding| binding.operation_name())
            .collect();
        for member in schema.installed_declaration().members() {
            if let ApplicationSchemaMember::OperationBody { operation, .. } = member
                && !covered.contains(operation.as_str())
            {
                return Err(denial(DenialKind::OperationBodyUnavailable, operation));
            }
        }
        Ok(())
    }

    pub(in crate::domain_computation::primary_graph) fn register<Binding, Handler>(
        &mut self,
        binding: &ApplicationSchemaBindingIdentity,
        installed: &WorthQueryInstalledApplicationMutationBinding<Schema, Binding>,
        handler: Handler,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial>
    where
        Binding: ApplicationMutationBinding<Schema>,
        Handler: OperationHandler<Schema, Binding>,
    {
        if installed.operation().body().is_some() {
            return Err(denial(
                DenialKind::OperationBodyReplacement,
                Binding::IDENTITY,
            ));
        }
        self.register_resolved(binding, installed, handler)
    }

    pub(in crate::domain_computation::primary_graph) fn register_body<Binding>(
        &mut self,
        binding: &ApplicationSchemaBindingIdentity,
        installed: &WorthQueryInstalledApplicationMutationBinding<Schema, Binding>,
        handler: OperationBodyHandler<Schema, Binding>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial>
    where
        Binding: ApplicationMutationBinding<
            Schema,
            Result = (),
            Denial = WorthQueryOperationBodyDenial,
            Decision = WorthQueryOperationBodyDecision<Schema, Scope<Schema, Binding>>,
        >,
    {
        if installed.operation().body() != Some(handler.body()) {
            return Err(denial(
                DenialKind::OperationBodyUnavailable,
                Binding::IDENTITY,
            ));
        }
        self.register_resolved(binding, installed, handler)
    }
}
