//! Original-request current-cache eviction uses ordinary Parent admission.
use super::completion::completed_effect;
use super::governed::Scope;
use super::*;
use worth_query_declaration::facade::{
    application_capability::{ApplicationCapabilityRef, ApplicationCapabilityRequest},
    application_schema::{ApplicationOperationMarkerIdentity, ApplicationOperationRef},
};
use worth_query_execution::facade::primary_graph::WorthQueryApprovedElevation;
impl<'application, 'principal, 'scope, Schema, Intent>
    WorthQueryApplicationQueryRequest<'application, 'principal, 'scope, Schema, Intent>
where
    Schema: ApplicationSchema,
    Intent: ApplicationQueryIntent<Schema>,
{
    /// Evicts the selected current image without reading Source content.
    /// Historical images may retain immutable material. Maintained candidate
    /// reads reconstruct only under their installed cold policy.
    pub fn evict_index<I: ApplicationManagedIndexBinding<Schema>>(
        self,
        index: &WorthQueryManagedIndex<Schema, I>,
    ) -> Result<(), WorthQueryApplicationRequestQueryDenial>
    where
        <Intent::Binding as ApplicationQueryBinding<Schema>>::ScopeBinding:
            ApplicationQueryScopeResolution<
                Schema,
                <Intent::Binding as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
            >,
    {
        let application = self.application;
        let request = self.scope;
        let mut completed = None;
        let advancement = application
            .with_application_advancement(request, |phase| {
                completed = Some(self.with_admitted_query(
                    &phase,
                    |application, binding, _, plan| {
                        application
                            .evict_application_managed_index(&phase, binding, plan, index)
                            .map_err(WorthQueryApplicationRequestQueryDenial::ManagedIndex)
                    },
                ));
            })
            .map_err(WorthQueryApplicationRequestQueryDenial::ExecutionRequest);
        completed_effect(advancement, completed)
    }
}

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
    /// Evicts current managed material after fresh current capability admission.
    pub fn evict_index_governed<I, Capability, Operation, Input>(
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
        self.evict_index_with_approval(index, None, capability, operation, input)
    }

    /// Evicts current managed material after a genuine approval and current capability admission.
    pub fn evict_index_approved<I, Capability, Operation, Input>(
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
        self.evict_index_with_approval(index, Some(approved), capability, operation, input)
    }

    fn evict_index_with_approval<I, Capability, Operation, Input>(
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
        let mut completed = None;
        let advancement = application
            .with_application_advancement(request, |phase| {
                completed = Some(self.with_admitted_governed_query(
                    &phase,
                    approved,
                    capability,
                    operation,
                    input,
                    |application, binding, _, plan| {
                        application
                            .evict_application_managed_index(&phase, binding, plan, index)
                            .map_err(WorthQueryApplicationRequestQueryDenial::ManagedIndex)
                    },
                ));
            })
            .map_err(WorthQueryApplicationRequestQueryDenial::ExecutionRequest);
        completed_effect(advancement, completed)
    }
}
