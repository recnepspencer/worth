//! Retain only continuation data that the fresh temporal owner consumes.
use super::super::super::{
    query_obligations::{
        QueryConditionalDecision, QueryConditionalDecisionRecord as Record,
        QueryConditionalPosture as Posture, QueryObligationDenial,
    },
    WorthQueryPrimaryGraphApplicationRuntime,
};
use super::super::signal_decision_reentry::WorthQueryRetainedConditionalWake;

pub(in crate::domain_computation::primary_graph::conditional_operation) fn flush_record<
    Schema: worth_query_installation::facade::ApplicationSchema,
>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    request: &worth_execution::ExecutionRequest<'_, '_>,
    wake: &mut WorthQueryRetainedConditionalWake,
) -> Result<(), QueryObligationDenial> {
    use super::super::signal_decision_reentry::WorthQueryRetainedConditionalDecision as Decision;
    let Some((key, intent)) = wake.continuation_identity else {
        return Ok(());
    };
    let Some(support) = runtime.installed_program_support() else {
        wake.continuation_identity = None;
        return Ok(());
    };
    let history = support.lifecycle();
    let posture = match &mut wake.decision {
        Decision::OperationCleanupRequired(_, cleanup) => {
            let identity = cleanup
                .obligation_identity()
                .ok_or(QueryObligationDenial::UnknownObligation)?;
            if cleanup.pending_kind().is_none()
                && history.settle_managed_cleanup_discharge(request, identity)?
            {
                cleanup.acknowledge_logical_discharge();
                wake.continuation_identity = None;
                return Ok(());
            }
            Posture::CleanupRequired(identity)
        }
        Decision::OperationRetryable(_, _) => Posture::Retryable,
        Decision::OperationCommitRetryable(_, _) => Posture::CommitRetryable,
        Decision::OperationProductStale(_, _) => Posture::ProductStale,
        Decision::OperationCommitted(_) | Decision::OperationAlreadyCommitted(_) => {
            history.record_conditional_decision(request, Record::Finish { key, intent })?;
            wake.continuation_identity = None;
            return Ok(());
        }
        Decision::Eligible(_)
        | Decision::Suppressed(_)
        | Decision::Deferred(_)
        | Decision::OperationPreparationPending(_, _)
        | Decision::OperationPreparationRefused(_, _)
        | Decision::OperationProductUnpublished(_, _, _)
        | Decision::OperationNoEffect(_, _)
        | Decision::OperationExecutionControlRetryable(_, _)
        | Decision::OperationBackpressured(_, _)
        | Decision::OperationTerminalFailure(_, _)
        | Decision::OperationControlStopped(_, _)
        | Decision::OperationIndeterminate(_, _)
        | Decision::InterruptedDuringReentry
        | Decision::Failed(_) => return Ok(()),
    };
    history.record_conditional_decision(
        request,
        Record::Retain(QueryConditionalDecision {
            key,
            intent,
            posture,
        }),
    )?;
    Ok(())
}
