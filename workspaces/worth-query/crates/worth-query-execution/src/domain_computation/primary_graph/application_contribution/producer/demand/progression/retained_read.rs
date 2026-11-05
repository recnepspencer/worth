//! The admitted output handle's frozen source meaning is the only fallback read.

use crate::domain_computation::primary_graph::application_query::FreshScopedOps;
use crate::domain_computation::primary_graph::application_query::PreparedPermissionScopedOps;
use crate::domain_computation::primary_graph::application_query::RetainedPermissionScopedOps;
use crate::domain_computation::primary_graph::application_query::WorkLimitOps;
use crate::domain_computation::primary_graph::product_operation::SharedQueryBasisScopedOps;
use worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_installation::facade::ApplicationSchema;
use worth_relational::facade::mvcc::CompanionPreflightStop;

use super::denial;
use super::FamilySourceQuery;
use super::FamilySourceValue;
use super::WorthQueryProducerOutputFamily;
use crate::domain_computation::primary_graph::application_output_demand::denial::WorthQueryOutputDemandDenial;
use crate::domain_computation::primary_graph::application_output_demand::denial::WorthQueryOutputDemandDenialKind;
use crate::domain_computation::primary_graph::application_query::FreshQueryPermissionStop;
use crate::domain_computation::primary_graph::product_operation::document_history::HistoryProducerInvocation;
use crate::domain_computation::primary_graph::vocabulary::application_query::denial::WorthQueryApplicationQueryAdmissionDenial;
use crate::domain_computation::primary_graph::vocabulary::application_query::denial::WorthQueryApplicationQueryAdmissionDenialKind;
use crate::domain_computation::primary_graph::vocabulary::application_query::one_shot_denial::WorthQueryApplicationOneShotDenial;
use crate::domain_computation::primary_graph::vocabulary::application_query::one_shot_denial::WorthQueryApplicationOneShotDenialKind;
use crate::domain_computation::primary_graph::vocabulary::entity_resolution_denial::WorthQueryEntityResolutionDenial;
use crate::domain_computation::primary_graph::vocabulary::product_operation::shared_query_basis::SelectedQueryBasisRetentionStop;
use crate::domain_computation::primary_graph::WorthQueryApplicationOutputDemandSource;
use crate::domain_computation::primary_graph::WorthQueryApplicationProjection;
use crate::domain_computation::primary_graph::WorthQueryApplicationQueryControls;
use crate::domain_computation::primary_graph::WorthQueryObservedSource;
use crate::domain_computation::vocabulary::primary_graph::resolution_denial::WorthQueryPrincipalResolutionDenial;
use worth_query_declaration::facade::{
    application_query::{ApplicationQueryBinding, ApplicationQueryScopeBinding},
    application_schema::ApplicationStructuredValueBinding,
};

type Source<S, F> = <F as WorthQueryProducerOutputFamily<S>>::Source;
type Parameters<S, F> = <<Source<S, F> as ApplicationQueryBinding<S>>::ParameterBinding as ApplicationStructuredValueBinding>::Value;
type Scope<S, F> =
    <<Source<S, F> as ApplicationQueryBinding<S>>::ScopeBinding as ApplicationQueryScopeBinding<
        S,
    >>::Scope;
type Permission<'a, S, F> =
    crate::domain_computation::primary_graph::application_query::PreparedApplicationQueryPermission<
        'a,
        S,
        FamilySourceQuery<S, F>,
        Parameters<S, F>,
        FamilySourceValue<S, F>,
        <Source<S, F> as ApplicationQueryBinding<S>>::Principal,
        <Source<S, F> as ApplicationQueryBinding<S>>::PrincipalIdentity,
        Scope<S, F>,
    >;

pub(in crate::domain_computation::primary_graph::application_contribution::producer) fn authorize_history_source<
    Schema,
    Family,
