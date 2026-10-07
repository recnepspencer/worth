use worth_harness::facade::{
    FailureInjectionPoint, InvariantCheck, RegressionTarget, RegressionTargetKind, WorkflowPlan,
    WorkflowRuntimeProfile, WorkflowState, WorkflowStep,
};

use super::super::certification_naming::{artifact_aliases, invariant_names, workflow_names};
use super::super::regimes::MarketRegime;
use super::workflow_session::FintechWorkflowStep;

pub(super) fn certified_step(
    name: impl Into<String>,
    operation: FintechWorkflowStep,
) -> WorkflowStep<FintechWorkflowStep> {
    WorkflowStep::new(name, operation)
        .capture_at(WorkflowState::Inspected)
        .inspect_at(WorkflowState::Inspected)
}

pub(super) fn checkpoint_step(
    name: impl Into<String>,
    operation: FintechWorkflowStep,
) -> WorkflowStep<FintechWorkflowStep> {
    certified_step(name, operation)
        .checkpoint_after()
        .capture_at(WorkflowState::Checkpointed)
}

pub(super) fn hostile_branch_replay_and_audit_plan() -> WorkflowPlan<FintechWorkflowStep> {
    WorkflowPlan::new(
        workflow_names::HOSTILE_BRANCH_REPLAY_AUDIT,
        "intraday-pricing-and-risk",
        "worth-signal",
        "fintech",
    )
    .with_seed(7)
    .with_regression_target(RegressionTarget {
        kind: RegressionTargetKind::ExpectedFailure,
        issue_key: "signal-workflow-certification-bootstrap".to_string(),
        summary: "Bootstrap hostile fintech certification through the new workflow runner"
            .to_string(),
        reproduction_hint: None,
    })
    .step(certified_step(
        "seed-main-regime",
        FintechWorkflowStep::SeedRegime {
            regime: MarketRegime::Calm,
            seed: 7,
        },
    ))
    .step(certified_step(
        "read-main-audit-surface",
        FintechWorkflowStep::ReadPrimaryAuditSurface {
            alias: artifact_aliases::BASELINE_AUDIT,
        },
    ))
    .step(checkpoint_step(
        "capture-main-snapshot",
        FintechWorkflowStep::CaptureActiveSnapshot {
            alias: artifact_aliases::MAIN_SNAPSHOT,
        },
    ))
    .step(certified_step(
        "open-analysis-branch",
        FintechWorkflowStep::OpenBranch {
            branch_name: "analysis-risk",
            alias: artifact_aliases::ANALYSIS_BRANCH,
        },
    ))
    .step(certified_step(
        "seed-analysis-regime",
        FintechWorkflowStep::SeedRegime {
            regime: MarketRegime::HighVol,
            seed: 17,
        },
    ))
    .step(certified_step(
        "read-analysis-audit-surface",
        FintechWorkflowStep::ReadPrimaryAuditSurface {
            alias: artifact_aliases::ANALYSIS_AUDIT,
        },
    ))
    .step(checkpoint_step(
        "capture-analysis-snapshot",
        FintechWorkflowStep::CaptureActiveSnapshot {
            alias: artifact_aliases::ANALYSIS_SNAPSHOT,
        },
    ))
    .step(certified_step(
        "capture-analysis-replay-before",
        FintechWorkflowStep::CaptureReplay {
            branch_alias: artifact_aliases::ANALYSIS_BRANCH,
            alias: artifact_aliases::ANALYSIS_REPLAY_BEFORE,
        },
    ))
    .step(
        certified_step(
            "inject-analysis-rollback",
            FintechWorkflowStep::InjectSyntheticRollback,
        )
        .capture_at(WorkflowState::Failed)
        .with_failure_injection(FailureInjectionPoint {
            boundary: WorkflowState::StepApplied,
            location: "analysis synthetic rollback".to_string(),
            detail: Some("branch-local failure injection during hostile correction".to_string()),
        }),
    )
    .step(certified_step(
        "capture-analysis-replay-after",
        FintechWorkflowStep::CaptureReplay {
            branch_alias: artifact_aliases::ANALYSIS_BRANCH,
            alias: artifact_aliases::ANALYSIS_REPLAY_AFTER,
        },
    ))
    .step(certified_step(
        "restore-analysis-snapshot",
        FintechWorkflowStep::RestoreSnapshot {
            branch_alias: artifact_aliases::ANALYSIS_BRANCH,
            snapshot_alias: artifact_aliases::ANALYSIS_SNAPSHOT,
        },
    ))
    .step(certified_step(
        "read-restored-analysis-audit-surface",
        FintechWorkflowStep::ReadPrimaryAuditSurface {
            alias: artifact_aliases::RESTORED_ANALYSIS_AUDIT,
        },
    ))
    .step(certified_step(
        "open-correction-branch",
        FintechWorkflowStep::OpenBranch {
            branch_name: "correction",
            alias: artifact_aliases::CORRECTION_BRANCH,
        },
    ))
    .step(certified_step(
        "seed-correction-regime",
        FintechWorkflowStep::SeedRegime {
            regime: MarketRegime::FxDislocation,
            seed: 29,
        },
    ))
    .step(certified_step(
        "read-correction-audit-surface",
        FintechWorkflowStep::ReadPrimaryAuditSurface {
            alias: "correction_audit",
        },
    ))
    .step(certified_step(
        "switch-main",
        FintechWorkflowStep::SwitchBranch {
            alias: artifact_aliases::MAIN_BRANCH,
        },
    ))
    .step(certified_step(
        "restore-main-snapshot",
        FintechWorkflowStep::RestoreSnapshot {
            branch_alias: artifact_aliases::MAIN_BRANCH,
            snapshot_alias: artifact_aliases::MAIN_SNAPSHOT,
        },
    ))
    .step(certified_step(
        "read-restored-main-audit-surface",
        FintechWorkflowStep::ReadPrimaryAuditSurface {
            alias: artifact_aliases::RESTORED_MAIN_AUDIT,
        },
    ))
    .step(certified_step(
        "capture-main-replay",
        FintechWorkflowStep::CaptureReplay {
            branch_alias: artifact_aliases::MAIN_BRANCH,
            alias: artifact_aliases::MAIN_REPLAY,
        },
    ))
    .step(certified_step(
        "capture-correction-replay",
        FintechWorkflowStep::CaptureReplay {
            branch_alias: artifact_aliases::CORRECTION_BRANCH,
            alias: artifact_aliases::CORRECTION_REPLAY,
        },
    ))
    .step(certified_step(
        "capture-analysis-replay-around-snapshot",
        FintechWorkflowStep::CaptureReplayAroundSnapshot {
            snapshot_alias: artifact_aliases::ANALYSIS_SNAPSHOT,
            alias: artifact_aliases::ANALYSIS_AROUND_SNAPSHOT,
        },
    ))
    .step(certified_step(
        "switch-correction",
        FintechWorkflowStep::SwitchBranch {
            alias: artifact_aliases::CORRECTION_BRANCH,
        },
    ))
    .step(certified_step(
        "capture-correction-lineage",
        FintechWorkflowStep::CaptureMainRiskLineage {
            alias: artifact_aliases::CORRECTION_LINEAGE,
        },
    ))
    .invariant(InvariantCheck::new(
        invariant_names::ANALYSIS_RESTORE_MATCHES,
        "analysis snapshot restore should preserve branch-local desk truth",
        WorkflowState::Completed,
    ))
    .invariant(InvariantCheck::new(
        invariant_names::MAIN_RESTORE_MATCHES,
        "main snapshot restore should preserve baseline desk truth",
        WorkflowState::Completed,
    ))
    .invariant(InvariantCheck::new(
        invariant_names::ANALYSIS_REPLAY_HAS_ROLLBACK,
        "analysis replay should retain rollback evidence",
        WorkflowState::Completed,
    ))
    .invariant(InvariantCheck::new(
        invariant_names::MAIN_REPLAY_BRANCH_LOCAL,
        "main replay should stay branch-local",
        WorkflowState::Completed,
    ))
    .invariant(InvariantCheck::new(
        invariant_names::CORRECTION_REPLAY_HAS_BRANCH_SWITCH,
        "correction replay should preserve branch activation",
        WorkflowState::Completed,
    ))
    .invariant(InvariantCheck::new(
        invariant_names::CORRECTION_LINEAGE_HAS_RECOVERY,
        "correction lineage should preserve risk evolution events",
        WorkflowState::Completed,
    ))
    .invariant(InvariantCheck::new(
        invariant_names::MAIN_BRANCH_HEAD_MATCHES,
        "main branch should retain its head snapshot metadata",
        WorkflowState::Completed,
    ))
    .invariant(InvariantCheck::new(
        invariant_names::ANALYSIS_REPLAY_MENTIONS_SNAPSHOT,
        "replay around snapshot should reference the saved analysis snapshot",
        WorkflowState::Completed,
    ))
}

pub(super) fn development_serial_profile() -> WorkflowRuntimeProfile {
    WorkflowRuntimeProfile {
        runtime_profile: "fintech-development".to_string(),
        policy_name: Some("fintech".to_string()),
        executor_name: Some("serial".to_string()),
        capability_profile: Some("serial-vs-parallel-hostile".to_string()),
    }
}

pub(super) fn development_parallel_profile(workers: usize) -> WorkflowRuntimeProfile {
    WorkflowRuntimeProfile {
        runtime_profile: "fintech-development".to_string(),
        policy_name: Some("fintech".to_string()),
        executor_name: Some(format!("lease-{workers}")),
        capability_profile: Some("serial-vs-parallel-hostile".to_string()),
    }
}
