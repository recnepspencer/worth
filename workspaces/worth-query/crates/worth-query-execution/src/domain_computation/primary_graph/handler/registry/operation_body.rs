//! Central implementation selection: authored bodies cannot be replaced.
use {super::denial, super::ApplicationMutationBinding, super::ApplicationSchema, super::ApplicationSchemaBindingIdentity, crate::domain_computation::graph_layout::denial::WorthQueryPrimaryGraphInstallationDenialKind as DenialKind, super::OperationHandler, super::PendingMutationHandlerRegistry, super::WorthQueryInstalledApplicationMutationBinding, crate::domain_computation::graph_layout::denial::WorthQueryPrimaryGraphInstallationDenial};
use worth_query_declaration::facade::application_operation::ApplicationMutationScopeBinding;
use {
    super::super::super::operation_body::OperationBodyHandler,
    super::super::super::operation_body::WorthQueryOperationBodyDecision,
    crate::domain_computation::primary_graph::vocabulary::operation_body::WorthQueryOperationBodyDenial,
};

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
                self.entries
                    .get(binding.identity())
                    .is_some_and(|entry| entry.origin == super::HandlerOrigin::CompiledBody)
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
        self.register_resolved(binding, installed, handler, super::HandlerOrigin::Supplier)
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
        if Binding::WORKFLOW_CONTROL
            && handler
                .body()
                .creation()
                .and_then(|creation| creation.signing())
                .is_none()
        {
            return Err(denial(
                DenialKind::OperationBodyUnavailable,
                Binding::IDENTITY,
            ));
        }
        self.register_resolved(
            binding,
            installed,
            handler,
            super::HandlerOrigin::CompiledBody,
        )
    }
}

pub(in crate::domain_computation::primary_graph) trait OperationBodyScopedOps<
    Schema: ApplicationSchema,
>:
    crate::domain_computation::primary_graph::handler::host::HandlerHost<Schema>
{
    fn mutation_body_for_approval<Binding>(
        &self,
    ) -> Result<
        std::sync::Arc<dyn OperationHandler<Schema, Binding>>,
        super::MutationHandlerExecutionDenial,
    >
    where
        Binding: ApplicationMutationBinding<Schema>,
    {
        if !Binding::WORKFLOW_CONTROL
            || !self
                .mutation_handlers()
                .entries
                .get(Binding::IDENTITY)
                .is_some_and(|entry| entry.origin == super::HandlerOrigin::CompiledBody)
        {
            return Err(super::MutationHandlerExecutionDenial::WorkflowControl);
        }
        self.mutation_handlers()
            .get::<Binding>()
            .ok_or(super::MutationHandlerExecutionDenial::HandlerNotInstalled)
    }
}

impl<
        Schema: ApplicationSchema,
        HostRuntime: crate::domain_computation::primary_graph::handler::host::HandlerHost<Schema> + ?Sized,
    > OperationBodyScopedOps<Schema> for HostRuntime
{
}
