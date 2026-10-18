use crate::durability::data::RecoveryVerificationMode as RecoveryMode;
use std::collections::BTreeMap;

use worth_harness::facade::{
    ArtifactBundle, ArtifactClass, ArtifactSurface, WorkflowCaptureRequest,
};

use crate::facade::replay::RelationalReplayOutcome;

use super::super::complexity::workflow_budgets;
use super::read_summaries::read_summary_artifacts;
use super::session::CertifiedRelationalFintechSession;
use super::workflow_artifact_projection::{
    bool_field, dynamic_workflow_artifact_object, optional_string_field, optional_u64_field,
    optional_usize_field, string_array_field, string_field, usize_field,
    workflow_artifact_bool_field, workflow_artifact_field, workflow_artifact_object,
    WorkflowArtifactProjection,
};

fn replay_summary(replay: &RelationalReplayOutcome) -> WorkflowArtifactProjection {
    workflow_artifact_object([
        optional_u64_field(
            "commit_id",
            replay.commit.as_ref().map(|commit| commit.commit_id.0),
        ),
        usize_field(
            "commit_closure_len",
            replay.reconstructed_commit_closure.len(),
        ),
        string_array_field(
            "compared_surfaces",
            replay
                .compared_surfaces
                .iter()
                .map(|surface| format!("{surface:?}")),
        ),
        usize_field("mismatch_count", replay.mismatches.len()),
        optional_string_field(
            "failure",
            replay
                .failure
                .as_ref()
                .map(|failure| format!("{failure:?}")),
        ),
        optional_string_field(
            "lineage_authority_basis_kind",
            replay
                .lineage_authority_basis
                .as_ref()
                .map(|basis| format!("{:?}", basis.kind())),
        ),
        optional_string_field(
            "lineage_authority_digest_mode",
            replay
                .lineage_authority_basis
                .as_ref()
                .map(|basis| format!("{:?}", basis.digest_mode())),
        ),
        optional_u64_field(
            "lineage_authority_event_count",
            replay
                .lineage_authority_basis
                .as_ref()
                .map(|basis| basis.lineage_event_count() as u64),
        ),
        optional_u64_field(
            "lineage_authority_decision_count",
            replay
                .lineage_authority_basis
                .as_ref()
                .map(|basis| basis.lineage_decision_count() as u64),
        ),
    ])
}

fn branch_summary(session: &CertifiedRelationalFintechSession) -> WorkflowArtifactProjection {
    let latest_commit = session
        .world
        .runtime
        .history()
        .latest_commit()
        .map(|commit| commit.commit_id.0);
    let branches = session
        .named_branches
        .iter()
        .map(|(alias, branch)| {
            (
                alias.clone(),
                workflow_artifact_object([
                    string_field("branch_id", branch.0.clone()),
                    optional_u64_field(
                        "head_commit",
                        session
                            .world
                            .runtime
                            .history()
                            .branch_head(branch)
                            .map(|head| head.commit_id.0),
                    ),
                ]),
            )
        })
        .collect::<Vec<_>>();
    workflow_artifact_object([
        optional_u64_field("latest_commit", latest_commit),
        workflow_artifact_field("branches", dynamic_workflow_artifact_object(branches)),
    ])
}

fn diagnostics_summary(session: &CertifiedRelationalFintechSession) -> WorkflowArtifactProjection {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let recovery = session
        .world
        .runtime
        .durability()
        .recovery_plan(execution, RecoveryMode::NormalRecoveryVerification)
        .unwrap();
    let publication_diagnostics = session
        .world
        .runtime
        .publication()
        .diagnostic_access()
        .snapshot();
    let observation = &publication_diagnostics.observation;
    workflow_artifact_object([
        bool_field("latest_patch_present", observation.latest_patch_present),
        bool_field("latest_replay_present", observation.latest_replay_present),
        usize_field(
            "diagnostics_artifact_count",
            observation.diagnostics_artifact_count,
        ),
        usize_field(
            "checkpoint_count",
            recovery
                .store
                .as_ref()
                .map(|store| store.checkpoints.len())
                .unwrap_or(session.checkpoints.len()),
        ),
        usize_field("tail_log_len", recovery.tail_log.len()),
        optional_u64_field(
            "selected_checkpoint",
            recovery.cursor.checkpoint_id.as_ref().map(|id| id.0),
        ),
    ])
}

