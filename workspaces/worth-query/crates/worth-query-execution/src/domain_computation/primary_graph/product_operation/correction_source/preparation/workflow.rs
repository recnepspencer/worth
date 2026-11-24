//! Workflow-owned continuation selection from the genuine source occurrence.
use super::super::*;
use crate::domain_computation::primary_graph::workflow::correction::{
    retain_current_continuation, unavailable, PreparedWorkflowCorrection,
};
use worth_query_execution_attempt::facade::ports::PrimaryGraphProviderInternalPort as _;
use worth_query_execution_query::facade::ports::ApplicationRuntimeCoreInternalPort as _;
use worth_query_execution_runtime::facade::ports::AdvancementPhaseInternalPort as _;
use worth_query_execution_runtime::facade::ports::ProductBranchLeaseInternalPort as _;

pub(in crate::domain_computation::primary_graph::product_operation::correction_source) fn prepare_workflow<
    S,
    C,
>(
    target: &WorthQuerySelectedProductOperation<'_, S>,
    phase: &WorthQueryAdvancementPhase<'_>,
    source: &WorthQuerySelectedCorrectionSource<'_, S>,
) -> Result<Option<PreparedWorkflowCorrection>, WorthQueryCorrectionDerivationDenial>
where
    S: ApplicationSchema,
    C: ApplicationCorrectionBinding<S>,
{
    use WorthQueryCorrectionDerivationDenial as Denial;
    let Some(origin) = source
        .occurrence()
        .memory()
        .and_then(|value| value.commit_evidence().workflow_origin())
    else {
        return if C::Source::REQUIRES_WORKFLOW_AUTHORITY {
            Err(Denial::Workflow(unavailable(
                "performed source has no workflow origin",
            )))
        } else {
            Ok(None)
        };
    };
    let application = target.application();
    let execution = phase
        .execution_request_for(application.inner.runtime.product_runtime_ref())
        .map_err(|_| Denial::ForeignPhase)?;
    application.inner.runtime.primary_provider_ref().graph_ref().with_runtime(&execution, |runtime| {
        let basis = target.product().relational_basis();
        crate::domain_computation::primary_graph::index_currency::ensure_selected_field_indexes_admitted(
            &execution,
            runtime,
            basis,
            [Some(origin.layout().transition.identity_index_id)],
        )
        .map_err(|stop| match stop {
            crate::domain_computation::primary_graph::index_currency::SelectedFieldIndexAdmissionStop::Admission(
                worth_relational::facade::mvcc::CompanionPreflightStop::Execution(cause),
            ) => Denial::Workflow(
                crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationAttemptDenial::relational_read_stopped(
                    cause,
                    "workflow correction settlement index",
                ),
            ),
            _ => Denial::Workflow(unavailable("workflow correction settlement index is unavailable")),
        })?;
        let snapshot = crate::domain_computation::primary_graph::exact_basis_access::open_exact_basis_snapshot(
            runtime, basis,
        ).map_err(Denial::Basis)?;
        let disposition = retain_current_continuation(&execution, runtime, &snapshot, origin)
            .map(Some).map_err(Denial::Workflow);
        crate::relational_snapshot_release::release_query_snapshot(runtime, &snapshot);
        disposition
    })
}
