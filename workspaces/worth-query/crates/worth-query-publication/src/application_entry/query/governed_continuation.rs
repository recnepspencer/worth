//! Public protected pages delegate the canonical admission and cursor owners.
use std::num::NonZeroUsize;

use worth_query_declaration::facade::{
    application_capability::{ApplicationCapabilityRef, ApplicationCapabilityRequest},
    application_query::{
        ApplicationQueryBinding, ApplicationQueryIntent, ApplicationQueryScopeResolution,
    },
    application_schema::{
        ApplicationOperationMarkerIdentity, ApplicationOperationRef, ApplicationSchema,
        ApplicationStructuredValueBinding,
    },
};
use worth_query_execution::facade::primary_graph::{
    WorthQueryAdmittedApplicationQueryPlan, WorthQueryApplicationContinuationPageResult,
    WorthQueryApplicationProjection, WorthQueryApplicationQueryAccessContext,
    WorthQueryApplicationQueryContinuation, WorthQueryApplicationQueryResumeControls,
    WorthQueryApprovedElevation, WorthQueryPrimaryGraphApplicationRuntime,
    WorthQueryPrincipalResolutionMode, WorthQueryProductQueryControls,
};

use super::{WorthQueryApplicationQueryRequest, WorthQueryApplicationRequestQueryDenial};

type Binding<Schema, Intent> = <Intent as ApplicationQueryIntent<Schema>>::Binding;
type Query<Schema, Intent> = <Binding<Schema, Intent> as ApplicationQueryBinding<Schema>>::Query;
type Parameters<Schema, Intent> = <<Binding<Schema, Intent> as ApplicationQueryBinding<Schema>>::ParameterBinding as ApplicationStructuredValueBinding>::Value;
type QueryResult<Schema, Intent> = <<Binding<Schema, Intent> as ApplicationQueryBinding<Schema>>::ResultBinding as ApplicationStructuredValueBinding>::Value;
type Principal<Schema, Intent> =
    <Binding<Schema, Intent> as ApplicationQueryBinding<Schema>>::Principal;
type PrincipalIdentity<Schema, Intent> =
    <Binding<Schema, Intent> as ApplicationQueryBinding<Schema>>::PrincipalIdentity;
type Scope<Schema, Intent> = <<Binding<Schema, Intent> as ApplicationQueryBinding<Schema>>::ScopeBinding as worth_query_declaration::facade::application_query::ApplicationQueryScopeBinding<Schema>>::Scope;

type Page<Schema, Intent> = WorthQueryApplicationContinuationPageResult<
    Schema,
    Query<Schema, Intent>,
    Parameters<Schema, Intent>,
    QueryResult<Schema, Intent>,
    Scope<Schema, Intent>,
>;
type Continuation<Schema, Intent> = WorthQueryApplicationQueryContinuation<
    Schema,
    Query<Schema, Intent>,
    Parameters<Schema, Intent>,
    QueryResult<Schema, Intent>,
    Scope<Schema, Intent>,
>;
type Plan<'a, Schema, Intent> = WorthQueryAdmittedApplicationQueryPlan<
    'a,
    Schema,
    Query<Schema, Intent>,
    Parameters<Schema, Intent>,
    QueryResult<Schema, Intent>,
    Principal<Schema, Intent>,
    PrincipalIdentity<Schema, Intent>,
    Scope<Schema, Intent>,
>;

mod page;
mod readmission;

