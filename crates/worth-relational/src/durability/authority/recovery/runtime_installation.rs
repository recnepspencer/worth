//! Admit runtime-bound recovery, rebuild it once, and install the original receiver.
use super::admission;
use super::declared_schema;
use crate::diagnostics::data::{DiagnosticsArtifactKind, DiagnosticsScope};
use crate::durability::authority::recovered_runtime::DurabilityRecoveryAuthority;
use crate::durability::authority::recovered_runtime::RecoveredRelationalRuntimeAuthority;
use crate::durability::data::RelationalRecoveryStop;
use crate::durability::data::{DurabilityError, RecoveryCoverage, RecoveryPlan};
use crate::{
    durability::data::RecoveryOutcome as RuntimeRecoveryOutcome, runtime::RelationalRuntime,
};
use worth_relational_durability::internal_port::durability::data::checkpoint_images::DurableCheckpointOwnerPort as _;
use worth_relational_durability::internal_port::durability::data::recovery_plan::RecoveryPlanOwnerPort as _;

use super::super::diagnostics::{recovery_checkpoint_selected, recovery_range_replayed};
use super::super::runtime_rebuild::rebuild_runtime_from_plan;

impl<'runtime> DurabilityRecoveryAuthority<'runtime> {
    pub fn restore_native_checkpoint_with_authority(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        checkpoint: &crate::durability::data::RelationalNativeCheckpoint,
        source: std::sync::Arc<dyn crate::durability::medium::RelationalDurableColdSource>,
    ) -> Result<(RuntimeRecoveryOutcome, RecoveredRelationalRuntimeAuthority), RelationalRecoveryStop>
    {
        crate::execution::check_request(execution).map_err(RelationalRecoveryStop::Request)?;
        let native_bytes_read = checkpoint.bytes().len();
        let checkpoint =
            crate::durability::log::native_checkpoint_codec::decode_checkpoint_from_source(
                execution,
                checkpoint.bytes(),
                source,
            )?
            .checkpoint;
        self.restore_decoded_native_checkpoint_with_authority(
            execution,
            checkpoint,
            native_bytes_read,
        )
    }

