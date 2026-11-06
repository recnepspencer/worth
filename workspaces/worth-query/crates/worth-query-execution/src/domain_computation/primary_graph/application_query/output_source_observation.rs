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
mod current_control;
mod permission;
use permission::{CurrentSourceRefusal, SourceScopePosture};
mod read_closure;
mod same_scope;
mod single;
pub(in crate::domain_computation::primary_graph) use read_closure::SourceQueryReadClosure;

type SourceValue<S, B> =
    <<B as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value;
pub(in crate::domain_computation::primary_graph) type SourceReadPlan<'a, S, B> = WorthQueryAdmittedApplicationQueryPlan<
    'a, S, <B as ApplicationQueryBinding<S>>::Query,
    <<B as ApplicationQueryBinding<S>>::ParameterBinding as ApplicationStructuredValueBinding>::Value,
    SourceValue<S, B>, <B as ApplicationQueryBinding<S>>::Principal,
    <B as ApplicationQueryBinding<S>>::PrincipalIdentity,
    <<B as ApplicationQueryBinding<S>>::ScopeBinding as ApplicationQueryScopeBinding<S>>::Scope,
>;

/// Borrowed from the real producer execution entry. Each call freshly admits
/// its installed query; historical truth carries no current permission.
pub(in crate::domain_computation::primary_graph) struct SourceQueryReaderAccess<'a, Schema> {
    pub(in crate::domain_computation::primary_graph) application:
        &'a WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    pub(in crate::domain_computation::primary_graph) shared:
        &'a SharedSelectedProductOperation<'a, Schema>,
    pub(in crate::domain_computation::primary_graph) principal:
        &'a WorthQueryAuthenticatedExternalPrincipal<Schema>,
    pub(in crate::domain_computation::primary_graph) request: &'a WorthQueryRequestScope,
    history: Option<current_control::HistorySourceQueryControl<'a, Schema>>,
    refused: std::cell::RefCell<Option<SourceQueryObservationStop>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SourceQueryObservationStop {
    CurrentControl(crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial),
    ManagedIndex(Arc<super::managed_index::WorthQueryManagedIndexDenial>),
    Installation(worth_query_installation::facade::WorthQueryApplicationQueryInstallationDenial),
    Principal(crate::domain_computation::primary_graph::WorthQueryPrincipalResolutionDenial),
    Scope(crate::domain_computation::primary_graph::WorthQueryEntityResolutionDenial),
    Admission(WorthQueryApplicationQueryAdmissionDenial),
    Preparation { phase: SourceQueryPreparationPhase, cause: WorthQueryApplicationQueryAdmissionDenial },
    Read(WorthQueryAdmittedOneShotStop),
    Basis(crate::domain_computation::primary_graph::product_operation::SelectedQueryBasisRetentionStop),
    Missing,
    Ambiguous,
    ForeignClosure,
    SourceFacts(WorthQuerySourceExpectationDenial),
}

/// Diagnostic location on the original read; it grants no admission or state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SourceQueryPreparationPhase {
    Parameters,
    Permission,
    Session,
}

pub(in crate::domain_computation::primary_graph) struct ObservedQuerySource<Query, Value> {
    pub(in crate::domain_computation::primary_graph) value: Value,
    pub(in crate::domain_computation::primary_graph) source: WorthQueryObservedSource<Query>,
    pub(in crate::domain_computation::primary_graph) closure: Arc<SourceQueryReadClosure>,
}

impl<Schema: ApplicationSchema> SourceQueryReaderAccess<'_, Schema> {
    pub(in crate::domain_computation::primary_graph) fn with_plan<Binding, R, E>(
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
        self.confirm_control(execution, native, result)
            .map_err(|stop| E::from(self.record_history_refusal(stop)))?
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
        self.confirm_control(execution, native, ())
            .map_err(|stop| self.record_history_refusal(stop))?;
        let selected = self
            .history
            .as_ref()
            .map_or(self.shared, |history| history.current)
            .selected();
        if !std::ptr::eq(application, selected.application()) {
            return Err(Stop::ForeignClosure.into());
        }
        let installed = application
            .installed_schema()
            .installed_query_binding::<Binding>()
            .map_err(Stop::Installation)?;
        let parameters = intent.parameters();
        let selector = intent.into_scope();
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
            .map_err(|stop| self.record_history_refusal(Stop::Principal(stop)))?;
        let (field, value) = selector.into_field_parts(principal.principal().principal_identity());
        let scope = selected.resolve_query_scope_selector_issued_with_native_access(
            execution,
            field,
            &value,
            self.request,
            native,
        );
        let (scope, posture) = match scope {
            Ok(scope) if self.history.as_ref().is_some_and(|history| history.invocation.historical_branch().is_some()) => {
                // Resolve D privately. A C business key can name a different
                // generation; its grant never authorizes the D entity. Failure
                // here keeps the old C path, including its authorization order.
                let truth = self.shared.selected().resolve_query_scope_selector_issued_with_native_access(
                    execution, field, &value, self.request, native,
                );
                match truth {
                    Ok(truth) if truth.scope().entity_id() != scope.scope().entity_id() =>
                        (truth, SourceScopePosture::Historical(CurrentSourceRefusal::DifferentGeneration)),
                    _ => (scope, SourceScopePosture::Current),
                }
            }
            Ok(scope) => (scope, SourceScopePosture::Current),
            Err(absent) if self.history.is_some() && matches!(absent.kind(),
                crate::domain_computation::primary_graph::WorthQueryEntityResolutionDenialKind::Refused(
                    crate::domain_computation::primary_graph::WorthQueryEntityResolutionRefusalKind::UnknownEntity)) => {
                // Resolution is private. Until C authorizes, neither a D lookup
                // failure nor a C denial reveals more than the old C refusal.
                let scope = self.shared.selected().resolve_query_scope_selector_issued_with_native_access(
                    execution, field, &value, self.request, native,
                ).map_err(|_| self.record_history_refusal(Stop::Scope(absent.clone())))?;
                (scope, SourceScopePosture::Historical(CurrentSourceRefusal::Absent(absent)))
            }
            Err(stop) => return Err(self.record_history_refusal(Stop::Scope(stop)).into()),
        };
        self.with_resolved_plan::<Binding, _, _>(
            execution, &installed, parameters, &principal, &scope, posture, native, consume,
        )
    }
}