>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &dyn crate::domain_computation::primary_graph::application_contribution::host::ApplicationContributionHost<Schema>,
    retained: &WorthQueryObservedSource<FamilySourceQuery<Schema, Family>>,
    principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
    request: &WorthQueryRequestScope,
    history: Option<HistoryProducerInvocation<'_>>,
) -> Result<(), WorthQueryOutputDemandDenial>
where
    Schema: ApplicationSchema + 'static,
    Family: WorthQueryProducerOutputFamily<Schema>,
    FamilySourceValue<Schema, Family>: 'static,
    FamilySourceQuery<Schema, Family>: 'static,
{
    let Some(history) = history else {
        return Ok(());
    };
    runtime
        .authorize_retained_history_query::<Source<Schema, Family>>(
            execution, history, retained, principal, request,
        )
        .map_err(history_permission_denial)
}

fn history_permission_denial(
    stop: crate::domain_computation::primary_graph::application_query::HistoryRetainedPermissionStop,
) -> WorthQueryOutputDemandDenial {
    use super::super::super::invocation::history_source_denial;
    use crate::domain_computation::primary_graph::application_query::HistoryRetainedPermissionStop as Stop;
    let denial = match stop {
        Stop::Current(stop) => history_source_denial(stop),
        Stop::Resource(stop) => resource_denial(stop),
        Stop::Installation(kind) => denial(
            WorthQueryOutputDemandDenialKind::SourceQueryInstallation(kind),
            "history source query",
        ),
        Stop::Permission(stop) => match stop {
            FreshQueryPermissionStop::Principal(stop) => principal_denial(stop),
            FreshQueryPermissionStop::Scope(stop) => scope_denial(stop),
            FreshQueryPermissionStop::Query(stop) => query_admission_denial(stop),
            FreshQueryPermissionStop::Inspect(stop) => history_source_denial(stop),
        },
    };
    // The sealed error came from this call's C-only permission gate. It is
    // neither a D projection failure nor an intrinsic producer decision.
    denial.with_recovery_posture(
        crate::domain_computation::primary_graph::application_output_demand::denial::WorthQueryOutputDemandRecoveryPosture::Retryable,
    )
}

pub(super) fn read_retained_source<Schema, Family>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &dyn crate::domain_computation::primary_graph::application_contribution::host::ApplicationContributionHost<Schema>,
    retained: &WorthQueryObservedSource<FamilySourceQuery<Schema, Family>>,
    principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
    request: &WorthQueryRequestScope,
    branch: crate::basis::WorthQueryProductBranch,
    history: Option<HistoryProducerInvocation<'_>>,
) -> Result<
    WorthQueryApplicationOutputDemandSource<
        FamilySourceQuery<Schema, Family>,
        FamilySourceValue<Schema, Family>,
    >,
    WorthQueryOutputDemandDenial,
>
where
    Schema: ApplicationSchema + 'static,
    Family: WorthQueryProducerOutputFamily<Schema>,
    FamilySourceValue<Schema, Family>:
        WorthQueryApplicationProjection<Schema, FamilySourceQuery<Schema, Family>> + 'static,
    FamilySourceQuery<Schema, Family>: 'static,
{
    with_retained_source_permission::<Schema, Family, _>(
        execution,
        runtime,
        retained,
        principal,
        request,
        branch,
        history,
        |permission, _, read_commit, reader| {
            let plan = runtime
                .finish_prepared_application_query_permission(execution, permission)
                .map_err(query_admission_denial)?;
            let read = reader.read_admitted_result::<Family::Source>(execution, plan,
                &mut crate::domain_computation::primary_graph::application_query::ApplicationQueryNativeAccess::acquire())
                .map_err(|stop| crate::domain_computation::primary_graph::invariant_projection::WorthQueryOutputAvailabilityDenial::source_query_stop(stop).demand_stop())?;
            crate::domain_computation::primary_graph::application_output_demand::observe_retained_source_selection(
                retained.source_root(), retained.selected_product_commit().expect("retained Product source").clone(), read_commit.clone());
            super::super::super::invocation::confirm_history_source(
                history,
                execution,
                runtime,
                read.into_admitted_disclosed().into_output_demand_source(),
            )
        },
    )
}

