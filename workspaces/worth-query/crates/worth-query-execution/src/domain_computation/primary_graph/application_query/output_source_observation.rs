//! Governed source read inside the original producer reader and native borrow.

use super::*;
use crate::domain_computation::primary_graph::{
    SharedSelectedProductOperation, WorthQueryPrimaryGraphApplicationRuntime,
    WorthQueryPrincipalResolutionMode,
};
use std::{num::NonZeroUsize, sync::Arc};
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope,
};
use worth_query_declaration::facade::{
    application_query::{
        ApplicationQueryBinding, ApplicationQueryIntent, ApplicationQueryScopeResolution,
    },
    application_schema::{ApplicationSchema, ApplicationStructuredValueBinding},
};
use worth_relational::facade::mvcc::CompanionPreflightStop;

mod read_closure;
pub(in crate::domain_computation::primary_graph) use read_closure::SourceQueryReadClosure;

type SourceValue<S, B> =
    <<B as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value;

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
    Installation(
        worth_query_installation::facade::WorthQueryInstalledQueryBindingAdmissionStop<
            CompanionPreflightStop,
        >,
    ),
    Principal(crate::domain_computation::primary_graph::WorthQueryPrincipalResolutionDenial),
    Scope(crate::domain_computation::primary_graph::WorthQueryEntityResolutionDenial),
    Admission(WorthQueryApplicationQueryAdmissionDenial),
    Preparation {
        phase: SourceQueryPreparationPhase,
        charged_work: u64,
        remaining_work: usize,
        query_maximum_work: usize,
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
    pub(in crate::domain_computation::primary_graph) fn read<Binding>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        intent: Binding::Input,
        context_use: &std::cell::Cell<
            crate::domain_computation::primary_graph::handler::DecisionContextUse,
        >,
        native: &mut ApplicationQueryNativeAccess<'_>,
        admission: &mut crate::domain_computation::primary_graph::InvalidationEditAdmission,
    ) -> Result<
        ObservedQuerySource<Binding::Query, SourceValue<Schema, Binding>>,
        SourceQueryObservationStop,
    >
    where
        Binding: ApplicationQueryBinding<Schema>,
        Binding::Input: ApplicationQueryIntent<Schema, Binding = Binding>,
        Binding::ScopeBinding: ApplicationQueryScopeResolution<Schema, Binding::PrincipalIdentity>,
        SourceValue<Schema, Binding>: WorthQueryApplicationProjection<Schema, Binding::Query>,
    {
        use SourceQueryObservationStop as Stop;
        let preparation_start = admission.charged_bytes();
        context_use.set(context_use.get().source_query_permission());
        let application = self.application;
        let selected = self.shared.selected();
        if !std::ptr::eq(application, selected.application()) {
            return Err(Stop::ForeignClosure);
        }
        let installed = application
            .installed_schema()
            .installed_query_binding_admitted::<Binding, _>(&mut |work, bytes| {
                admission.admit_read_scratch(bytes)?;
                admission.charge_external_work(work)
            })
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
                admission,
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
                admission,
                native,
            )
            .map_err(Stop::Scope)?;
        let access =
            WorthQueryApplicationQueryAccessContext::new(principal.principal(), scope.scope());
        let query = installed.query();
        let limits = application.resolve_application_query_limits(installed.limits());
        let maximum = NonZeroUsize::new(admission.remaining_work()).ok_or_else(|| {
            Stop::Admission(WorthQueryApplicationQueryAdmissionDenial::new(
                WorthQueryApplicationQueryAdmissionDenialKind::WorkLimitExceeded,
                query.name(),
            ))
        })?;
        let query_maximum = NonZeroUsize::new(query.read_graph().maximum_result_count())
            .unwrap_or(NonZeroUsize::MAX);
        let (product, basis) = application
            .retain_selected_query_basis_admitted(self.shared, admission)
            .map_err(Stop::Basis)?;
        let controls = WorthQueryApplicationQueryControls::selected_read_one_shot(
            product,
            basis,
            limits.maximum_results().min(query_maximum),
            limits.maximum_work().min(maximum),
            self.request,
            admission,
        )
        .map_err(|stop| Stop::Read(WorthQueryAdmittedOneShotStop::Admission(stop)))?;
        // Use the existing typed parameter admission/canonical engine and its
        // installed canonical ceiling; no caller digest grants query meaning.
        let (parameters, controls) = application
            .prepare_source_query_admission(query, &access, parameters, controls, admission)
            .map_err(|cause| Stop::Preparation {
                phase: SourceQueryPreparationPhase::Parameters,
                charged_work: admission.charged_work(),
                remaining_work: admission.remaining_work(),
                query_maximum_work: limits.maximum_work().get(),
                cause,
            })?;
        let permission = application
            .prepare_application_query_permission_with_parameters(
                execution,
                query,
                access,
                query.identity(),
                parameters,
                controls,
                selected,
                self.request,
                &principal,
                &scope,
                admission,
                native,
            )
            .map_err(|cause| Stop::Preparation {
                phase: SourceQueryPreparationPhase::Permission,
                charged_work: admission.charged_work(),
                remaining_work: admission.remaining_work(),
                query_maximum_work: limits.maximum_work().get(),
                cause,
            })?;
        let plan = application
            .finish_prepared_application_query_permission_with_native_access(
                execution, permission, admission, native,
            )
            .map_err(|cause| Stop::Preparation {
                phase: SourceQueryPreparationPhase::Session,
                charged_work: admission.charged_work(),
                remaining_work: admission.remaining_work(),
                query_maximum_work: limits.maximum_work().get(),
                cause,
            })?;
        let completed = application
            .execute_application_query_one_shot_admitted_with_native_access(
                execution, plan, admission, native,
            )
            .map_err(Stop::Read)?;
        let (rows, disclosure) = completed.read.into_admitted_disclosed().into_parts();
        if rows.is_empty() {
            return Err(Stop::Missing);
        }
        if rows.len() != 1 {
            return Err(Stop::Ambiguous);
        }
        let (sources, affinity, receipt) = disclosure.into_parts();
        if sources.len() != 1
            || !affinity
                .as_ref()
                .is_some_and(|affinity| affinity.admits(self.principal, self.request))
            || !completed
                .authorization
                .belongs_to_session(receipt.read_completion().session_identity())
        {
            return Err(Stop::ForeignClosure);
        }
        let source = sources
            .into_iter()
            .next()
            .expect("exactly one native source");
        let facts = source.admit_fact_materialization(
            &application.primary_provider.graph.layout, admission, installed.query().name(),
        ).and_then(|prepared| prepared.into_facts(
            crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StorageControl::new(execution.allocation_policy(), Some(self.request)),
        )).map_err(Stop::SourceFacts)?;
        let graph = application
            .runtime
            .primary_graph()
            .ok_or(Stop::ForeignClosure)?;
        let selected_native = native.with_runtime_mut(
            graph,
            execution,
            || Stop::ForeignClosure,
            |runtime| {
                runtime
                    .read_truth()
                    .positioned_snapshot(selected.application_basis().snapshot_handle())
                    .map_err(|_| Stop::ForeignClosure)
            },
        )?;
        let closure = SourceQueryReadClosure::retain(
            application,
            receipt,
            completed.authorization,
            &selected_native,
            admission
                .charged_bytes()
                .checked_sub(preparation_start)
                .ok_or(Stop::ForeignClosure)?,
            facts,
            admission,
        )
        .map_err(|stop| Stop::Read(WorthQueryAdmittedOneShotStop::Admission(stop)))?;
        Ok(ObservedQuerySource {
            value: rows.into_iter().next().expect("exactly one disclosed row"),
            source,
            closure,
        })
    }
}
