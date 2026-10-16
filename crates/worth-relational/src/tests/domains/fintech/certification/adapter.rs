use std::collections::{BTreeMap, BTreeSet};

use worth_harness::facade::{
    AdapterSupport, ArtifactBundle, ArtifactSurface, CheckpointSemantics,
    DifferentialMatrixCapability, FailureInjectionPoint, InvariantCheck, InvariantReport,
    ProfileConditionalGuarantee, ReproductionMetadata, UnsupportedWorkflowComparison,
    WorkflowArtifactSurfaceCapability, WorkflowCaptureRequest, WorkflowCertificationAdapter,
    WorkflowCertificationCapabilities, WorkflowCheckpoint, WorkflowFailureContext, WorkflowPlan,
    WorkflowRuntimeProfile, WorkflowState, WorkflowStep, WorkflowStepOutcome,
};

use crate::facade::history::BranchId;
use crate::facade::replay::{RelationalReplayRequest, ReplayExecutionMode, ReplayVerificationMode};

use super::super::actions::{
    correct_seeded_trade_candidate, open_analysis_branch, refresh_risk_views,
    repair_seeded_failed_settlement, shock_market_on_branch, stress_seeded_intraday_risk,
};
use super::super::fixture::FintechWorkflowCase;
use super::super::scenarios::{setup_world_for, FintechScenario};
use super::artifacts::capture_artifacts;
use super::invariants::run_checks;
use super::read_summaries::{case_read_summary, read_summary};
use super::session::CertifiedRelationalFintechSession;
use super::steps::{FintechCaseRef, FintechWorkflowStep};
use super::workflow_artifact_projection::{
    optional_usize_field, string_array_field, string_field, workflow_artifact_object,
    WorkflowArtifactProjection,
};

#[derive(Debug, Default, Clone, Copy)]
pub(super) struct RelationalFintechWorkflowCertificationAdapter;

impl RelationalFintechWorkflowCertificationAdapter {
    fn case_for(
        session: &CertifiedRelationalFintechSession,
        case: FintechCaseRef,
    ) -> FintechWorkflowCase {
        match case {
            FintechCaseRef::LateTradeCorrection => session.world.late_trade_correction_case(),
            FintechCaseRef::IntradayRisk => session.world.intraday_risk_case(),
            FintechCaseRef::FailedSettlementRepair => session.world.failed_settlement_repair_case(),
        }
    }
}

impl WorkflowCertificationAdapter for RelationalFintechWorkflowCertificationAdapter {
    type Session = CertifiedRelationalFintechSession;
    type Step = FintechWorkflowStep;
    type Error = String;

