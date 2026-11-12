//! Reapplication must still address the generations returned by its active undo.
use super::super::{
    WorthQueryCorrectionDerivationDenial as Denial, WorthQuerySelectedProductOperation,
};
use crate::domain_computation::application_aftermath::WorthQueryRecordedCorrectionSource;
use crate::domain_computation::primary_graph::{
    admitted_capture::checkpoint, exact_basis_access::open_exact_basis_snapshot,
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationOutputPosture as Posture,
};
use worth_execution::ExecutionRequest;
use worth_query_installation::facade::ApplicationSchema;
use worth_relational::facade::runtime::RelationalBorrowedRecordReadDenial as ReadDenial;

pub(super) fn require<S: ApplicationSchema>(
    target: &WorthQuerySelectedProductOperation<'_, S>,
    execution: ExecutionRequest<'_, '_>,
    active: &WorthQueryRecordedCorrectionSource,
) -> Result<(), Denial> {
    let provider = &target.application().runtime.primary_provider;
    // The caller already selected this occurrence's ancestry. This is an exact
    // completed-occurrence lookup, not another history traversal.
    let occurrence = provider
        .observe_completed_application(active.relational())
        .ok_or(Denial::SemanticMaterialUnavailable)?;
    let publication = occurrence
        .committed_product_publication()
        .composite_commit();
    if publication != active.world() {
        return Err(Denial::ForeignAlternative);
    }
    provider.graph.with_runtime_mut(&execution, |runtime| {
        let snapshot = open_exact_basis_snapshot(runtime, target.product().relational_basis())
            .map_err(Denial::Basis)?;
        let result = (|| {
            let view = runtime.read_truth().project_snapshot(&snapshot)
                .ok_or(Denial::SemanticMaterialUnavailable)?;
            let work = view.exact_entity_state_read_work_bound()
                .ok_or(Denial::SemanticMaterialUnavailable)?;
            let (result, _) = execution.run_native_with_checkpoint(|context| {
                for (_, posture, _, entity) in occurrence.commit_evidence()
                    .output_correspondence().native_witness_roles()
                {
                    checkpoint(context, work).map_err(|cause| Denial::Workflow(
                        WorthQueryApplicationAttemptDenial::request_read_stopped(
                            cause, "reapplication output generation")))?;
                    if posture == Posture::Retire {
                        continue;
                    }
                    let live = view.read_exact_entity_state(&execution, entity, |metadata, _| metadata)
                        .map_err(|cause| match cause {
                            ReadDenial::ExactBasisRequired => Denial::SemanticMaterialUnavailable,
                            ReadDenial::Execution(cause) => Denial::Workflow(
                                WorthQueryApplicationAttemptDenial::relational_read_stopped(
                                    cause, "reapplication output generation")),
                        })?;
                    if live.is_none() {
                        return Err(Denial::SourceDiverged);
                    }
                }
                Ok(())
            }).map_err(|cause| Denial::RestoredSupport(
                super::super::restored_support::WorthQueryCorrectionRestoredSupportDenial::Request(cause)))?;
            result
        })();
        crate::relational_snapshot_release::release_query_snapshot(runtime, &snapshot);
        result
    })
}
