use crate::durability::data::RelationalRecoveryStop;
pub(crate) use codec_transition::{
    AdmittedRecoveredCodecPublication, RecoveredCodecPublicationCustody,
};
pub(super) mod admission;
pub(super) mod codec_transition;
mod declared_schema;

use crate::diagnostics::data::{DiagnosticsArtifactKind, DiagnosticsScope};
use crate::durability::data::{DurabilityError, RecoveryCoverage, RecoveryPlan};
use crate::runtime::{RecoveryOutcome as RuntimeRecoveryOutcome, RelationalRuntime};

use super::authority_continuity::record_recovery_verification_counters;
use super::diagnostics::{recovery_checkpoint_selected, recovery_range_replayed};
use super::runtime_rebuild::rebuild_runtime_from_plan;
use super::DurabilityRecoveryAuthority;
use super::RecoveredRelationalRuntimeAuthority;

impl<'runtime> DurabilityRecoveryAuthority<'runtime> {
    pub fn restore_native_checkpoint_with_authority(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        checkpoint: &crate::durability::data::RelationalNativeCheckpoint,
    ) -> Result<(RuntimeRecoveryOutcome, RecoveredRelationalRuntimeAuthority), RelationalRecoveryStop>
    {
        let outcome = self.restore_native_checkpoint(execution, checkpoint)?;
        Ok((outcome, self.restored_runtime_authority()))
    }

    fn restored_runtime_authority(&self) -> RecoveredRelationalRuntimeAuthority {
        let recovered_branch_images = self
            .runtime
            .history()
            .branch_cells_snapshot()
            .into_iter()
            .map(|cell| (cell.branch_id, (cell.observation, cell.truth_version)))
            .collect();
        RecoveredRelationalRuntimeAuthority {
            runtime_instance_id: self.runtime.runtime_instance_id(),
            recovered_branch_images,
            codec_transition: None,
            performed_codec_retirement: None,
        }
    }

    pub fn restore_native_checkpoint(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        checkpoint: &crate::durability::data::RelationalNativeCheckpoint,
    ) -> Result<RuntimeRecoveryOutcome, RelationalRecoveryStop> {
        crate::execution::check_request(execution).map_err(RelationalRecoveryStop::Request)?;
        let native_bytes_read = checkpoint.bytes().len();
        let checkpoint = crate::durability::log::native_checkpoint_codec::decode_checkpoint(
            execution,
            checkpoint.bytes(),
        )?
        .checkpoint;
        self.restore_decoded_native_checkpoint(execution, checkpoint, native_bytes_read)
    }

    pub(in crate::durability::authority) fn restore_decoded_native_checkpoint_with_authority(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        checkpoint: crate::durability::data::DurableCheckpoint,
        native_bytes_read: usize,
    ) -> Result<(RuntimeRecoveryOutcome, RecoveredRelationalRuntimeAuthority), RelationalRecoveryStop>
    {
        let outcome =
            self.restore_decoded_native_checkpoint(execution, checkpoint, native_bytes_read)?;
        Ok((outcome, self.restored_runtime_authority()))
    }

    fn restore_decoded_native_checkpoint(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        checkpoint: crate::durability::data::DurableCheckpoint,
        native_bytes_read: usize,
    ) -> Result<RuntimeRecoveryOutcome, RelationalRecoveryStop> {
        let initial_schema_authority = self.runtime.initial_schema_authority_snapshot();
        let native_envelopes_readmitted = checkpoint.envelopes.len();
        let plan = crate::durability::access::native_checkpoint_recovery_plan(
            self.runtime,
            checkpoint,
            crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
        );
        let mut outcome = self.recover(execution, plan)?;
        if let Some(work) = &mut outcome.checkpoint_restore_work {
            work.native_bytes_read = Some(native_bytes_read);
            work.native_envelopes_readmitted = Some(native_envelopes_readmitted);
        }
        self.runtime
            .restore_initial_schema_authority_after_recovery(initial_schema_authority);
        Ok(outcome)
    }

    pub fn recover(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        plan: RecoveryPlan,
    ) -> Result<RuntimeRecoveryOutcome, RelationalRecoveryStop> {
        crate::execution::check_request(execution).map_err(RelationalRecoveryStop::Request)?;
        let admitted = admit_recovery_or_emit(self.runtime, plan)?;
        let mut material = rebuild_admitted_recovery_or_emit(execution, self.runtime, admitted)?;
        crate::execution::check_request(execution).map_err(RelationalRecoveryStop::Request)?;
        declared_schema::require_declared_schema_of_recovered_main(&material.restored)?;
        move_durable_medium(self.runtime, &mut material.restored)?;
        let outcome = publish_recovered_runtime(self.runtime, material);
        Ok(outcome)
    }
}

