//! Workflow-owned continuation selection from the genuine source occurrence.
use super::super::*;
use crate::domain_computation::primary_graph::workflow::correction::{
    retain_current_continuation, unavailable, PreparedWorkflowCorrection,
};

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
    let Some(origin) = source.occurrence().commit_evidence().workflow_origin() else {
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
        .execution_request_for(&application.product_runtime)
        .map_err(|_| Denial::ForeignPhase)?;
    application.primary_provider.graph.with_runtime_mut(&execution, |runtime| {
        let snapshot = crate::domain_computation::primary_graph::exact_basis_access::open_exact_basis_snapshot(
            runtime, target.product().relational_basis(),
        ).map_err(Denial::Basis)?;
        let disposition = retain_current_continuation(&execution, runtime, &snapshot, origin)
            .map(Some).map_err(Denial::Workflow);
        crate::relational_snapshot_release::release_query_snapshot(runtime, &snapshot);
        disposition
    })
}