fn patch_summary(session: &CertifiedRelationalFintechSession) -> WorkflowArtifactProjection {
    let publication = session.world.runtime.publication();
    let artifacts = publication.artifact_snapshot();
    let observation = &artifacts.observation;
    workflow_artifact_object([
        optional_u64_field(
            "latest_commit",
            observation.latest_commit_id.map(|commit_id| commit_id.0),
        ),
        optional_u64_field(
            "publication_snapshot",
            observation
                .publication_snapshot_id
                .map(|snapshot_id| snapshot_id.0),
        ),
        optional_string_field(
            "publication_status",
            observation
                .publication_status
                .as_ref()
                .map(|status| format!("{status:?}")),
        ),
        optional_u64_field(
            "patch_position",
            observation.latest_patch_position.map(|position| position.0),
        ),
        optional_usize_field("patch_record_count", observation.latest_patch_record_count),
        optional_u64_field(
            "replay_commit",
            observation
                .latest_replay_commit_id
                .map(|commit_id| commit_id.0),
        ),
        bool_field(
            "snapshot_patch_matches_latest_patch",
            artifacts
                .latest_patch
                .as_ref()
                .zip(publication.latest_patch())
                .map(|(snapshot_patch, patch)| snapshot_patch == &patch)
                .unwrap_or(false),
        ),
        bool_field(
            "snapshot_replay_matches_latest_replay",
            artifacts
                .latest_replay
                .as_ref()
                .zip(publication.latest_replay())
                .map(|(snapshot_replay, replay)| snapshot_replay == &replay)
                .unwrap_or(false),
        ),
    ])
}

fn complexity_summary(session: &CertifiedRelationalFintechSession) -> WorkflowArtifactProjection {
    let counters = session.world.runtime.performance_access().counters();
    workflow_artifact_object([
        usize_field("full_state_clones", counters.full_state_clones),
        usize_field(
            "snapshot_pin_full_rebuilds",
            counters.snapshot_pin_full_rebuilds,
        ),
        usize_field(
            "partitions_touched_by_commit",
            counters.partitions_touched_by_commit,
        ),
        usize_field(
            "entity_slots_touched_by_commit",
            counters.entity_slots_touched_by_commit,
        ),
        usize_field(
            "visibility_entity_slot_scans",
            counters.visibility_entity_slot_scans,
        ),
        usize_field(
            "visibility_relation_slot_scans",
            counters.visibility_relation_slot_scans,
        ),
    ])
}

fn budget_summary(session: &CertifiedRelationalFintechSession) -> WorkflowArtifactProjection {
    let counters = session.world.runtime.performance_access().counters();
    let mut all_passed = true;
    let checks = workflow_budgets()
        .into_iter()
        .map(|budget| {
            let actual = (budget.selector)(&counters);
            let passed = actual <= budget.max;
            all_passed &= passed;
            workflow_artifact_object([
                string_field("label", budget.label.to_string()),
                usize_field("max", budget.max),
                usize_field("actual", actual),
                bool_field("passed", passed),
            ])
        })
        .collect::<Vec<_>>();
    workflow_artifact_object([
        bool_field("all_passed", all_passed),
        workflow_artifact_field("checks", WorkflowArtifactProjection::Array(checks)),
    ])
}