struct RecoveredRuntimeMaterial {
    restored: RelationalRuntime,
    plan: RecoveryPlan,
    admission: admission::RecoveryAdmission,
    checkpoint_restore_work: Option<crate::durability::data::CheckpointRestoreWork>,
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
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    admitted: admission::AdmittedRecoveryPlan,
) -> Result<RecoveredRuntimeMaterial, RelationalRecoveryStop> {
    let admission = admitted.admission().clone();
    match rebuild_runtime_from_plan(execution, admitted) {
        Ok((restored, plan, checkpoint_restore_work)) => Ok(RecoveredRuntimeMaterial {
            restored,
            plan,
            admission,
            checkpoint_restore_work,
        }),
        Err(RelationalRecoveryStop::Durability(error)) => {
            admission.emit(runtime);
            Err(RelationalRecoveryStop::Durability(error))
        }
        Err(RelationalRecoveryStop::Request(cause)) => Err(RelationalRecoveryStop::Request(cause)),
        Err(RelationalRecoveryStop::Allocation(cause)) => {
            Err(RelationalRecoveryStop::Allocation(cause))
        }
    }
}

/// Move the recovering runtime's medium into the rebuilt runtime only after
/// replay, so rebuilding history never republishes it. The medium is owned by
/// exactly one runtime (D1): it leaves the recovering runtime as it enters
/// the rebuilt one, which starts with no unresolved durable fate.
fn move_durable_medium(
    runtime: &mut RelationalRuntime,
    restored: &mut RelationalRuntime,
) -> Result<(), DurabilityError> {
    if runtime.durability.medium().is_none() {
        return Ok(());
    }
    let unmovable = |detail: &str| {
        DurabilityError::new(
            crate::durability::data::RecoveryFailureClass::ReplayFailure,
            detail,
        )
    };
    let rebuilt = restored.unshared_state_mut().ok_or_else(|| {
        unmovable("a rebuilt runtime is shared before its durable medium is bound")
    })?;
    let medium = runtime
        .durability
        .detach_medium()
        .ok_or_else(|| unmovable("the recovering runtime's durable medium already moved"))?;
    rebuilt.durability.attach_medium(medium);
    Ok(())
}

fn publish_recovered_runtime(
    runtime: &mut RelationalRuntime,
    material: RecoveredRuntimeMaterial,
) -> RuntimeRecoveryOutcome {
    let RecoveredRuntimeMaterial {
        restored,
        plan,
        admission,
        checkpoint_restore_work,
    } = material;
    let tail_commits = plan.tail_commit_count();
    let checkpoint_commits = restored
        .durability
        .latest_checkpoint()
        .as_ref()
        .map(|checkpoint| {
            checkpoint
                .directory
                .as_ref()
                .map_or(checkpoint.envelopes.len(), |directory| {
                    directory.manifest.commits.len()
                })
        })
        .unwrap_or(0);
    let checkpoint_covered_position = plan.checkpoint_covered_position();
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
            recovery_checkpoint_selected(checkpoint_covered_position),
            recovery_range_replayed(tail_commits),
        ],
    );
    let outcome = RuntimeRecoveryOutcome {
        recovered_commits: restored.history.catalog_len(),
        latest_commit: restored.history().latest_commit(),
        restored_branches: restored.history.branch_count(),
        coverage: RecoveryCoverage {
            checkpoint_covered_position,
            checkpoint_commits,
            replayed_tail_commits: tail_commits,
            recovered_through_commit: restored.history().latest_commit(),
        },
        checkpoint_restore_work,
    };
    *runtime = restored;
    outcome
}

impl RelationalRuntime {
    pub(crate) fn rebuild_runtime_from_plan(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        plan: RecoveryPlan,
    ) -> Result<RelationalRuntime, RelationalRecoveryStop> {
        let admitted =
            admission::admit_recovery(self, plan).map_err(|rejection| rejection.into_error())?;
        rebuild_runtime_from_plan(execution, admitted).map(|(restored, _, _)| restored)
    }
}
