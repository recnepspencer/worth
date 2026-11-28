//! Governed source read inside the original producer reader and native borrow.
use super::*;
use crate::domain_computation::primary_graph::{
    SharedSelectedProductOperation, WorthQueryPrimaryGraphApplicationRuntime,
    WorthQueryPrincipalResolutionMode,
};
use std::num::NonZeroUsize;
use std::sync::Arc;
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope,
};
use worth_query_declaration::facade::application_query::{
    ApplicationQueryBinding, ApplicationQueryIntent, ApplicationQueryScopeBinding,
    ApplicationQueryScopeResolution,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationSchema, ApplicationStructuredValueBinding,
};
use worth_relational::facade::mvcc::CompanionPreflightStop;

mod admitted;
mod applied;
mod candidate_names;
mod current_control;
mod permission;
mod predecessor;
mod read_closure;
mod same_scope;
mod single;
pub use read_closure::SourceQueryReadClosure;

type SourceValue<S, B> =
    <<B as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value;
pub type SourceReadPlan<'a, S, B> = WorthQueryAdmittedApplicationQueryPlan<
    'a, S, <B as ApplicationQueryBinding<S>>::Query,
    <<B as ApplicationQueryBinding<S>>::ParameterBinding as ApplicationStructuredValueBinding>::Value,
    SourceValue<S, B>, <B as ApplicationQueryBinding<S>>::Principal,
    <B as ApplicationQueryBinding<S>>::PrincipalIdentity,
    <<B as ApplicationQueryBinding<S>>::ScopeBinding as ApplicationQueryScopeBinding<S>>::Scope,
>;

/// Borrowed from the real producer execution entry. Each call freshly admits
/// its installed query; historical truth carries no current permission.
pub struct SourceQueryReaderAccess<'a, Schema> {
    pub application:
        &'a WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    pub shared:
        &'a SharedSelectedProductOperation<'a, Schema>,
    pub principal:
        &'a WorthQueryAuthenticatedExternalPrincipal<Schema>,
    pub request: &'a WorthQueryRequestScope,
    pub applied_participants: Option<&'a dyn crate::domain_computation::primary_graph::provider::AppliedOutputParticipants>,
    pub applied_source_publication: Option<applied::AppliedSourcePublicationLoan>,
    /// The live coupled assembly a producer decision's applied candidate came
    /// from; it certifies alignment and lends the actual applied Source.
    pub applied_managed_sources:
        Option<&'a worth_relational::facade::mvcc::RelationalAuthoredCoupledCandidate<'a>>,
    observation_refused: std::cell::RefCell<Option<crate::domain_computation::primary_graph::application_contribution::producer::observation_reads::WorthQueryProducerObservationReadDenial>>,
    read_authorization: Option<crate::domain_computation::authorization::BorrowedReadOperationAdmission<'a>>,
    predecessor: Option<crate::domain_computation::primary_graph::application_contribution::producer::observation_reads::ProducerPredecessorReadAccess>,
    original: Option<crate::domain_computation::primary_graph::application_contribution::producer::observation_reads::ProducerOriginalReadAccess>,
    current: Option<current_control::ordinary::CurrentSourceQueryControl<'a>>,
    refused: std::cell::RefCell<Option<SourceQueryObservationStop>>,
}

pub use worth_query_execution_lineage::facade::primary_graph::application_query::source_observation_stop::{
    SourceQueryObservationStop, SourceQueryPreparationPhase,
};

pub struct ObservedQuerySource<Query, Value> {
    pub value: Value,
    pub source: WorthQueryObservedSource<Query>,
    pub closure: Arc<SourceQueryReadClosure>,
}

impl<Schema: ApplicationSchema> SourceQueryReaderAccess<'_, Schema> {
    pub fn with_plan<Binding, R, E>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        intent: Binding::Input,
        context_use: &std::cell::Cell<
            crate::domain_computation::primary_graph::handler::DecisionContextUse,
        >,
        native: &mut ApplicationQueryNativeAccess<'_>,
        consume: impl for<'plan> FnOnce(
            SourceReadPlan<'plan, Schema, Binding>,
            &mut ApplicationQueryNativeAccess<'_>,
        ) -> Result<R, E>,
    ) -> Result<R, E>
    where
        Binding: ApplicationQueryBinding<Schema>,
        Binding::Input: ApplicationQueryIntent<Schema, Binding = Binding>,
        Binding::ScopeBinding: ApplicationQueryScopeResolution<Schema, Binding::PrincipalIdentity>,
        SourceValue<Schema, Binding>: WorthQueryApplicationProjection<Schema, Binding::Query>,
        E: From<SourceQueryObservationStop>,
    {
        if let Some(stop) = self.refused.borrow().as_ref() {
            return Err(stop.clone().into());
        }
        let result = self.with_plan_in_control::<Binding, R, E>(
            execution,
            intent,
            context_use,
            native,
            consume,
        );
        // Confirm even a consumer error: current control loss cannot become an
        // authored data outcome merely because the consumer returned a denial.
        self.confirm_current_control(execution, native, result)
            .map_err(|stop| E::from(self.record_control_refusal(stop)))?
    }

    fn with_plan_in_control<Binding, R, E>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        intent: Binding::Input,
        context_use: &std::cell::Cell<
            crate::domain_computation::primary_graph::handler::DecisionContextUse,
        >,
        native: &mut ApplicationQueryNativeAccess<'_>,
        consume: impl for<'plan> FnOnce(
            SourceReadPlan<'plan, Schema, Binding>,
            &mut ApplicationQueryNativeAccess<'_>,
        ) -> Result<R, E>,
    ) -> Result<R, E>
    where
        Binding: ApplicationQueryBinding<Schema>,
        Binding::Input: ApplicationQueryIntent<Schema, Binding = Binding>,
        Binding::ScopeBinding: ApplicationQueryScopeResolution<Schema, Binding::PrincipalIdentity>,
        SourceValue<Schema, Binding>: WorthQueryApplicationProjection<Schema, Binding::Query>,
        E: From<SourceQueryObservationStop>,
    {
        use SourceQueryObservationStop as Stop;
        context_use.set(context_use.get().source_query_permission());
        let application = self.application;
        self.confirm_current_control(execution, native, ())
            .map_err(|stop| self.record_control_refusal(stop))?;
        let selected = self.shared.selected();
        if !std::ptr::eq(application, selected.application()) {
            return Err(Stop::ForeignClosure.into());
        }
        let installed = application
            .installed_schema()
            .installed_query_binding::<Binding>()
            .map_err(Stop::Installation)?;
        let principal = selected
            .resolve_authenticated_principal_issued_with_native_access(
                execution,
                installed.principal_binding(),
                self.principal,
                self.request,
                WorthQueryPrincipalResolutionMode::Ordinary,
                None,
                native,
            )
            .map_err(|stop| self.record_control_refusal(Stop::Principal(stop)))?;
        let parameters = intent.parameters();
        let (field, value) = intent
            .into_scope()
            .into_field_parts(principal.principal().principal_identity());
        let scope = selected
            .resolve_query_scope_selector_issued_with_native_access(
                execution, field, &value, self.request, native,
            )
            .map_err(|stop| self.record_control_refusal(Stop::Scope(stop)))?;
        self.with_resolved_plan::<Binding, _, _>(
            execution, &installed, parameters, &principal, &scope, native, consume,
        )
    }
}