    fn restored_runtime_authority(
        &self,
        recovered_roots: super::super::runtime_rebuild::root_inventory::RecoveredRootInventory,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<RecoveredRelationalRuntimeAuthority, RelationalRecoveryStop> {
        let recovered_roots = recovered_roots.into_readmission(execution)?;
        self.runtime
            .durability
            .install_source_root_inventory(recovered_roots.clone());
        let authority = recovered_authority(self.runtime, recovered_roots);
        self.runtime
            .durability
            .install_home_readmission_phase(std::sync::Arc::clone(
                &authority.home_readmission_phase,
            ));
        Ok(authority)
    }

    pub fn restore_native_checkpoint(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        checkpoint: &crate::durability::data::RelationalNativeCheckpoint,
        source: std::sync::Arc<dyn crate::durability::medium::RelationalDurableColdSource>,
    ) -> Result<RuntimeRecoveryOutcome, RelationalRecoveryStop> {
        self.restore_native_checkpoint_with_authority(execution, checkpoint, source)
            .map(|(outcome, _authority)| outcome)
    }

    pub(in crate::durability::authority) fn restore_decoded_native_checkpoint_with_authority(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        checkpoint: crate::durability::data::DurableCheckpoint,
        native_bytes_read: usize,
    ) -> Result<(RuntimeRecoveryOutcome, RecoveredRelationalRuntimeAuthority), RelationalRecoveryStop>
    {
        let native_envelopes_readmitted = checkpoint.envelopes().len();
        let plan = crate::runtime::durability_access::native_checkpoint_recovery_plan(
            self.runtime,
            checkpoint,
            crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
        );
        let (mut outcome, recovered) = self.recover_with_authority(execution, plan)?;
        if let Some(work) = &mut outcome.checkpoint_restore_work {
            work.native_bytes_read = Some(native_bytes_read);
            work.native_envelopes_readmitted = Some(native_envelopes_readmitted);
        }
        Ok((outcome, recovered))
    }

    pub fn recover(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        plan: RecoveryPlan,
    ) -> Result<RuntimeRecoveryOutcome, RelationalRecoveryStop> {
        self.recover_with_authority(execution, plan)
            .map(|(outcome, _authority)| outcome)
    }

    /// Reconstruct once and retain the source-produced exact images with their
    /// real root owners. This grants neither currentness nor World adoption.
    pub fn recover_with_authority(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        plan: RecoveryPlan,
    ) -> Result<(RuntimeRecoveryOutcome, RecoveredRelationalRuntimeAuthority), RelationalRecoveryStop>
    {
        crate::execution::check_request(execution).map_err(RelationalRecoveryStop::Request)?;
        let admitted = admit_recovery_or_emit(self.runtime, plan)?;
        self.recover_admitted_with_authority(execution, admitted)
    }

    pub(super) fn recover_admitted_with_authority(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        admitted: admission::AdmittedRecoveryPlan,
    ) -> Result<(RuntimeRecoveryOutcome, RecoveredRelationalRuntimeAuthority), RelationalRecoveryStop>
    {
        let initial_schema_authority = self.runtime.initial_schema_authority_snapshot();
        let mut material = rebuild_admitted_recovery_or_emit(execution, self.runtime, admitted)?;
        crate::execution::check_request(execution).map_err(RelationalRecoveryStop::Request)?;
        // Source replay reconstructs durable data, not the installed executors.
        // Preserve this owner's actual sealed inventory before runtime cutover.
        material
            .restored
            .restore_initial_schema_authority_after_recovery(initial_schema_authority);
        require_recovered_main_schema(&material.restored)?;
        move_durable_medium(self.runtime, &mut material.restored)?;
        let (outcome, recovered_roots) = publish_recovered_runtime(self.runtime, material);
        Ok((
            outcome,
            self.restored_runtime_authority(recovered_roots, execution)?,
        ))
    }
}

pub(super) fn recovered_authority(
    runtime: &RelationalRuntime,
    recovered_roots: std::sync::Arc<
        super::super::runtime_rebuild::root_inventory::RecoveredRootInventory,
    >,
) -> RecoveredRelationalRuntimeAuthority {
    let recovered_branch_images = runtime
        .history()
        .branch_cells_snapshot()
        .into_iter()
        .map(|cell| (cell.branch_id, (cell.observation, cell.truth_version)))
        .collect();
    let home_readmission_phase = std::sync::Arc::new(crate::durability::authority::HomeReadmissionMetadataPhase::new(
        runtime.runtime_instance_id(),
        runtime.history().latest_commit().map(|receipt| receipt.commit_id),
        runtime.schema_contract_runtime.home_readmission_metadata_kinds.clone(),
    ));
    RecoveredRelationalRuntimeAuthority {
        home_readmission_phase,
        home_catalog_readmission: std::sync::Mutex::new(()),
        home_program_predecessor: None,
        runtime_instance_id: runtime.runtime_instance_id(),
        recovered_branch_images,
        recovered_roots,
        codec_transition: None,
        performed_codec_retirement: None,
    }
}

pub(super) struct RecoveredRuntimeMaterial {
    pub(super) restored: RelationalRuntime,
    pub(super) plan: RecoveryPlan,
    pub(super) admission: admission::RecoveryAdmission,
    pub(super) checkpoint_restore_work: Option<crate::durability::data::CheckpointRestoreWork>,
    pub(super) recovered_roots:
        super::super::runtime_rebuild::root_inventory::RecoveredRootInventory,
}

pub(in crate::durability::authority) fn admit_recovery_or_emit(
    runtime: &RelationalRuntime,
    plan: RecoveryPlan,
) -> Result<admission::AdmittedRecoveryPlan, DurabilityError> {
    admit_selected_home_recovery(runtime, plan).map_err(|rejection| {
        if rejection.records_verification_counters() {
            record_recovery_verification_counters(runtime, rejection.authority_continuity());
        }
        emit_recovery_diagnostic(runtime, rejection.diagnostic());
        rejection.into_error()
    })
}

pub(super) fn rebuild_admitted_recovery_or_emit(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    admitted: admission::AdmittedRecoveryPlan,
) -> Result<RecoveredRuntimeMaterial, RelationalRecoveryStop> {
    let admission = admitted.admission().clone();
    match rebuild_runtime_from_plan(execution, admitted) {
        Ok((restored, plan, checkpoint_restore_work, recovered_roots)) => {
            Ok(RecoveredRuntimeMaterial {
                restored,
                plan,
                admission,
                checkpoint_restore_work,
                recovered_roots,
            })
        }
        Err(RelationalRecoveryStop::Durability(error)) => {
            emit_recovery_diagnostic(runtime, &admission);
            Err(RelationalRecoveryStop::Durability(error))
        }
        Err(RelationalRecoveryStop::Content(cause)) => Err(RelationalRecoveryStop::Content(cause)),
        Err(RelationalRecoveryStop::Request(cause)) => Err(RelationalRecoveryStop::Request(cause)),
        Err(RelationalRecoveryStop::AdmissionImage(cause)) => {
            Err(RelationalRecoveryStop::AdmissionImage(cause))
        }
        Err(RelationalRecoveryStop::Allocation(cause)) => {
            Err(RelationalRecoveryStop::Allocation(cause))
        }
    }
}

/// Move the recovering runtime's medium into the rebuilt runtime only after
/// replay, so rebuilding history never republishes it. The medium is owned by
/// exactly one runtime (D1): it leaves the recovering runtime as it enters
/// the rebuilt one, which starts with no unresolved durable fate.
pub(super) fn move_durable_medium(
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
    rebuilt.durability.attach_recovered_medium(medium);
    Ok(())
}

pub(super) fn publish_recovered_runtime(
    runtime: &mut RelationalRuntime,
    material: RecoveredRuntimeMaterial,
) -> (
    RuntimeRecoveryOutcome,
    super::super::runtime_rebuild::root_inventory::RecoveredRootInventory,
) {
    let RecoveredRuntimeMaterial {
        restored,
        plan,
        admission,
        checkpoint_restore_work,
        recovered_roots,
    } = material;
    let tail_commits = plan.tail_commit_count();
    let checkpoint_commits = restored
        .durability
        .latest_checkpoint()
        .as_ref()
        .map(|checkpoint| {
            checkpoint
                .selected_directory()
                .map_or(checkpoint.envelopes().len(), |directory| {
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
    runtime.durability.close_home_readmission_phase();
    *runtime = restored;
    (outcome, recovered_roots)
}

impl RelationalRuntime {
    pub(crate) fn rebuild_runtime_from_plan(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        plan: RecoveryPlan,
    ) -> Result<RelationalRuntime, RelationalRecoveryStop> {
        let admitted =
            admit_selected_home_recovery(self, plan).map_err(|rejection| rejection.into_error())?;
        rebuild_runtime_from_plan(execution, admitted).map(|(restored, _, _, _)| restored)
    }
}

fn emit_recovery_diagnostic(
    runtime: &RelationalRuntime,
    diagnostic: &admission::RecoveryAdmission,
) {
    runtime.publication_authority().push_bounded_diagnostic(
        DiagnosticsScope::History,
        diagnostic.artifact_kind(),
        vec![diagnostic.entry().clone()],
    );
}

fn admit_selected_home_recovery(
    runtime: &RelationalRuntime,
    mut plan: RecoveryPlan,
) -> Result<admission::AdmittedRecoveryPlan, admission::RecoveryRejection> {
    plan.select_home_world(
        runtime
            .durability
            .medium()
            .and_then(|medium| medium.home_world_selection()),
    );
    admission::admit_recovery(&runtime.config, plan)
}

fn record_recovery_verification_counters(
    runtime: &RelationalRuntime,
    authority_continuity: &crate::durability::data::RecoveryAuthorityContinuityCheck,
) {
    let layer = match authority_continuity.verification_outcome {
        crate::durability::data::RecoveryVerificationOutcome::VerifiedAtLayer(layer) => layer,
        crate::durability::data::RecoveryVerificationOutcome::Rejected { layer, .. } => layer,
    };
    runtime
        .performance_access()
        .count_replay_verification_layer(layer);
    if matches!(
        authority_continuity.first_mismatch,
        Some(crate::durability::data::RecoveryAuthorityContinuityMismatch::DescriptorSemanticsVersion { .. })
            | Some(crate::durability::data::RecoveryAuthorityContinuityMismatch::DescriptorCanonicalBasisVersion { .. })
    ) {
        runtime
            .performance_access()
            .count_descriptor_version_mismatch();
    }
}

pub(super) fn require_recovered_main_schema(
    rebuilt: &RelationalRuntime,
) -> Result<(), DurabilityError> {
    let main = rebuilt.main_branch_identity();
    let root = rebuilt
        .history
        .branch_cell(main.branch_id())
        .and_then(|cell| cell.root())
        .ok_or_else(|| {
            DurabilityError::new(
                crate::durability::data::RecoveryFailureClass::ReplayFailure,
                "the recovered main branch has no head root to name its schema",
            )
        })?;
    declared_schema::require_declared_schema_of_recovered_main(
        root.schema_authority(),
        &rebuilt.config.schema.registry,
    )
}