    fn adapter_name(&self) -> &'static str {
        "worth-relational-fintech-workflow"
    }

    fn capabilities(&self) -> WorkflowCertificationCapabilities {
        WorkflowCertificationCapabilities {
            artifact_surfaces: vec![
                WorkflowArtifactSurfaceCapability {
                    surface: ArtifactSurface::SnapshotVisibleTruth,
                    profiles: BTreeSet::from(["fintech-development".to_string()]),
                },
                WorkflowArtifactSurfaceCapability {
                    surface: ArtifactSurface::BranchHeadState,
                    profiles: BTreeSet::from(["fintech-development".to_string()]),
                },
                WorkflowArtifactSurfaceCapability {
                    surface: ArtifactSurface::ReplayRecoveryTruthState,
                    profiles: BTreeSet::from(["fintech-development".to_string()]),
                },
                WorkflowArtifactSurfaceCapability {
                    surface: ArtifactSurface::Diagnostics,
                    profiles: BTreeSet::from(["fintech-development".to_string()]),
                },
                WorkflowArtifactSurfaceCapability {
                    surface: ArtifactSurface::PatchChangeSurface,
                    profiles: BTreeSet::from(["fintech-development".to_string()]),
                },
                WorkflowArtifactSurfaceCapability {
                    surface: ArtifactSurface::StepTrace,
                    profiles: BTreeSet::from(["fintech-development".to_string()]),
                },
                WorkflowArtifactSurfaceCapability {
                    surface: ArtifactSurface::CheckpointTrace,
                    profiles: BTreeSet::from(["fintech-development".to_string()]),
                },
                WorkflowArtifactSurfaceCapability {
                    surface: ArtifactSurface::ComplexityCounters,
                    profiles: BTreeSet::from(["fintech-development".to_string()]),
                },
                WorkflowArtifactSurfaceCapability {
                    surface: ArtifactSurface::BudgetOutcome,
                    profiles: BTreeSet::from(["fintech-development".to_string()]),
                },
            ],
            checkpoint_semantics: BTreeSet::from([CheckpointSemantics::AdapterDefined]),
            replay_recovery_support: BTreeSet::from([
                ArtifactSurface::BranchHeadState,
                ArtifactSurface::ReplayRecoveryTruthState,
                ArtifactSurface::SnapshotVisibleTruth,
            ]),
            differential_matrices: vec![DifferentialMatrixCapability {
                matrix_name: "relational-fintech-baseline".to_string(),
                profiles: BTreeSet::from(["fintech-development".to_string()]),
                guaranteed_surfaces: BTreeSet::from([
                    ArtifactSurface::BranchHeadState,
                    ArtifactSurface::ReplayRecoveryTruthState,
                    ArtifactSurface::SnapshotVisibleTruth,
                ]),
            }],
            unsupported_comparisons: vec![UnsupportedWorkflowComparison {
                surface: ArtifactSurface::ComplexityCounters,
                reason: "relational fintech workflow certification exposes raw counters but does not yet compare them across profiles"
                    .to_string(),
            }],
            profile_guarantees: vec![ProfileConditionalGuarantee {
                profile_name: "fintech-development".to_string(),
                guarantee:
                    "baseline branch/snapshot/replay artifacts stay crate-local and workflow-driven"
                        .to_string(),
            }],
            budget_artifacts: AdapterSupport::Supported,
        }
    }

    fn initialize_session(
        &self,
        _plan: &WorkflowPlan<Self::Step>,
        _profile: &WorkflowRuntimeProfile,
    ) -> Result<Self::Session, Self::Error> {
        let host_request = crate::test_execution_policy::serial_request();
        let execution = &worth_execution::ExecutionRequest::serial(&host_request);
        Ok(CertifiedRelationalFintechSession {
            world: setup_world_for(execution, FintechScenario::SmokeBook),
            named_branches: BTreeMap::from([("main".to_string(), BranchId("main".to_string()))]),
            named_snapshots: BTreeMap::new(),
            named_reads: BTreeMap::new(),
            named_replays: BTreeMap::new(),
            executed_steps: Vec::new(),
            checkpoints: Vec::new(),
        })
    }

    fn execute_step(
        &self,

        session: &mut Self::Session,
        step: &WorkflowStep<Self::Step>,
        _injection: Option<&FailureInjectionPoint>,
    ) -> Result<WorkflowStepOutcome, Self::Error> {
        let host_request = crate::test_execution_policy::serial_request();
        let execution = &worth_execution::ExecutionRequest::serial(&host_request);
        session.executed_steps.push(step.name.clone());
        match &step.operation {
            FintechWorkflowStep::CaptureMainSnapshot { alias } => {
                let snapshot = session.world.runtime.visibility_authority().snapshot();
                session
                    .named_snapshots
                    .insert((*alias).to_string(), snapshot);
                Ok(WorkflowStepOutcome {
                    detail: Some(format!("captured main snapshot `{alias}`")),
                    request_checkpoint: true,
                })
            }
            FintechWorkflowStep::OpenAnalysisBranch { alias } => {
                let branch = open_analysis_branch(&mut session.world);
                session.named_branches.insert((*alias).to_string(), branch);
                Ok(WorkflowStepOutcome::applied())
            }
            FintechWorkflowStep::ShockMarket { branch_alias } => {
                let branch = session.branch(branch_alias)?;
                shock_market_on_branch(execution, &mut session.world, branch);
                Ok(WorkflowStepOutcome::applied())
            }
            FintechWorkflowStep::CorrectCaseTrade { branch_alias, case } => {
                let branch = session.branch(branch_alias)?;
                match case {
                    FintechCaseRef::LateTradeCorrection => {
                        correct_seeded_trade_candidate(execution, &mut session.world, branch);
                    }
                    other => {
                        return Err(format!(
                            "unsupported trade-correction case for certification: {other:?}"
                        ))
                    }
                }
                Ok(WorkflowStepOutcome::applied())
            }
            FintechWorkflowStep::StressCaseRisk { branch_alias, case } => {
                let branch = session.branch(branch_alias)?;
                match case {
                    FintechCaseRef::IntradayRisk => {
                        stress_seeded_intraday_risk(execution, &mut session.world, branch);
                    }
                    other => {
                        return Err(format!(
                            "unsupported risk-stress case for certification: {other:?}"
                        ))
                    }
                }
                Ok(WorkflowStepOutcome::applied())
            }
            FintechWorkflowStep::RepairCaseSettlement { branch_alias, case } => {
                let branch = session.branch(branch_alias)?;
                match case {
                    FintechCaseRef::FailedSettlementRepair => {
                        repair_seeded_failed_settlement(execution, &mut session.world, branch);
                    }
                    other => {
                        return Err(format!(
                            "unsupported settlement-repair case for certification: {other:?}"
                        ))
                    }
                }
                Ok(WorkflowStepOutcome::applied())
            }
            FintechWorkflowStep::RefreshRisk { branch_alias } => {
                let branch = session.branch(branch_alias)?;
                refresh_risk_views(execution, &mut session.world, branch);
                Ok(WorkflowStepOutcome::applied())
            }
            FintechWorkflowStep::ReadSnapshot {
                snapshot_alias,
                read_alias,
            } => {
                let snapshot = session.snapshot(snapshot_alias)?;
                let summary = read_summary(execution, session, snapshot)?;
                session
                    .named_reads
                    .insert((*read_alias).to_string(), summary);
                Ok(WorkflowStepOutcome::applied())
            }
            FintechWorkflowStep::ReadCaseProbe { case, read_alias } => {
                let workflow_case = Self::case_for(session, *case);
                session.named_reads.insert(
                    (*read_alias).to_string(),
                    case_read_summary(execution, session, workflow_case.role),
                );
                Ok(WorkflowStepOutcome::applied())
            }
            FintechWorkflowStep::CaptureReplay {
                branch_alias,
                alias,
            } => {
                let branch = session.branch(branch_alias)?;
                let commit_id = session
                    .world
                    .runtime
                    .history()
                    .latest_commit()
                    .ok_or_else(|| "latest commit unavailable for replay capture".to_string())?
                    .commit_id;
                let replay = session
                    .world
                    .runtime
                    .replay_authority()
                    .replay_commit(
                        RelationalReplayRequest {
                            commit_id,
                            branch_id: branch,
                            execution_mode: ReplayExecutionMode::SerialDeterministic,
                            verification_mode: ReplayVerificationMode::NormalRecoveryVerification,
                        },
                        execution,
                    )
                    .unwrap();
                session.named_replays.insert((*alias).to_string(), replay);
                Ok(WorkflowStepOutcome::applied())
            }
        }
    }

    fn create_checkpoint(
        &self,
        session: &mut Self::Session,
        checkpoint: &WorkflowCheckpoint,
    ) -> Result<(), Self::Error> {
        let host_request = crate::test_execution_policy::serial_request();
        let execution = &worth_execution::ExecutionRequest::serial(&host_request);
        session.checkpoints.push(checkpoint.checkpoint_name.clone());
        session
            .world
            .runtime
            .durability_authority()
            .checkpoint(execution)
            .map(|_| ())
            .map_err(|error| error.to_string())
    }

    fn capture_artifacts(
        &self,
        session: &Self::Session,
        request: &WorkflowCaptureRequest,
    ) -> Result<Vec<ArtifactBundle>, Self::Error> {
        Ok(capture_artifacts(session, request))
    }

    fn run_invariants(
        &self,
        session: &Self::Session,
        _boundary: WorkflowState,
        checks: &[InvariantCheck],
    ) -> Result<Vec<InvariantReport>, Self::Error> {
        run_checks(session, checks)
    }

    fn serialize_reproduction(
        &self,
        session: &Self::Session,
        failure: &WorkflowFailureContext,
    ) -> Result<ReproductionMetadata, Self::Error> {
        Ok(ReproductionMetadata {
            format: "json".to_string(),
            payload: reproduction_workflow_artifact_projection(session, failure)
                .into_record_summary_value()
                .to_string(),
        })
    }
}

fn reproduction_workflow_artifact_projection(
    session: &CertifiedRelationalFintechSession,
    failure: &WorkflowFailureContext,
) -> WorkflowArtifactProjection {
    workflow_artifact_object([
        string_field("state", format!("{:?}", failure.state)),
        optional_usize_field("step_index", failure.step_index),
        string_array_field("known_branches", session.named_branches.keys().cloned()),
        string_array_field("known_snapshots", session.named_snapshots.keys().cloned()),
        string_array_field("known_reads", session.named_reads.keys().cloned()),
        string_array_field("known_replays", session.named_replays.keys().cloned()),
    ])
}
