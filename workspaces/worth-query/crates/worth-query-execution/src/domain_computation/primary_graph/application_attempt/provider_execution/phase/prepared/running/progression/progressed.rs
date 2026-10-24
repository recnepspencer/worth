use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
mod cleanup_failure;
use worth_query_installation::facade::ApplicationSchema;

use crate::domain_computation::primary_graph::application_attempt::provider_execution::{
    recovery_evidence::unknown_commit_recovery_evidence,
};
use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationCommitOutcome;
use crate::domain_computation::primary_graph::application_attempt::provider_execution::WorthQueryProviderProgressionOutcome;
use super::WorthQueryProgressedApplicationCommit;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;
use crate::domain_computation::WorthQueryManagedRunTerminalKind;

pub(in crate::domain_computation::primary_graph::application_attempt::provider_execution) fn finish_application_commit<
    Schema,
>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    phase: &WorthQueryAdvancementPhase<'_>,

    application: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    progressed: WorthQueryProgressedApplicationCommit,
) -> WorthQueryApplicationCommitOutcome
where
    Schema: ApplicationSchema,
{
    let WorthQueryProgressedApplicationCommit {
        outcome,
        workflow_settlement_publication,
        lease,
        running,
        cleanup,
        request,
    } = progressed;
    let terminal = terminal_for(&outcome);
    let snapshot_release = lease.release_custody();
    application
        .primary_provider
        .observe_managed_application_cleanup();
    let completion = match cleanup.finish(
        &application.primary_provider,
        running,
        terminal,
        snapshot_release,
    ) {
        Ok(completion) => completion,
        Err(failure) => {
            let evidence = cleanup_failure::cleanup_failed_outcome(outcome);
            return WorthQueryApplicationCommitOutcome::CleanupRequired(Box::new(
                crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationCleanupRequired::new(evidence, failure, application.product_runtime.clone(), application.bridge.conditional_operations()),
            ));
        }
    };
    let committed = outcome.finish(completion).unwrap_or_else(|| {
        WorthQueryApplicationCommitOutcome::Indeterminate(unknown_commit_recovery_evidence(
            "provider progression outcome could not complete commit receipt",
        ))
    });
    if let WorthQueryApplicationCommitOutcome::Committed(receipt) = &committed {
        crate::domain_computation::primary_graph::application_contribution::record_advancement_commit(receipt);
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
        WorthQueryProviderProgressionOutcome::Committed(_)
        | WorthQueryProviderProgressionOutcome::AlreadyCommitted(_)
        | WorthQueryProviderProgressionOutcome::Stale(_)
        | WorthQueryProviderProgressionOutcome::ProductStale(_)
        | WorthQueryProviderProgressionOutcome::NoEffect(_) => {
            WorthQueryManagedRunTerminalKind::Completed
        }
        WorthQueryProviderProgressionOutcome::Cancelled => {
            WorthQueryManagedRunTerminalKind::Cancelled
        }
        WorthQueryProviderProgressionOutcome::TimedOut => WorthQueryManagedRunTerminalKind::Failed,
        WorthQueryProviderProgressionOutcome::Denied(_)
        | WorthQueryProviderProgressionOutcome::ProductUnpublished(_)
        | WorthQueryProviderProgressionOutcome::Deferred(_)
        | WorthQueryProviderProgressionOutcome::Aborted
        | WorthQueryProviderProgressionOutcome::Indeterminate(_) => {
            WorthQueryManagedRunTerminalKind::Failed
        }
    }
}

#[cfg(test)]
#[path = "association_tests.rs"]
mod association_tests;