impl<Schema, Intent> WorthQueryApplicationQueryRequest<'_, '_, '_, Schema, Intent>
where
    Schema: ApplicationSchema,
    Intent: ApplicationQueryIntent<Schema>,
    <Intent::Binding as ApplicationQueryBinding<Schema>>::ScopeBinding:
        ApplicationQueryScopeResolution<
            Schema,
            <Intent::Binding as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
        >,
    QueryResult<Schema, Intent>: WorthQueryApplicationProjection<Schema, Query<Schema, Intent>>,
{
    /// Fresh capability and independent READ admission precede first-page metadata.
    pub fn page_governed<Capability, Operation, Input>(
        self,
        capability: ApplicationCapabilityRef<Schema, Capability>,
        operation: ApplicationOperationRef<Schema, Operation, Input>,
        input: Input,
        page_width: NonZeroUsize,
    ) -> Result<Page<Schema, Intent>, WorthQueryApplicationRequestQueryDenial>
    where
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Operation::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
        Input: ApplicationCapabilityRequest<Schema, Capability, Scope = Scope<Schema, Intent>>
            + 'static,
    {
        self.page_with_governance(None, capability, operation, input, page_width)
    }
    /// Consumes the actual cursor and freshly admits its next page before invoking the callback.
    #[allow(clippy::too_many_arguments)]
    pub fn readmit_resume_governed<Capability, Operation, Input, Output>(
        self,
        capability: ApplicationCapabilityRef<Schema, Capability>,
        operation: ApplicationOperationRef<Schema, Operation, Input>,
        input: Input,
        continuation: Continuation<Schema, Intent>,
        page_width: NonZeroUsize,
        after_readmission: impl for<'admitted> FnOnce(
            &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
            &'admitted WorthQueryPrimaryGraphApplicationRuntime<Schema>,
            Plan<'admitted, Schema, Intent>,
        ) -> Output,
    ) -> Result<Output, WorthQueryApplicationRequestQueryDenial>
    where
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Operation::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
        Input: ApplicationCapabilityRequest<Schema, Capability, Scope = Scope<Schema, Intent>>
            + 'static,
    {
        self.readmit_with_governance(
            None,
            capability,
            operation,
            input,
            continuation,
            page_width,
            after_readmission,
        )
    }
    /// Resumes the retained page data through current governance and ordinary finalization.
    #[allow(clippy::too_many_arguments)]
    pub fn resume_governed<Capability, Operation, Input>(
        self,
        capability: ApplicationCapabilityRef<Schema, Capability>,
        operation: ApplicationOperationRef<Schema, Operation, Input>,
        input: Input,
        continuation: Continuation<Schema, Intent>,
        page_width: NonZeroUsize,
    ) -> Result<Page<Schema, Intent>, WorthQueryApplicationRequestQueryDenial>
    where
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Operation::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
        Input: ApplicationCapabilityRequest<Schema, Capability, Scope = Scope<Schema, Intent>>
            + 'static,
    {
        self.readmit_with_governance(
            None,
            capability,
            operation,
            input,
            continuation,
            page_width,
            |phase, application, plan| {
                application.execute_application_query_continuation_page(phase, plan)
            },
        )?
        .map_err(WorthQueryApplicationRequestQueryDenial::ContinuationExecution)
    }
    /// Fresh capability and independent READ admission precede first-page metadata.
    pub fn page_approved<Capability, Operation, Input>(
        self,
        approved: &WorthQueryApprovedElevation,
        capability: ApplicationCapabilityRef<Schema, Capability>,
        operation: ApplicationOperationRef<Schema, Operation, Input>,
        input: Input,
        page_width: NonZeroUsize,
    ) -> Result<Page<Schema, Intent>, WorthQueryApplicationRequestQueryDenial>
    where
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Operation::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
        Input: ApplicationCapabilityRequest<Schema, Capability, Scope = Scope<Schema, Intent>>
            + 'static,
    {
        self.page_with_governance(Some(approved), capability, operation, input, page_width)
    }
    /// Consumes the actual cursor and freshly admits its next page before invoking the callback.
    #[allow(clippy::too_many_arguments)]
    pub fn readmit_resume_approved<Capability, Operation, Input, Output>(
        self,
        approved: &WorthQueryApprovedElevation,
        capability: ApplicationCapabilityRef<Schema, Capability>,
        operation: ApplicationOperationRef<Schema, Operation, Input>,
        input: Input,
        continuation: Continuation<Schema, Intent>,
        page_width: NonZeroUsize,
        after_readmission: impl for<'admitted> FnOnce(
            &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
            &'admitted WorthQueryPrimaryGraphApplicationRuntime<Schema>,
            Plan<'admitted, Schema, Intent>,
        ) -> Output,
    ) -> Result<Output, WorthQueryApplicationRequestQueryDenial>
    where
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Operation::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
        Input: ApplicationCapabilityRequest<Schema, Capability, Scope = Scope<Schema, Intent>>
            + 'static,
    {
        self.readmit_with_governance(
            Some(approved),
            capability,
            operation,
            input,
            continuation,
            page_width,
            after_readmission,
        )
    }
    /// Resumes the retained page data through current governance and ordinary finalization.
    #[allow(clippy::too_many_arguments)]
    pub fn resume_approved<Capability, Operation, Input>(
        self,
        approved: &WorthQueryApprovedElevation,
        capability: ApplicationCapabilityRef<Schema, Capability>,
        operation: ApplicationOperationRef<Schema, Operation, Input>,
        input: Input,
        continuation: Continuation<Schema, Intent>,
        page_width: NonZeroUsize,
    ) -> Result<Page<Schema, Intent>, WorthQueryApplicationRequestQueryDenial>
    where
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Operation::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
        Input: ApplicationCapabilityRequest<Schema, Capability, Scope = Scope<Schema, Intent>>
            + 'static,
    {
        self.readmit_with_governance(
            Some(approved),
            capability,
            operation,
            input,
            continuation,
            page_width,
            |phase, application, plan| {
                application.execute_application_query_continuation_page(phase, plan)
            },
        )?
        .map_err(WorthQueryApplicationRequestQueryDenial::ContinuationExecution)
    }
}