fn with_retained_source_permission<Schema, Family, R>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &dyn crate::domain_computation::primary_graph::application_contribution::host::ApplicationContributionHost<Schema>,
    retained: &WorthQueryObservedSource<FamilySourceQuery<Schema, Family>>,
    principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
    request: &WorthQueryRequestScope,
    branch: crate::basis::WorthQueryProductBranch,
    history: Option<HistoryProducerInvocation<'_>>,
    inspect: impl for<'prepared> FnOnce(Permission<'prepared, Schema, Family>,
        Option<&'prepared crate::domain_computation::primary_graph::application_query::WorthQueryApplicationQueryBasisCustody>,
        &worth_runtime_world::facade::CompositeCommitIdentity,
        &crate::domain_computation::primary_graph::application_query::SourceQueryReaderAccess<'_, Schema>) -> Result<R, WorthQueryOutputDemandDenial>,
) -> Result<R, WorthQueryOutputDemandDenial>
where
    Schema: ApplicationSchema + 'static,
    Family: WorthQueryProducerOutputFamily<Schema>,
    FamilySourceValue<Schema, Family>: 'static,
    FamilySourceQuery<Schema, Family>: 'static,
{
    // Schema and installed-query getters, limit headers, the three bounded
    // minimum operands, and the callback's retained-source argument precede
    // the Query owner's one-shot execution allowance.

    let installed = runtime
        .installed_schema()
        .installed_query_binding::<Family::Source>()
        .map_err(|error| {
            denial(
                WorthQueryOutputDemandDenialKind::SourceQueryInstallation(error.kind()),
                "",
            )
        })?;
    let selected = runtime
        .on_branch(branch)
        .select_with_request(execution)
        .map_err(|stop| WorthQueryOutputDemandDenial::product_selection(stop, ""))?;
    let shared = selected
        .prepare_shared_query_basis()
        .map_err(|(_, stop)| resource_denial(stop))?;
    let installed_limits = runtime.resolve_application_query_limits(installed.limits());
    let control = crate::domain_computation::primary_graph::application_contribution::producer::execution::select_history_control(execution, runtime, history)
        .map_err(|stop| stop.into_output_denial())?;
    let control_basis = control.as_ref().map(|control| {
        crate::domain_computation::primary_graph::application_query::WorthQueryApplicationQueryBasisCustody::from_selected(runtime, control)
    }).transpose().map_err(|stop| match stop {
        SelectedQueryBasisRetentionStop::Basis => denial(WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable, ""),
        SelectedQueryBasisRetentionStop::Admission(stop) => resource_denial(stop),
    })?;
    let (product, basis) = runtime
        .retain_selected_query_basis_admitted(&shared)
        .map_err(|stop| match stop {
            SelectedQueryBasisRetentionStop::Basis => denial(
                WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable,
                "",
            ),
            SelectedQueryBasisRetentionStop::Admission(stop) => resource_denial(stop),
        })?;
    let read_commit = product.selected_commit().clone();
    let controls = if let Some(control) = &control {
        WorthQueryApplicationQueryControls::selected_read_one_shot_with_control(
            control
                .selected()
                .product()
                .read_lease_ref()
                .retained_clone(),
            product,
            basis,
            installed_limits.maximum_results(),
            request,
        )
    } else {
        WorthQueryApplicationQueryControls::selected_read_one_shot(
            product,
            basis,
            installed_limits.maximum_results(),
            request,
        )
        .map_err(resource_denial)?
    }
    .with_selection_origin(shared.selected().selection_origin())
    .with_original_settled_producer(shared.selected().original_settled_producer().cloned());
    let source_reader =
        crate::domain_computation::primary_graph::application_query::SourceQueryReaderAccess::new(
            runtime,
            &shared,
            control.as_ref(),
            history,
            principal,
            request,
        )?;
    runtime
        .with_fresh_retained_query_permission(
            execution,
            &installed,
            None,
            principal,
            retained,
            control.as_ref().unwrap_or(&shared).selected(),
            control.as_ref().map(|_| shared.selected()),
            controls,
            |permission| {
                if let Some(history) = history {
                    permission
                        .require_history_principal(history)
                        .map_err(query_admission_denial)?;
                    permission
                        .require_original_scope(retained)
                        .map_err(query_admission_denial)?;
                }
                super::super::super::invocation::confirm_history_source(
                    history,
                    execution,
                    runtime,
                    (),
                )?;
                inspect(permission, control_basis.as_ref(), &read_commit, &source_reader)
            },
        )
        .map_err(|stop| {
            let denial = match stop {
                FreshQueryPermissionStop::Principal(stop) => principal_denial(stop),
                FreshQueryPermissionStop::Scope(stop) => scope_denial(stop),
                FreshQueryPermissionStop::Query(stop) => query_admission_denial(stop),
                // The callback owns D execution/projection, with its original posture.
                FreshQueryPermissionStop::Inspect(stop) => return stop,
            };
            if history.is_some() {
                denial.with_recovery_posture(crate::domain_computation::primary_graph::application_output_demand::denial::WorthQueryOutputDemandRecoveryPosture::Retryable)
            } else { denial }
        })
}

