//! Current request authority is required even for a read-only negative completion.
use super::super::{authentication::*, schema::NumericSchema};
use super::program::{Program, Root};
use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};
use worth_query_host::facade::{
    application_entry::*, application_installation::WorthQueryProgramApplicationRuntime,
    primary_graph::*,
};
pub(super) fn refuse(
    application: &WorthQueryProgramApplicationRuntime<NumericSchema, Program>,
    output: &mut WorthQueryApplicationProgramOutputHandle<NumericSchema, Program, Root>,
) {
    let scope = request_scope();
    let principal = authenticate_operator(application.installed_schema(), &scope);
    let foreign = super::courts::install(false);
    let foreign_principal = authenticate_operator(foreign.installed_schema(), &scope);
    assert!(matches!(
        output.finish_unavailable(
            application,
            &foreign.runtime().request(&foreign_principal, &scope)
        ),
        Err(WorthQueryRequiredOutputPreparationDenial::Demand(
            WorthQueryApplicationOutputDemandDenial::FreshRequestMismatch
        ))
    ));
    let branch = application
        .with_host_advancement(|phase| {
            application
                .runtime()
                .branches()
                .fork(application.current_world())
                .components(|parts| parts.fork_relational().reuse_exact_signal_basis())
                .create(&phase)
                .unwrap()
        })
        .unwrap();
    let refusal = output
        .finish_unavailable(
            application,
            &application
                .runtime()
                .request(&principal, &scope)
                .on_branch(branch),
        )
        .err()
        .unwrap();
    assert!(
        matches!(&refusal, WorthQueryRequiredOutputPreparationDenial::Demand(
        WorthQueryApplicationOutputDemandDenial::Demand(denial))
        if denial.kind() == WorthQueryOutputDemandDenialKind::ForeignSource),
        "{refusal:?}"
    );
    let expired = WorthQueryRequestScope::new(
        Some(std::time::Instant::now() - std::time::Duration::from_secs(1)),
        WorthQueryCancellationSource::new().token(),
        worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let refusal = output
        .finish_unavailable(
            application,
            &application.runtime().request(&principal, &expired),
        )
        .err()
        .unwrap();
    assert!(
        matches!(&refusal, WorthQueryRequiredOutputPreparationDenial::Demand(
        WorthQueryApplicationOutputDemandDenial::Demand(denial))
        if matches!(denial.kind(), WorthQueryOutputDemandDenialKind::ExecutionRequest(
            worth_query_host::facade::application_contribution::WorthQueryAdvancementDenial::Interrupted(
                worth_query_host::facade::application_contribution::WorthQueryManagedComputationInterruption::DeadlineExceeded)))),
        "{refusal:?}"
    );
    let cancellation = WorthQueryCancellationSource::new();
    let stopped = WorthQueryRequestScope::new(None, cancellation.token(), worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system());
    cancellation.cancel();
    let refusal = output
        .finish_unavailable(
            application,
            &application.runtime().request(&principal, &stopped),
        )
        .err()
        .unwrap();
    assert!(
        matches!(&refusal, WorthQueryRequiredOutputPreparationDenial::Demand(
        WorthQueryApplicationOutputDemandDenial::Demand(denial))
        if matches!(denial.kind(), WorthQueryOutputDemandDenialKind::ExecutionRequest(
            worth_query_host::facade::application_contribution::WorthQueryAdvancementDenial::Interrupted(
                worth_query_host::facade::application_contribution::WorthQueryManagedComputationInterruption::Cancelled)))),
        "{refusal:?}"
    );
}
