//! Retain only continuation data that the fresh temporal owner consumes.
//! An indeterminate answer keeps the original World association the cold
//! consumer joined, or stays unclassified until World's readmission decides it.
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
    use super::super::signal_decision_reentry::{
        WorthQueryConditionalContinuationIdentity as Continuation,
        WorthQueryRetainedConditionalDecision as Decision,
    };
    let Some(Continuation {
        key,
        intent,
        product,
    }) = wake.continuation_identity
    else {
        return Ok(());
    };
    let Some(support) = runtime.installed_program_support() else {
        wake.continuation_identity = None;
        return Ok(());
    };
    let history = support.lifecycle();
    if !history.has_medium() {
        return Ok(());
    }
    let already_indeterminate = history.is_indeterminate(&key);
    let posture = match &mut wake.decision {
        Decision::OperationCleanupRequired(_, cleanup) => {
            let identity = cleanup
                .obligation_identity()
                .ok_or(QueryObligationDenial::UnknownObligation)?;
            if cleanup.pending_kind().is_none()
                && history.settle_managed_cleanup_discharge(request, identity)?
            {
                cleanup.acknowledge_logical_discharge();
                if !already_indeterminate {
                    wake.continuation_identity = None;
                }
                return Ok(());
            }
            if already_indeterminate {
                return Ok(());
            }
            Posture::CleanupRequired(identity)
        }
        Decision::OperationRetryable(_, _) => {
            if already_indeterminate {
                return Ok(());
            }
            Posture::Retryable
        }
        Decision::OperationCommitRetryable(_, _) => {
            if already_indeterminate {
                return Ok(());
            }
            Posture::CommitRetryable
        }
        Decision::OperationProductStale(_, _) => {
            if already_indeterminate {
                return Ok(());
            }
            Posture::ProductStale
        }
        Decision::OperationIndeterminate(_, _) => {
            Posture::Indeterminate(history.retained_indeterminate_original(&key))
        }
        Decision::OperationCommitted(_, _)
        | Decision::OperationAlreadyCommitted(_, _)
        | Decision::OperationRetiredNoEffect(_) => {
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
        | Decision::InterruptedDuringReentry
        | Decision::Failed(_) => return Ok(()),
    };
    let product = product.ok_or(QueryObligationDenial::ForeignHome)?;
    history.record_conditional_decision(
        request,
        Record::Retain(QueryConditionalDecision {
            key,
            intent,
            product,
            posture,
        }),
    )?;
    Ok(())
}
