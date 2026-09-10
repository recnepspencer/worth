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
    application: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    progressed: WorthQueryProgressedApplicationCommit,
) -> WorthQueryApplicationCommitOutcome
where
    Schema: ApplicationSchema,
{
    let WorthQueryProgressedApplicationCommit {
        outcome,
        lease,
        running,
        cleanup,
    } = progressed;
    let terminal = terminal_for(&outcome);
    let snapshot_released = lease.release();
    application
        .primary_provider
        .observe_managed_application_cleanup();
    let completion = match cleanup.finish(running, terminal, snapshot_released) {
        Ok(completion) => completion,
        Err(()) => {
            return match outcome {
                WorthQueryProviderProgressionOutcome::ProductUnpublished(unpublished) => {
                    WorthQueryApplicationCommitOutcome::ProductUnpublished(unpublished)
                }
                WorthQueryProviderProgressionOutcome::ProductStale(stale) => {
                    WorthQueryApplicationCommitOutcome::ProductStale(stale)
                }
                WorthQueryProviderProgressionOutcome::NoEffect(no_effect) => {
                    WorthQueryApplicationCommitOutcome::NoEffect(
                        crate::domain_computation::primary_graph::WorthQueryApplicationNoEffect::from_world(
                            no_effect,
                        ),
                    )
                }
                _ => WorthQueryApplicationCommitOutcome::Indeterminate(
                    unknown_commit_recovery_evidence(
                        "managed mutation run failed to finish after provider progression",
                    ),
                ),
            }
        }
    };
    let committed = outcome.finish(completion).unwrap_or_else(|| {
        WorthQueryApplicationCommitOutcome::Indeterminate(unknown_commit_recovery_evidence(
            "provider progression outcome could not complete commit receipt",
        ))
    });
    if let WorthQueryApplicationCommitOutcome::Committed(receipt) = &committed {
        let touched = receipt
            .mutation_work()
            .into_iter()
            .flat_map(|work| work.touched_records())
            .map(|identity| identity.record().clone())
            .collect::<Vec<_>>();
        application.maintain_conditional_commit(receipt.commit_reference(), touched);
    }
    application.dispatch_committed_external_effect(committed)
}

const fn terminal_for(
    outcome: &WorthQueryProviderProgressionOutcome,
) -> WorthQueryManagedRunTerminalKind {
    match outcome {
        WorthQueryProviderProgressionOutcome::Committed(_)
        | WorthQueryProviderProgressionOutcome::AlreadyCommitted(_)
        | WorthQueryProviderProgressionOutcome::Stale(_)
        | WorthQueryProviderProgressionOutcome::ProductStale(_)
        | WorthQueryProviderProgressionOutcome::NoEffect(_)
        | WorthQueryProviderProgressionOutcome::SettlementDeferred(_) => {
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