pub(super) fn capture_artifacts(
    session: &CertifiedRelationalFintechSession,
    request: &WorkflowCaptureRequest,
) -> Vec<ArtifactBundle> {
    let mut artifacts = Vec::new();
    for surface in &request.requested_surfaces {
        match surface {
            ArtifactSurface::SnapshotVisibleTruth => artifacts.push(captured_artifact_bundle(
                ArtifactClass::Truth,
                ArtifactSurface::SnapshotVisibleTruth,
                "snapshot-visible-truth",
                request,
                read_summary_artifacts(&session.named_reads),
            )),
            ArtifactSurface::BranchHeadState => artifacts.push(captured_artifact_bundle(
                ArtifactClass::Truth,
                ArtifactSurface::BranchHeadState,
                "branch-head-state",
                request,
                branch_summary(session),
            )),
            ArtifactSurface::ReplayRecoveryTruthState => artifacts.push(captured_artifact_bundle(
                ArtifactClass::Truth,
                ArtifactSurface::ReplayRecoveryTruthState,
                "replay-recovery-truth",
                request,
                dynamic_workflow_artifact_object(
                    session
                        .named_replays
                        .iter()
                        .map(|(alias, replay)| (alias.clone(), replay_summary(replay)))
                        .collect::<Vec<_>>(),
                ),
            )),
            ArtifactSurface::Diagnostics => artifacts.push(captured_artifact_bundle(
                ArtifactClass::Observability,
                ArtifactSurface::Diagnostics,
                "diagnostics",
                request,
                diagnostics_summary(session),
            )),
            ArtifactSurface::PatchChangeSurface => artifacts.push(captured_artifact_bundle(
                ArtifactClass::Observability,
                ArtifactSurface::PatchChangeSurface,
                "patch-change-surface",
                request,
                patch_summary(session),
            )),
            ArtifactSurface::StepTrace => artifacts.push(captured_artifact_bundle(
                ArtifactClass::Forensic,
                ArtifactSurface::StepTrace,
                "step-trace",
                request,
                WorkflowArtifactProjection::Array(
                    session
                        .executed_steps
                        .iter()
                        .cloned()
                        .map(WorkflowArtifactProjection::String)
                        .collect(),
                ),
            )),
            ArtifactSurface::CheckpointTrace => artifacts.push(captured_artifact_bundle(
                ArtifactClass::Forensic,
                ArtifactSurface::CheckpointTrace,
                "checkpoint-trace",
                request,
                workflow_artifact_object([
                    string_array_field("checkpoints", session.checkpoints.iter().cloned()),
                    string_array_field("snapshots", session.named_snapshots.keys().cloned()),
                ]),
            )),
            ArtifactSurface::ComplexityCounters => artifacts.push(captured_artifact_bundle(
                ArtifactClass::Performance,
                ArtifactSurface::ComplexityCounters,
                "complexity-counters",
                request,
                complexity_summary(session),
            )),
            ArtifactSurface::BudgetOutcome => artifacts.push(captured_artifact_bundle(
                ArtifactClass::Performance,
                ArtifactSurface::BudgetOutcome,
                "budget-outcome",
                request,
                budget_summary(session),
            )),
            _ => {}
        }
    }
    artifacts
}

fn captured_artifact_bundle(
    artifact_class: ArtifactClass,
    surface: ArtifactSurface,
    name: &'static str,
    request: &WorkflowCaptureRequest,
    projection: WorkflowArtifactProjection,
) -> ArtifactBundle {
    let metadata = captured_artifact_metadata(surface.clone(), &projection);
    ArtifactBundle {
        artifact_class,
        surface,
        name: name.to_string(),
        boundary: request.boundary,
        payload: projection.into_record_summary_value(),
        attachments: Vec::new(),
        metadata,
    }
}

fn captured_artifact_metadata(
    surface: ArtifactSurface,
    projection: &WorkflowArtifactProjection,
) -> BTreeMap<String, String> {
    match surface {
        ArtifactSurface::BudgetOutcome => workflow_artifact_bool_field(projection, "all_passed")
            .map(|passed| BTreeMap::from([("budget_all_passed".to_string(), passed.to_string())]))
            .unwrap_or_default(),
        _ => BTreeMap::new(),
    }
}
