//! Original source readmission retains actual issued guards through the whole page action.
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope,
};
use worth_query_declaration::facade::{
    application_query::ApplicationHistoryQueryBinding, application_schema::ApplicationSchema,
};
use worth_query_installation::facade::WorthQueryInstalledApplicationHistoryQuery;

use super::super::{
    admission::FreshQueryPermissionStop,
    disclosure::WorthQueryPendingApplicationQueryGovernance as Pending,
};
use super::{
    HistoryMetadataAdmission, SubjectAccess, SubjectPlan,
    WorthQueryApplicationHistoryDenial as Denial,
    WorthQueryCapturedApplicationHistorySubjects as Subjects,
};
use crate::domain_computation::primary_graph::{
    WorthQueryPrimaryGraphApplicationRuntime, WorthQueryProductQueryControls,
    WorthQuerySelectedProductOperation,
};

mod opening;
mod paging;

type PendingDecisions = (Option<Pending>, Option<Pending>);

#[allow(clippy::too_many_arguments)]
fn with_session<'application, Schema, History, Output>(
    application: &'application WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    execution: worth_execution::ExecutionRequest<'_, '_>,
    history: &'application WorthQueryInstalledApplicationHistoryQuery<Schema, History>,
    external: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
    subjects: &Subjects<Schema, History>,
    security: WorthQuerySelectedProductOperation<'application, Schema>,
    truth: &WorthQuerySelectedProductOperation<'application, Schema>,
    request: &'application WorthQueryRequestScope,
    authorize: impl for<'fresh> FnOnce(
        &SubjectAccess<'fresh, Schema, History>,
    ) -> Result<PendingDecisions, Denial>,
    operation: impl for<'fresh> FnOnce(
        SubjectPlan<'fresh, Schema, History>,
        HistoryMetadataAdmission<History>,
    ) -> Result<Output, Denial>,
) -> Result<Output, Denial>
where
    Schema: ApplicationSchema,
    History: ApplicationHistoryQueryBinding<Schema>,
{
    if !std::ptr::eq(application, truth.application()) {
        return Err(Denial::SubjectMismatch);
    }
    let shared_truth = truth
        .retain_selection(&execution)
        .map_err(Denial::Selection)?;
    let (_, controls) = security
        .retained_query_controls(
            &execution,
            shared_truth,
            History::IDENTITY,
            WorthQueryProductQueryControls::new(std::num::NonZeroUsize::MIN, request),
        )
        .map_err(Denial::Query)?;
    application
        .with_fresh_retained_query_access(
            &execution,
            history.subject_read(),
            None,
            external,
            subjects.owner.observed.source(),
            truth,
            controls,
            |access, source, _principal, _scope| {
                // A later occupant with the same descriptive scope key cannot lend
                // disclosure to the captured generation, even when ordinary readmission permits that key.
                if access.scope().entity_id() != subjects.owner.observed.source().model_root {
                    return Err(FreshQueryPermissionStop::Inspect(Denial::SubjectMismatch));
                }
                let (subject_pending, history_pending) =
                    authorize(&access).map_err(FreshQueryPermissionStop::Inspect)?;
                let (parameters, controls) = source.into_parts();
                let mut plan = application
                    .finish_application_query_readmission(
                        &execution,
                        history.subject_read().query(),
                        &access,
                        parameters,
                        controls,
                        subject_pending,
                    )
                    .map_err(FreshQueryPermissionStop::Query)?;
                let metadata = application
                    .admit_application_history_metadata(
                        &execution,
                        history,
                        &mut plan,
                        history_pending,
                    )
                    .map_err(FreshQueryPermissionStop::Inspect)?;
                operation(plan, metadata).map_err(FreshQueryPermissionStop::Inspect)
            },
        )
        .map_err(|cause| match cause {
            FreshQueryPermissionStop::Principal(cause) => Denial::Principal(cause),
            FreshQueryPermissionStop::Scope(cause) => Denial::Scope(cause),
            FreshQueryPermissionStop::Query(cause) => Denial::Query(cause),
            FreshQueryPermissionStop::Inspect(cause) => cause,
        })
}

fn request_for<'scope, Schema: ApplicationSchema>(
    application: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'scope>,
) -> Result<worth_execution::ExecutionRequest<'scope, 'scope>, Denial> {
    phase.execution_request_for(&application.product_runtime).map_err(|_| {
        Denial::Authorization(crate::domain_computation::primary_graph::WorthQueryOperationAuthorizationDenial::new(
            crate::domain_computation::primary_graph::WorthQueryOperationAuthorizationDenialKind::Refused(
                crate::domain_computation::authorization::WorthQueryOperationAuthorizationRefusalKind::ForeignPhase,
            ), "history-phase",
        ))
    })
}
