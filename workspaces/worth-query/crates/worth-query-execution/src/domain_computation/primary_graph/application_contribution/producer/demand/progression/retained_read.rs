//! The admitted output handle's frozen source meaning is the only fallback read.

use worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_installation::facade::ApplicationSchema;
use worth_relational::facade::mvcc::CompanionPreflightStop;

use super::denial;
use super::FamilySourceQuery;
use super::FamilySourceValue;
use super::WorthQueryOutputDemandDenial;
use super::WorthQueryOutputDemandDenialKind;
use super::WorthQueryProducerOutputFamily;
use crate::domain_computation::primary_graph::application_query::FreshQueryPermissionStop;
use crate::domain_computation::primary_graph::product_operation::SelectedQueryBasisRetentionStop;
use crate::domain_computation::primary_graph::WorthQueryApplicationOneShotDenial;
use crate::domain_computation::primary_graph::WorthQueryApplicationOneShotDenialKind;
use crate::domain_computation::primary_graph::WorthQueryApplicationOutputDemandSource;
use crate::domain_computation::primary_graph::WorthQueryApplicationProjection;
use crate::domain_computation::primary_graph::WorthQueryApplicationQueryAdmissionDenial;
use crate::domain_computation::primary_graph::WorthQueryApplicationQueryAdmissionDenialKind;
use crate::domain_computation::primary_graph::WorthQueryApplicationQueryControls;
use crate::domain_computation::primary_graph::WorthQueryEntityResolutionDenial;
use crate::domain_computation::primary_graph::WorthQueryObservedSource;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;
use crate::domain_computation::primary_graph::WorthQueryPrincipalResolutionDenial;

pub(super) fn read_retained_source<Schema, Family>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    retained: &WorthQueryObservedSource<FamilySourceQuery<Schema, Family>>,
    principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
    request: &WorthQueryRequestScope,
    branch: crate::basis::WorthQueryProductBranch,
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
    let controls = WorthQueryApplicationQueryControls::selected_read_one_shot(
        product,
        basis,
        installed_limits.maximum_results(),
        request,
    )
    .map_err(resource_denial)?;
    runtime
        .with_fresh_retained_query_permission(execution,
            &installed,
            None,
            principal,
            retained,
            shared.selected(),
            controls,
            |permission| {
                let plan = runtime
                    .finish_prepared_application_query_permission(execution, permission)
                    .map_err(query_admission_denial)?;
                let read = runtime
                    .execute_application_query_one_shot_admitted(execution, plan)
                    .map_err(|stop| {
use crate::domain_computation::primary_graph::application_query::WorthQueryAdmittedOneShotStop as Stop;
                        match stop {
                            Stop::Admission(stop) => resource_denial(stop),
                            Stop::Execution(stop) => query_execution_denial(stop),
                            Stop::WorkCounterOverflow
                            | Stop::WorkAccountingMismatch => work_denial(),
                        }
                    })?;
                crate::domain_computation::primary_graph::application_output_demand::observe_retained_source_selection(
                    retained.source_root(), retained.selected_product_commit().expect("retained Product source").clone(),
                    read_commit.clone());
                Ok(read.into_admitted_disclosed().into_output_demand_source())
            },
        )
        .map_err(|stop| match stop {
            FreshQueryPermissionStop::Principal(stop) => principal_denial(stop),
            FreshQueryPermissionStop::Scope(stop) => scope_denial(stop),
            FreshQueryPermissionStop::Query(stop) => query_admission_denial(stop),
            FreshQueryPermissionStop::Inspect(stop) => stop,
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
        CompanionPreflightStop::WorkExhausted { .. }
        | CompanionPreflightStop::WorkExhaustedWithoutRequirement { .. }
        | CompanionPreflightStop::WorkCounterOverflow => {
            WorthQueryOutputDemandDenialKind::WorkBudgetExceeded
        }
        _ => WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded,
    };
    denial(kind, "")
}