fn principal_denial(stop: WorthQueryPrincipalResolutionDenial) -> WorthQueryOutputDemandDenial {
    denial(
        WorthQueryOutputDemandDenialKind::of_principal_resolution(stop.kind()),
        "",
    )
}

fn scope_denial(stop: WorthQueryEntityResolutionDenial) -> WorthQueryOutputDemandDenial {
    denial(
        WorthQueryOutputDemandDenialKind::of_scope_resolution(stop.kind()),
        "",
    )
}

fn query_admission_denial(
    stop: WorthQueryApplicationQueryAdmissionDenial,
) -> WorthQueryOutputDemandDenial {
    use WorthQueryApplicationQueryAdmissionDenialKind as Kind;
    let kind = match stop.kind() {
        Kind::Cancelled => WorthQueryOutputDemandDenialKind::Cancelled,
        Kind::DeadlineExceeded => WorthQueryOutputDemandDenialKind::TimedOut,
        Kind::Authorization(kind) => {
            WorthQueryOutputDemandDenialKind::of_request_authorization(kind)
        }
        Kind::WorkLimitExceeded | Kind::CanonicalWorkDenied => {
            WorthQueryOutputDemandDenialKind::WorkBudgetExceeded
        }
        Kind::ReadmissionPreparationMemoryExhausted | Kind::RetentionCapacityExhausted => {
            WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded
        }
        other => WorthQueryOutputDemandDenialKind::SourceQueryAdmission(other),
    };
    denial(kind, "")
}

fn query_execution_denial(
    stop: WorthQueryApplicationOneShotDenial,
) -> WorthQueryOutputDemandDenial {
    use WorthQueryApplicationOneShotDenialKind as Kind;
    let kind = match stop.kind() {
        Kind::Cancelled => WorthQueryOutputDemandDenialKind::Cancelled,
        Kind::DeadlineExceeded => WorthQueryOutputDemandDenialKind::TimedOut,
        Kind::Authorization(kind) => {
            WorthQueryOutputDemandDenialKind::of_request_authorization(kind)
        }
        Kind::WorkLimitExceeded => WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
        Kind::RetentionCapacityExhausted => {
            WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded
        }
        other => WorthQueryOutputDemandDenialKind::SourceQueryExecution(other),
    };
    denial(kind, "")
}

fn work_denial() -> WorthQueryOutputDemandDenial {
    denial(WorthQueryOutputDemandDenialKind::WorkBudgetExceeded, "")
}

fn resource_denial(stop: CompanionPreflightStop) -> WorthQueryOutputDemandDenial {
    let kind = match stop {
        CompanionPreflightStop::WorkCounterOverflow => {
            WorthQueryOutputDemandDenialKind::WorkBudgetExceeded
        }
        _ => WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded,
    };
    denial(kind, "")
}
