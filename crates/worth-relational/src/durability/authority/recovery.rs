pub(super) mod admission;

use crate::diagnostics::data::{DiagnosticsArtifactKind, DiagnosticsScope};
use crate::durability::data::{DurabilityError, RecoveryCoverage, RecoveryPlan};
use crate::runtime::{RecoveryOutcome as RuntimeRecoveryOutcome, RelationalRuntime};

use super::authority_continuity::record_recovery_verification_counters;
use super::diagnostics::{recovery_checkpoint_selected, recovery_range_replayed};
use super::runtime_rebuild::rebuild_runtime_from_plan;
use super::DurabilityRecoveryAuthority;

impl<'runtime> DurabilityRecoveryAuthority<'runtime> {
    pub fn restore_native_checkpoint(
        &mut self,
        checkpoint: &crate::durability::data::RelationalNativeCheckpoint,
    ) -> Result<RuntimeRecoveryOutcome, DurabilityError> {
        let initial_schema_authority = self.runtime.initial_schema_authority_snapshot();
        let checkpoint =
            crate::durability::log::native_file_codec::decode_checkpoint(checkpoint.bytes())?
                .checkpoint;
        let plan = crate::durability::access::native_checkpoint_recovery_plan(
            self.runtime,
            checkpoint,
            crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
        );
        let outcome = self.recover(plan)?;
        self.runtime
            .restore_initial_schema_authority_after_recovery(initial_schema_authority);
        Ok(outcome)
    }

    pub fn recover(
        &mut self,
        plan: RecoveryPlan,
    ) -> Result<RuntimeRecoveryOutcome, DurabilityError> {
        let admitted = admit_recovery_or_emit(self.runtime, plan)?;
        let material = rebuild_admitted_recovery_or_emit(self.runtime, admitted)?;
        Ok(publish_recovered_runtime(self.runtime, material))
    }
}

struct RecoveredRuntimeMaterial {
    restored: RelationalRuntime,
    plan: RecoveryPlan,
    admission: admission::RecoveryAdmission,
}

fn admit_recovery_or_emit(
    runtime: &RelationalRuntime,
    plan: RecoveryPlan,
) -> Result<admission::AdmittedRecoveryPlan, DurabilityError> {
    admission::admit_recovery(runtime, plan).map_err(|rejection| {
        if rejection.records_verification_counters() {
            record_recovery_verification_counters(runtime, rejection.authority_continuity());
        }
        rejection.emit(runtime);
        rejection.into_error()
    })
}

fn rebuild_admitted_recovery_or_emit(
    runtime: &RelationalRuntime,
    admitted: admission::AdmittedRecoveryPlan,
) -> Result<RecoveredRuntimeMaterial, DurabilityError> {
    let admission = admitted.admission().clone();
    match rebuild_runtime_from_plan(admitted) {
        Ok((restored, plan)) => Ok(RecoveredRuntimeMaterial {
            restored,
            plan,
            admission,
        }),
        Err(error) => {
            admission.emit(runtime);
            Err(error)
        }
    }
}

fn publish_recovered_runtime(
    runtime: &mut RelationalRuntime,
    material: RecoveredRuntimeMaterial,
) -> RuntimeRecoveryOutcome {
    let RecoveredRuntimeMaterial {
        restored,
        plan,
        admission,
    } = material;
    let tail_commits = plan.tail_commit_count();
    let checkpoint_commits = plan
        .checkpoint
        .as_ref()
        .map(|checkpoint| checkpoint.envelopes.len())
        .unwrap_or(0);
    restored.durability.set_store(plan.store.clone());
    record_recovery_verification_counters(&restored, &plan.authority_continuity);
    restored.publication_authority().push_bounded_diagnostic(
        DiagnosticsScope::History,
        admission.artifact_kind(),
        vec![admission.into_entry()],
    );
    restored.publication_authority().push_bounded_diagnostic(
        DiagnosticsScope::History,
        DiagnosticsArtifactKind::MinimalSummary,
        vec![
            recovery_checkpoint_selected(
                plan.cursor.checkpoint_id,
                &plan.integrity_report.skipped_corrupt_checkpoints,
            ),
            recovery_range_replayed(&plan.cursor.segment_ids, tail_commits),
        ],
    );
    let outcome = RuntimeRecoveryOutcome {
        recovered_commits: restored.history.recorded_commit_envelope_count(),
        latest_commit: restored.history().latest_commit(),
        restored_branches: restored.history.branch_count(),
        cursor: plan.cursor,
        coverage: RecoveryCoverage {
            checkpoint_commits,
            replayed_tail_commits: tail_commits,
            recovered_through_commit: restored.history().latest_commit(),
        },
        integrity_report: plan.integrity_report,
    };
    *runtime = restored;
    outcome
}

impl RelationalRuntime {
    pub(crate) fn rebuild_runtime_from_plan(
        &self,
        plan: RecoveryPlan,
    ) -> Result<RelationalRuntime, DurabilityError> {
        let admitted =
            admission::admit_recovery(self, plan).map_err(|rejection| rejection.into_error())?;
        rebuild_runtime_from_plan(admitted).map(|(restored, plan)| {
            restored.durability.set_store(plan.store);
            restored
        })
    }
}
