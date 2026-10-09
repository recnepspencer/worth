use std::sync::Arc;

use super::super::managed_graph_execution::{
    WorthQueryManagedGraphExecution, WorthQueryRestoredManagedGraphExecutionParts,
};
use super::super::retained_graph_execution::WorthQueryRetainedManagedGraphExecution;
use super::recovery::{
    WorthQueryManagedGraphRestoreCleanupRequired, WorthQueryManagedGraphRestoreRecoveryKind,
    WorthQueryManagedGraphRestoreRecoveryRequired,
};
use crate::domain_computation::provider_session::graph_provider::bounded_step::{
    WorthQueryGraphProviderRestoreMemory, WorthQueryGraphProviderStepArtifactContext,
    WorthQueryOwnedGraphProviderExecution, WorthQueryProviderCheckpointRestoreInvocation,
};
use crate::domain_computation::WorthQueryGraphProviderCall;

pub(in crate::domain_computation::managed_run) enum WorthQueryManagedGraphRestoreOutcome {
    Pending(WorthQueryManagedGraphRestorePending),
    Denied(WorthQueryManagedGraphRestoreDenied),
    RecoveryRequired(WorthQueryManagedGraphRestoreRecoveryRequired),
}

pub(in crate::domain_computation::managed_run) struct WorthQueryManagedGraphRestorePending {
    retained: WorthQueryRetainedManagedGraphExecution,
    fresh_call: WorthQueryGraphProviderCall,
    contract: super::super::step_contract_admission::WorthQueryAdmittedManagedStepContract,
    restored_execution: WorthQueryOwnedGraphProviderExecution,
}

pub(in crate::domain_computation::managed_run) enum WorthQueryManagedGraphRestoreAbortOutcome {
    Aborted(WorthQueryRetainedManagedGraphExecution),
    RecoveryRequired(WorthQueryManagedGraphRestoreRecoveryRequired),
}

pub(in crate::domain_computation::managed_run) enum WorthQueryManagedGraphRestoreCommitOutcome {
    Restored(WorthQueryManagedGraphExecution),
    RecoveryRequired(WorthQueryManagedGraphRestoreRecoveryRequired),
}

pub(in crate::domain_computation::managed_run) struct WorthQueryManagedGraphRestoreDenied {
    detail: Arc<str>,
    retained: WorthQueryRetainedManagedGraphExecution,
}

pub(in crate::domain_computation::managed_run) fn restore(
    active_request: worth_execution::ExecutionRequest<'_, '_>,
    retained: WorthQueryRetainedManagedGraphExecution,
    fresh_call: WorthQueryGraphProviderCall,
    contract: super::super::step_contract_admission::WorthQueryAdmittedManagedStepContract,
) -> WorthQueryManagedGraphRestoreOutcome {
    let mut memory =
        WorthQueryGraphProviderRestoreMemory::new(active_request, retained.memory.clone());
    let invocation = retained.checkpoint.invoke_restore(&fresh_call, &mut memory);
    let unreturned_execution_release = memory.release_unreturned_execution();
    match invocation {
        WorthQueryProviderCheckpointRestoreInvocation::Returned(Err(failure)) => {
            if let Some(restored_execution) = unreturned_execution_release {
                WorthQueryManagedGraphRestoreOutcome::RecoveryRequired(
                    WorthQueryManagedGraphRestoreRecoveryRequired::retained_after_restored_release(
                        WorthQueryManagedGraphRestoreRecoveryKind::
                            ProviderRestoreRejectedAfterExecutionAdmission,
                        failure.detail(),
                        retained,
                        restored_execution,
                    ),
                )
            } else {
                WorthQueryManagedGraphRestoreOutcome::Denied(WorthQueryManagedGraphRestoreDenied {
                    detail: Arc::from(failure.detail()),
                    retained,
                })
            }
        }
        WorthQueryProviderCheckpointRestoreInvocation::Panicked => {
            WorthQueryManagedGraphRestoreOutcome::RecoveryRequired(
                match unreturned_execution_release {
                    Some(restored_execution) => WorthQueryManagedGraphRestoreRecoveryRequired::
                        retained_after_restored_release(
                            WorthQueryManagedGraphRestoreRecoveryKind::ProviderRestorePanicked,
                            "provider checkpoint restore panicked",
                            retained,
                            restored_execution,
                        ),
                    None => WorthQueryManagedGraphRestoreRecoveryRequired::retained(
                        WorthQueryManagedGraphRestoreRecoveryKind::ProviderRestorePanicked,
                        "provider checkpoint restore panicked",
                        retained,
                    ),
                },
            )
        }
        WorthQueryProviderCheckpointRestoreInvocation::Returned(Ok(execution)) => {
            WorthQueryManagedGraphRestoreOutcome::Pending(WorthQueryManagedGraphRestorePending {
                retained,
                fresh_call,
                contract,
                restored_execution: WorthQueryOwnedGraphProviderExecution::new(execution),
            })
        }
    }
}

