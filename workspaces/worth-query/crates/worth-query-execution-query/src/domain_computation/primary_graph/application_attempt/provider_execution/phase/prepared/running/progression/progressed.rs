use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
use worth_query_execution_lineage::facade::ports::ApplicationCleanupRequiredInternalPort as _;
use worth_query_execution_lineage::facade::ports::CleanupRequiredInternalPort as _;
use worth_query_execution_runtime::facade::ports::RunningDirectRunInternalPort as _;
pub mod cleanup_failure;
use worth_query_installation::facade::ApplicationSchema;

use crate::domain_computation::primary_graph::application_attempt::provider_execution::{
    recovery_evidence::unknown_commit_recovery_evidence,
};
use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationCommitOutcome;
use crate::domain_computation::primary_graph::application_attempt::provider_execution::WorthQueryProviderProgressionOutcome;
use super::WorthQueryProgressedApplicationCommit;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;
use crate::domain_computation::WorthQueryManagedRunTerminalKind;

pub fn finish_application_commit<Schema>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    phase: &WorthQueryAdvancementPhase<'_>,

    application: &dyn crate::ports::ApplicationRuntimeRef<Schema>,
    progressed: super::super::completion_storage::CompletedApplicationCommit,
) -> WorthQueryApplicationCommitOutcome
where
    Schema: ApplicationSchema,
{
    finish_application_commit_with_delivery(
        execution,
        phase,
        application.runtime_inner(),
        progressed,
        true,
    )
}
pub fn finish_coupled_application_commit<Schema: ApplicationSchema>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    phase: &WorthQueryAdvancementPhase<'_>,
    application: &dyn crate::ports::ApplicationRuntimeRef<Schema>,
    progressed: super::super::completion_storage::CompletedApplicationCommit,
) -> WorthQueryApplicationCommitOutcome {
    finish_application_commit_with_delivery(
        execution,
        phase,
        application.runtime_inner(),
        progressed,
        false,
    )
}
fn finish_application_commit_with_delivery<Schema: ApplicationSchema>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    phase: &WorthQueryAdvancementPhase<'_>,
    application: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    progressed: super::super::completion_storage::CompletedApplicationCommit,
    delivers_product: bool,
) -> WorthQueryApplicationCommitOutcome {
    let WorthQueryProgressedApplicationCommit {
        original_completion,
        outcome,
        workflow_settlement_publication,
        lease,
        running,
        cleanup,
        request,
    } = progressed.consume();
    if let Some(custody) = running.managed_home_custody() {
        custody.associate_original(crate::domain_computation::primary_graph::application_attempt::business_original::from_progression(&outcome));
    }
    let terminal = terminal_for(&outcome);
    let snapshot_release = lease.release_custody();
    application
        .runtime
        .primary_provider
        .observe_managed_application_cleanup();
    let completion = match cleanup.finish(
        &application.runtime.primary_provider,
        running,
        terminal,
        snapshot_release,
    ) {
        Ok(completion) => completion,
        Err(failure) => {
            let (evidence, original) =
                cleanup_failure::cleanup_failed_outcome(outcome, &original_completion);
            let mut retained = original_completion.cleanup_envelope().fill(
                evidence,
                failure,
                application.runtime.product_runtime.clone(),
                original,
                Box::new(application.runtime.bridge.conditional_operations()),
            );
            if let Some(support) = application.installed_program_support() {
                retained.retain_for_home(execution, support.obligation_history());
            }
            return WorthQueryApplicationCommitOutcome::CleanupRequired(retained);
        }
    };
    let committed = outcome.finish(completion).unwrap_or_else(|| {
        WorthQueryApplicationCommitOutcome::Indeterminate(unknown_commit_recovery_evidence(
            "provider progression outcome could not complete commit receipt",
        ))
    });
    // Park after World's commit resolved and before returning the answer.
    application
        .runtime
        .primary_provider
        .pause_before_commit_answer();
    // The resolved outcome (and any receipt) is dropped unread: World keeps
    // whatever it did, and only the answer is lost.
    let committed = if application
        .runtime
        .primary_provider
        .take_lost_commit_answer()
    {
        drop(committed);
        WorthQueryApplicationCommitOutcome::Indeterminate(unknown_commit_recovery_evidence(
            "application commit answer was lost before classification",
        ))
    } else {
        committed
    };
    if let WorthQueryApplicationCommitOutcome::Committed(receipt) = &committed {
        if delivers_product {
            crate::domain_computation::primary_graph::application_contribution::record_advancement_commit(receipt);
            let _ = application.flush_pending_coupled_outputs(execution);
        }
        if let Some(publication) = workflow_settlement_publication {
            publication.maintain(application, receipt);
        }
    }
    application.dispatch_committed_external_effect(execution, phase, committed, &request)
}

const fn terminal_for(
    outcome: &WorthQueryProviderProgressionOutcome,
) -> WorthQueryManagedRunTerminalKind {
    match outcome {
        WorthQueryProviderProgressionOutcome::OutputAdopted(_)
        | WorthQueryProviderProgressionOutcome::Committed(_)
        | WorthQueryProviderProgressionOutcome::RecoveredCommitted(_)
        | WorthQueryProviderProgressionOutcome::Stale(_)
        | WorthQueryProviderProgressionOutcome::ProductStale(_)
        | WorthQueryProviderProgressionOutcome::NoEffect(_) => {
            WorthQueryManagedRunTerminalKind::Completed
        }
        WorthQueryProviderProgressionOutcome::Cancelled => {
            WorthQueryManagedRunTerminalKind::Cancelled
        }
        WorthQueryProviderProgressionOutcome::TimedOut => WorthQueryManagedRunTerminalKind::Failed,
        WorthQueryProviderProgressionOutcome::CoupledFailure(_)
        | WorthQueryProviderProgressionOutcome::Denied(_)
        | WorthQueryProviderProgressionOutcome::ProductUnpublished(_)
        | WorthQueryProviderProgressionOutcome::ProductPreparationPending(_)
        | WorthQueryProviderProgressionOutcome::ProductPreparationRefused(_)
        | WorthQueryProviderProgressionOutcome::Deferred(_)
        | WorthQueryProviderProgressionOutcome::Aborted
        | WorthQueryProviderProgressionOutcome::Indeterminate(_) => {
            WorthQueryManagedRunTerminalKind::Failed
        }
    }
}
