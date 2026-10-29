//! Governed registration and reconstruction use the ordinary capability owner.
use super::*;
use worth_query_declaration::facade::{
    application_capability::{ApplicationCapabilityRef, ApplicationCapabilityRequest},
    application_schema::{ApplicationOperationMarkerIdentity, ApplicationOperationRef},
};
use worth_query_execution::facade::primary_graph::WorthQueryApprovedElevation;
type Scope<S, I> = <<<I as ApplicationQueryIntent<S>>::Binding as ApplicationQueryBinding<S>>::ScopeBinding as worth_query_declaration::facade::application_query::ApplicationQueryScopeBinding<S>>::Scope;

impl<Schema, Intent> WorthQueryApplicationQueryRequest<'_, '_, '_, Schema, Intent>
where
    Schema: ApplicationSchema,
    Intent: ApplicationQueryIntent<Schema>,
    <Intent::Binding as ApplicationQueryBinding<Schema>>::ScopeBinding:
        ApplicationQueryScopeResolution<
            Schema,
            <Intent::Binding as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
        >,
{
    /// Opens the shared registration after fresh current capability admission.
    pub fn open_index_governed<I, Capability, Operation, Input>(
        self,
        capability: ApplicationCapabilityRef<Schema, Capability>,
        operation: ApplicationOperationRef<Schema, Operation, Input>,
        input: Input,
    ) -> Result<WorthQueryManagedIndex<Schema, I>, WorthQueryApplicationRequestQueryDenial>
    where
        I: ApplicationManagedIndexBinding<Schema>,
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Operation::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
        Input: ApplicationCapabilityRequest<Schema, Capability, Scope = Scope<Schema, Intent>>
            + 'static,
    {
        self.open_index_with_approval(None, capability, operation, input)
    }

    /// Opens the shared registration after a genuine approval and current capability admission.
    pub fn open_index_approved<I, Capability, Operation, Input>(
        self,
        approved: &WorthQueryApprovedElevation,
        capability: ApplicationCapabilityRef<Schema, Capability>,
        operation: ApplicationOperationRef<Schema, Operation, Input>,
        input: Input,
    ) -> Result<WorthQueryManagedIndex<Schema, I>, WorthQueryApplicationRequestQueryDenial>
    where
        I: ApplicationManagedIndexBinding<Schema>,
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Operation::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
        Input: ApplicationCapabilityRequest<Schema, Capability, Scope = Scope<Schema, Intent>>
            + 'static,
    {
        self.open_index_with_approval(Some(approved), capability, operation, input)
    }

    fn open_index_with_approval<I, Capability, Operation, Input>(
        self,
        approved: Option<&WorthQueryApprovedElevation>,
        capability: ApplicationCapabilityRef<Schema, Capability>,
        operation: ApplicationOperationRef<Schema, Operation, Input>,
        input: Input,
    ) -> Result<WorthQueryManagedIndex<Schema, I>, WorthQueryApplicationRequestQueryDenial>
    where
        I: ApplicationManagedIndexBinding<Schema>,
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Operation::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
        Input: ApplicationCapabilityRequest<Schema, Capability, Scope = Scope<Schema, Intent>>
            + 'static,
    {
        let application = self.application;
        let request = self.scope;
        application
            .with_application_advancement(request, |phase| {
                self.with_admitted_governed_query(
                    &phase,
                    approved,
                    capability,
                    operation,
                    input,
                    |application, binding, _, plan| {
                        application
                            .open_application_managed_index(&phase, binding, plan)
                            .map_err(WorthQueryApplicationRequestQueryDenial::ManagedIndex)
                    },
                )
            })
            .map_err(WorthQueryApplicationRequestQueryDenial::ExecutionRequest)?
    }

    /// Explicitly rebuilds the complete Source after fresh current capability admission.
    pub fn rebuild_index_governed<I, Capability, Operation, Input>(
        self,
        index: &WorthQueryManagedIndex<Schema, I>,
        capability: ApplicationCapabilityRef<Schema, Capability>,
        operation: ApplicationOperationRef<Schema, Operation, Input>,
        input: Input,
    ) -> Result<(), WorthQueryApplicationRequestQueryDenial>
    where
        I: ApplicationManagedIndexBinding<Schema>,
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Operation::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
        Input: ApplicationCapabilityRequest<Schema, Capability, Scope = Scope<Schema, Intent>>
            + 'static,
    {
        self.rebuild_index_with_approval(index, None, capability, operation, input)
    }

    /// Explicitly rebuilds the complete Source after a genuine approval and current capability admission.
    pub fn rebuild_index_approved<I, Capability, Operation, Input>(
        self,
        index: &WorthQueryManagedIndex<Schema, I>,
        approved: &WorthQueryApprovedElevation,
        capability: ApplicationCapabilityRef<Schema, Capability>,
        operation: ApplicationOperationRef<Schema, Operation, Input>,
        input: Input,
    ) -> Result<(), WorthQueryApplicationRequestQueryDenial>
    where
        I: ApplicationManagedIndexBinding<Schema>,
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Operation::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
        Input: ApplicationCapabilityRequest<Schema, Capability, Scope = Scope<Schema, Intent>>
            + 'static,
    {
        self.rebuild_index_with_approval(index, Some(approved), capability, operation, input)
    }

    fn rebuild_index_with_approval<I, Capability, Operation, Input>(
        self,
        index: &WorthQueryManagedIndex<Schema, I>,
        approved: Option<&WorthQueryApprovedElevation>,
        capability: ApplicationCapabilityRef<Schema, Capability>,
        operation: ApplicationOperationRef<Schema, Operation, Input>,
        input: Input,
    ) -> Result<(), WorthQueryApplicationRequestQueryDenial>
    where
        I: ApplicationManagedIndexBinding<Schema>,
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Operation::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
        Input: ApplicationCapabilityRequest<Schema, Capability, Scope = Scope<Schema, Intent>>
            + 'static,
    {
        let application = self.application;
        let request = self.scope;
        application
            .with_application_advancement(request, |phase| {
                self.with_admitted_governed_query(
                    &phase,
                    approved,
                    capability,
                    operation,
                    input,
                    |application, binding, external, plan| {
                        application
                            .rebuild_application_managed_index(
                                &phase, binding, external, plan, index,
                            )
                            .map_err(WorthQueryApplicationRequestQueryDenial::ManagedIndex)
                    },
                )
            })
            .map_err(WorthQueryApplicationRequestQueryDenial::ExecutionRequest)?
    }
}
