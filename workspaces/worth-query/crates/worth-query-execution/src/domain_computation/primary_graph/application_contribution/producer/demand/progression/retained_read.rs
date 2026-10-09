//! The admitted output handle's frozen source meaning is the only fallback read.

use std::num::NonZeroUsize;

use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope,
};
use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryInstalledQueryBindingAdmissionStop,
};
use worth_relational::facade::mvcc::CompanionPreflightStop;

use super::{
    denial, FamilySourceQuery, FamilySourceValue, InvalidationEditAdmission,
    WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind, WorthQueryProducerOutputFamily,
};
use crate::domain_computation::primary_graph::{
    application_query::FreshQueryPermissionStop,
    product_operation::SelectedQueryBasisRetentionStop, WorthQueryApplicationOneShotDenial,
    WorthQueryApplicationOneShotDenialKind, WorthQueryApplicationOutputDemandSource,
    WorthQueryApplicationProjection, WorthQueryApplicationQueryAdmissionDenial,
    WorthQueryApplicationQueryAdmissionDenialKind, WorthQueryApplicationQueryControls,
    WorthQueryEntityResolutionDenial, WorthQueryObservedSource,
    WorthQueryPrimaryGraphApplicationRuntime, WorthQueryPrincipalResolutionDenial,
};

pub(super) fn read_retained_source<Schema, Family>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    retained: &WorthQueryObservedSource<FamilySourceQuery<Schema, Family>>,
    principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
    request: &WorthQueryRequestScope,
    branch: crate::basis::WorthQueryProductBranch,
    limits: crate::domain_computation::execution_runtime::WorthQueryOutputDemandLimits,
    request_admission: &mut InvalidationEditAdmission,
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
    request_admission
        .charge_external_work(12)
        .map_err(|_| work_denial())?;
    let installed = runtime
        .installed_schema()
        .installed_query_binding_admitted::<Family::Source, _>(&mut |work, bytes| {
            request_admission.charge_external_work(work)?;
            request_admission.admit_read_scratch(bytes)
        })
        .map_err(|stop| match stop {
            WorthQueryInstalledQueryBindingAdmissionStop::Installation(error) => denial(
                WorthQueryOutputDemandDenialKind::SourceQueryInstallation(error.kind()),
                "",
            ),
            WorthQueryInstalledQueryBindingAdmissionStop::Admission(stop) => resource_denial(stop),
            WorthQueryInstalledQueryBindingAdmissionStop::AccountingOverflow => work_denial(),
        })?;
    let selected = runtime
        .on_branch(branch)
        .select()
        .map_err(|stop| WorthQueryOutputDemandDenial::product_selection(stop, ""))?;
    let shared = selected
        .prepare_shared_query_basis(request_admission)
        .map_err(|(_, stop)| resource_denial(stop))?;
    let installed_limits = runtime.resolve_application_query_limits(installed.limits());
    let maximum_work = NonZeroUsize::new(
        installed_limits
            .maximum_work()
            .get()
            .min(limits.producer_work())
            .min(request_admission.remaining_work()),
    )
    .ok_or_else(work_denial)?;
    let (product, basis) = runtime
        .retain_selected_query_basis_admitted(&shared, request_admission)
        .map_err(|stop| match stop {
            SelectedQueryBasisRetentionStop::Basis => denial(
                WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable,
                "",
            ),
            SelectedQueryBasisRetentionStop::Admission(stop) => resource_denial(stop),
        })?;
    #[cfg(feature = "test-query-execution-observer")]
    let read_commit = product.selected_commit().clone();
    let controls = WorthQueryApplicationQueryControls::selected_read_one_shot(
        product,
        basis,
        installed_limits.maximum_results(),
        maximum_work,
        request,
        request_admission,
    )
    .map_err(resource_denial)?;
    runtime
        .with_fresh_retained_query_permission(
            &installed,
            None,
            principal,
            retained,
            shared.selected(),
            controls,
            request_admission,
            |permission, admission| {
                let plan = runtime
                    .finish_prepared_application_query_permission(permission, admission)
                    .map_err(query_admission_denial)?;
                #[cfg(feature = "test-query-execution-observer")]
                let before = admission.charged_work();
                let read = runtime
                    .execute_application_query_one_shot_admitted(plan, admission)
                    .map_err(|stop| {
                        use crate::domain_computation::primary_graph::application_query::WorthQueryAdmittedOneShotStop as Stop;
                        match stop {
                            Stop::Admission(stop) => resource_denial(stop),
                            Stop::Execution(stop) => query_execution_denial(stop),
                            Stop::WorkUnavailable | Stop::WorkCounterOverflow
                            | Stop::WorkAccountingMismatch => work_denial(),
                        }
                    })?;
                #[cfg(feature = "test-query-execution-observer")]
                super::caller_pass_observation::observe_read_debit(
                    read.receipt().work().total_work_units() as u64, admission.charged_work() - before,
                );
                #[cfg(feature="test-query-execution-observer")]
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
        | CompanionPreflightStop::WorkCounterOverflow => {
            WorthQueryOutputDemandDenialKind::WorkBudgetExceeded
        }
        _ => WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded,
    };
    denial(kind, "")
}