impl WorthQueryManagedGraphRestorePending {
    pub(in crate::domain_computation::managed_run) fn checkpoint_evidence(
        &self,
    ) -> &crate::domain_computation::WorthQueryProviderCheckpointEvidence {
        self.retained.checkpoint_evidence()
    }

    pub(in crate::domain_computation::managed_run) fn abort(
        self,
    ) -> WorthQueryManagedGraphRestoreAbortOutcome {
        let Self {
            retained,
            fresh_call,
            contract: _,
            restored_execution,
        } = self;
        drop(fresh_call);
        let restored_execution = restored_execution.release();
        if restored_execution.recovery_required() {
            WorthQueryManagedGraphRestoreAbortOutcome::RecoveryRequired(
                WorthQueryManagedGraphRestoreRecoveryRequired::retained_after_restored_release(
                    WorthQueryManagedGraphRestoreRecoveryKind::
                        RestoredExecutionReleaseRecoveryRequired,
                    "replacement provider execution required physical-release recovery while restore was aborted",
                    retained,
                    restored_execution,
                ),
            )
        } else {
            WorthQueryManagedGraphRestoreAbortOutcome::Aborted(retained)
        }
    }

    pub(in crate::domain_computation::managed_run) fn into_cleanup(
        self,
    ) -> WorthQueryManagedGraphRestoreCleanupRequired {
        match self.abort() {
            WorthQueryManagedGraphRestoreAbortOutcome::Aborted(retained) => {
                WorthQueryManagedGraphRestoreCleanupRequired::retained(retained, None)
            }
            WorthQueryManagedGraphRestoreAbortOutcome::RecoveryRequired(recovery) => {
                recovery.into_cleanup()
            }
        }
    }

    pub(in crate::domain_computation::managed_run) fn commit(
        self,
        artifact_context: Option<WorthQueryGraphProviderStepArtifactContext>,
    ) -> WorthQueryManagedGraphRestoreCommitOutcome {
        let Self {
            retained,
            fresh_call,
            contract,
            restored_execution,
        } = self;
        finish_restore(
            retained,
            fresh_call,
            contract,
            artifact_context,
            restored_execution,
        )
    }
}

fn finish_restore(
    retained: WorthQueryRetainedManagedGraphExecution,
    fresh_call: WorthQueryGraphProviderCall,
    admitted_contract: super::super::step_contract_admission::WorthQueryAdmittedManagedStepContract,
    artifact_context: Option<WorthQueryGraphProviderStepArtifactContext>,
    restored_execution: WorthQueryOwnedGraphProviderExecution,
) -> WorthQueryManagedGraphRestoreCommitOutcome {
    let WorthQueryRetainedManagedGraphExecution {
        call,
        checkpoint,
        contract: _,
        memory,
        completed_work_units,
        applied_effect_count,
        peak_scratch_bytes,
        retained_bytes,
        projection,
        artifact_context: old_artifact_context,
        produced_artifact_count,
        retained_artifact_count,
        disposed_artifact_count,
    } = retained;
    let anchor = checkpoint.provider_anchor();
    let checkpoint_release = checkpoint.release();
    drop((call, old_artifact_context));
    if checkpoint_release.disposition().recovery_required() {
        let restored_execution = restored_execution.release();
        return WorthQueryManagedGraphRestoreCommitOutcome::RecoveryRequired(
            WorthQueryManagedGraphRestoreRecoveryRequired::released(
                WorthQueryManagedGraphRestoreRecoveryKind::CheckpointReleasePanicked,
                "provider checkpoint panicked after replacement execution was restored",
                checkpoint_release,
                restored_execution,
            ),
        );
    }
    WorthQueryManagedGraphRestoreCommitOutcome::Restored(WorthQueryManagedGraphExecution::restored(
        WorthQueryRestoredManagedGraphExecutionParts {
            call: fresh_call,
            execution: restored_execution.into_execution(),
            anchor,
            contract: admitted_contract,
            memory,
            completed_work_units,
            applied_effect_count,
            peak_scratch_bytes,
            retained_bytes,
            projection,
            artifact_context,
            produced_artifact_count,
            retained_artifact_count,
            disposed_artifact_count,
        },
    ))
}

impl WorthQueryManagedGraphRestoreDenied {
    pub(in crate::domain_computation::managed_run) fn detail(&self) -> &str {
        &self.detail
    }

    pub(in crate::domain_computation::managed_run) fn into_retained(
        self,
    ) -> WorthQueryRetainedManagedGraphExecution {
        self.retained
    }
}
