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
    /// Opens a complete maintained Parent watch after fresh current capability admission.
    pub fn watch_index_governed<I, Capability, Operation, Input>(
        self,
        capability: ApplicationCapabilityRef<Schema, Capability>,
        operation: ApplicationOperationRef<Schema, Operation, Input>,
        input: Input,
    ) -> Result<
        (
            WorthQueryPublishedApplicationResult<
                Query<Schema, Intent::Binding>,
                Row<Schema, Intent::Binding>,
            >,
            WorthQueryManagedIndexWatch<Schema, I>,
        ),
        WorthQueryApplicationRequestQueryDenial,
    >
    where
        I: ApplicationManagedIndexBinding<Schema>,
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Operation::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
        Input: ApplicationCapabilityRequest<Schema, Capability, Scope = Scope<Schema, Intent>>
            + 'static,
    {
        self.watch_with_approval(None, capability, operation, input)
    }
    /// Opens a complete maintained Parent watch after fresh current capability admission.
    pub fn watch_index_approved<I, Capability, Operation, Input>(
        self,
        approved: &WorthQueryApprovedElevation,
        capability: ApplicationCapabilityRef<Schema, Capability>,
        operation: ApplicationOperationRef<Schema, Operation, Input>,
        input: Input,
    ) -> Result<
        (
            WorthQueryPublishedApplicationResult<
                Query<Schema, Intent::Binding>,
                Row<Schema, Intent::Binding>,
            >,
            WorthQueryManagedIndexWatch<Schema, I>,
        ),
        WorthQueryApplicationRequestQueryDenial,
    >
    where
        I: ApplicationManagedIndexBinding<Schema>,
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Operation::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
        Input: ApplicationCapabilityRequest<Schema, Capability, Scope = Scope<Schema, Intent>>
            + 'static,
    {
        self.watch_with_approval(Some(approved), capability, operation, input)
    }
    fn watch_with_approval<I, Capability, Operation, Input>(
        self,
        approved: Option<&WorthQueryApprovedElevation>,
        capability: ApplicationCapabilityRef<Schema, Capability>,
        operation: ApplicationOperationRef<Schema, Operation, Input>,
        input: Input,
    ) -> Result<
        (
            WorthQueryPublishedApplicationResult<
                Query<Schema, Intent::Binding>,
                Row<Schema, Intent::Binding>,
            >,
            WorthQueryManagedIndexWatch<Schema, I>,
        ),
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
                            .open_application_managed_index_watch::<Intent::Binding, I>(
                                &phase, binding, external, plan,
                            )
                            .map(|(read, watch)| {
                                (
                                    publish_application_result(read.into_admitted_disclosed()),
                                    watch,
                                )
                            })
                            .map_err(WorthQueryApplicationRequestQueryDenial::ManagedIndex)
                    },
                ));
            })
            .map_err(WorthQueryApplicationRequestQueryDenial::ExecutionRequest);
        completion::completed_effect(advancement, completed)
    }
}
