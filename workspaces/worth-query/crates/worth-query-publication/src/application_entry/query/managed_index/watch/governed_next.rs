//! Fresh capability and optional approval admission share the ordinary watch receiver.
use super::super::governed::Scope;
use super::*;
use worth_query_declaration::facade::{
    application_capability::{ApplicationCapabilityRef, ApplicationCapabilityRequest},
    application_schema::{ApplicationOperationMarkerIdentity, ApplicationOperationRef},
};
use worth_query_execution::facade::primary_graph::WorthQueryApprovedElevation;

impl<Schema, Intent> WorthQueryApplicationQueryRequest<'_, '_, '_, Schema, Intent>
where
    Schema: ApplicationSchema,
    Intent: ApplicationQueryIntent<Schema>,
    <Intent::Binding as ApplicationQueryBinding<Schema>>::ScopeBinding:
        ApplicationQueryScopeResolution<
            Schema,
            <Intent::Binding as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
        >,
    Row<Schema, Intent::Binding>:
        WorthQueryApplicationProjection<Schema, Query<Schema, Intent::Binding>>,
{
    /// Freshly admits capability and Parent READ before inspecting or acknowledging notifications.
    pub fn next_watch_governed<I, Capability, Operation, Input>(
        self,
        watch: &mut WorthQueryManagedIndexWatch<Schema, I>,
        capability: ApplicationCapabilityRef<Schema, Capability>,
        operation: ApplicationOperationRef<Schema, Operation, Input>,
        input: Input,
    ) -> Result<
        WorthQueryManagedIndexWatchOutcome<
            Query<Schema, Intent::Binding>,
            Row<Schema, Intent::Binding>,
        >,
        WorthQueryApplicationRequestQueryDenial,
    >
    where
        I: ApplicationManagedIndexBinding<Schema>,
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Operation::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
        Input: ApplicationCapabilityRequest<Schema, Capability, Scope = Scope<Schema, Intent>>
            + 'static,
    {
        self.next_with_approval(watch, None, capability, operation, input)
    }
    /// Freshly admits capability and Parent READ before inspecting or acknowledging notifications.
    pub fn next_watch_approved<I, Capability, Operation, Input>(
        self,
        watch: &mut WorthQueryManagedIndexWatch<Schema, I>,
        approved: &WorthQueryApprovedElevation,
        capability: ApplicationCapabilityRef<Schema, Capability>,
        operation: ApplicationOperationRef<Schema, Operation, Input>,
        input: Input,
    ) -> Result<
        WorthQueryManagedIndexWatchOutcome<
            Query<Schema, Intent::Binding>,
            Row<Schema, Intent::Binding>,
        >,
        WorthQueryApplicationRequestQueryDenial,
    >
    where
        I: ApplicationManagedIndexBinding<Schema>,
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Operation::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
        Input: ApplicationCapabilityRequest<Schema, Capability, Scope = Scope<Schema, Intent>>
            + 'static,
    {
        self.next_with_approval(watch, Some(approved), capability, operation, input)
    }
    fn next_with_approval<I, Capability, Operation, Input>(
        self,
        watch: &mut WorthQueryManagedIndexWatch<Schema, I>,
        approved: Option<&WorthQueryApprovedElevation>,
        capability: ApplicationCapabilityRef<Schema, Capability>,
        operation: ApplicationOperationRef<Schema, Operation, Input>,
        input: Input,
    ) -> Result<
        WorthQueryManagedIndexWatchOutcome<
            Query<Schema, Intent::Binding>,
            Row<Schema, Intent::Binding>,
        >,
        WorthQueryApplicationRequestQueryDenial,
    >
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
                if self.retained.is_some() {
                    completed = Some(Err(WorthQueryApplicationRequestQueryDenial::RequestMode));
                    return;
                }
                completed = Some(self.with_admitted_governed_query(
                    &phase,
                    approved,
                    capability,
                    operation,
                    input,
                    |application, binding, external, plan| {
                        application
                            .next_application_managed_index_watch::<Intent::Binding, I>(
                                &phase, binding, external, plan, watch,
                            )
                            .map_err(WorthQueryApplicationRequestQueryDenial::ManagedIndex)
                    },
                ));
            })
            .map_err(WorthQueryApplicationRequestQueryDenial::ExecutionRequest);
        finish_next(advancement, completed)
    }
}
