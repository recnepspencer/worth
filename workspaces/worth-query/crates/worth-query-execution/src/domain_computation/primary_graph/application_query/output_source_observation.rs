//! Governed source read inside the original producer reader and native borrow.

use super::*;
use crate::domain_computation::primary_graph::SharedSelectedProductOperation;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;
use crate::domain_computation::primary_graph::WorthQueryPrincipalResolutionMode;
use std::num::NonZeroUsize;
use std::sync::Arc;
use worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_declaration::facade::application_query::ApplicationQueryBinding;
use worth_query_declaration::facade::application_query::ApplicationQueryIntent;
use worth_query_declaration::facade::application_query::ApplicationQueryScopeBinding;
use worth_query_declaration::facade::application_query::ApplicationQueryScopeResolution;
use worth_query_declaration::facade::application_schema::ApplicationSchema;
use worth_query_declaration::facade::application_schema::ApplicationStructuredValueBinding;
use worth_relational::facade::mvcc::CompanionPreflightStop;

mod admitted;
mod permission;
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

/// Borrowed from the real producer execution entry. It carries no cached
/// permission: each call freshly resolves the installed query's own access.
pub(in crate::domain_computation::primary_graph) struct SourceQueryReaderAccess<'a, Schema> {
    pub(in crate::domain_computation::primary_graph) application:
        &'a WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    pub(in crate::domain_computation::primary_graph) shared:
        &'a SharedSelectedProductOperation<'a, Schema>,
    pub(in crate::domain_computation::primary_graph) principal:
        &'a WorthQueryAuthenticatedExternalPrincipal<Schema>,
    pub(in crate::domain_computation::primary_graph) request: &'a WorthQueryRequestScope,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) enum SourceQueryObservationStop {
    ManagedIndex(Arc<super::managed_index::WorthQueryManagedIndexDenial>),
    Installation(worth_query_installation::facade::WorthQueryApplicationQueryInstallationDenial),
    Principal(crate::domain_computation::primary_graph::WorthQueryPrincipalResolutionDenial),
    Scope(crate::domain_computation::primary_graph::WorthQueryEntityResolutionDenial),
    Admission(WorthQueryApplicationQueryAdmissionDenial),
    Preparation {
        phase: SourceQueryPreparationPhase,
        cause: WorthQueryApplicationQueryAdmissionDenial,
    },
    Read(WorthQueryAdmittedOneShotStop),
    Basis(crate::domain_computation::primary_graph::product_operation::SelectedQueryBasisRetentionStop),
    Missing,
    Ambiguous,
    ForeignClosure,
    SourceFacts(WorthQuerySourceExpectationDenial),
}

/// Diagnostic location on the original read; it grants no admission or state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) enum SourceQueryPreparationPhase {
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
        use SourceQueryObservationStop as Stop;
        context_use.set(context_use.get().source_query_permission());
        let application = self.application;
        let selected = self.shared.selected();
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
            .map_err(Stop::Principal)?;
        let (field, value) = selector.into_field_parts(principal.principal().principal_identity());
        let scope = selected
            .resolve_query_scope_issued_with_native_access(
                execution,
                field,
                value,
                self.request,
                native,
            )
            .map_err(Stop::Scope)?;
        self.with_resolved_plan::<Binding, _, _>(
            execution, &installed, parameters, &principal, &scope, native, consume,
        )
    }
}
