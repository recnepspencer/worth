use std::collections::BTreeMap;

use serde_json::json;
use worth_harness::facade::{
    FailureInjectionPoint, InvariantCheck, InvariantReport, ReproductionMetadata,
    WorkflowCheckpoint, WorkflowFailureContext, WorkflowPlan, WorkflowRuntimeProfile,
    WorkflowState, WorkflowStep, WorkflowStepOutcome,
};

use crate::facade::SignalError;
use crate::tests::domains::fintech::FintechScale;

use super::super::market_seed::MarketSeed;
use super::super::regimes::MarketRegime;
use super::super::world::{compile_unseeded_runtime_fixture, FinancialWorldDefinition};
use super::independent_oracle;
use super::workflow_session::{
    CertifiedFintechWorkflowSession, FintechWorkflowStep, SignalFintechWorkflowCertificationAdapter,
};

pub(super) fn initialize_session(
    _plan: &WorkflowPlan<FintechWorkflowStep>,
    profile: &WorkflowRuntimeProfile,
) -> Result<CertifiedFintechWorkflowSession, SignalError> {
    let policy = SignalFintechWorkflowCertificationAdapter::runtime_policy(profile)?;
    let workers = SignalFintechWorkflowCertificationAdapter::workers(profile)?;
    let mut world = compile_unseeded_runtime_fixture(FinancialWorldDefinition::runtime_fixture(
        FintechScale::smoke(),
        MarketRegime::Calm,
        7,
    ));
    world.runtime.set_runtime_policy(policy);
    let main = world.current_branch();
    Ok(CertifiedFintechWorkflowSession {
        world,
        workers,
        policy,
        named_branches: BTreeMap::from([("main".to_string(), main)]),
        named_snapshots: BTreeMap::new(),
        named_audits: BTreeMap::new(),
        named_replays: BTreeMap::new(),
        named_lineages: BTreeMap::new(),
        executed_steps: Vec::new(),
        checkpoints: Vec::new(),
        failure_injections: Vec::new(),
    })
}

pub(super) fn execute_step(
    session: &mut CertifiedFintechWorkflowSession,
    step: &WorkflowStep<FintechWorkflowStep>,
    injection: Option<&FailureInjectionPoint>,
) -> Result<WorkflowStepOutcome, SignalError> {
    session.executed_steps.push(step.name.clone());
    match &step.operation {
        FintechWorkflowStep::SeedRegime { regime, seed } => {
            session.world.seed_market(MarketSeed::new(*regime, *seed))?;
            Ok(WorkflowStepOutcome::applied())
        }
        FintechWorkflowStep::CaptureActiveSnapshot { alias } => {
            let snapshot = session.world.capture_world_snapshot();
            session
                .named_snapshots
                .insert((*alias).to_string(), snapshot);
            Ok(WorkflowStepOutcome {
                detail: Some(format!("captured snapshot `{alias}`")),
                request_checkpoint: true,
            })
        }
        FintechWorkflowStep::OpenBranch { branch_name, alias } => {
            let branch = session.world.open_branch(branch_name)?;
            session.named_branches.insert((*alias).to_string(), branch);
            Ok(WorkflowStepOutcome::applied())
        }
        FintechWorkflowStep::SwitchBranch { alias } => {
            let branch = session.branch(alias)?;
            session.world.runtime.switch_branch(branch)?;
            Ok(WorkflowStepOutcome::applied())
        }
        FintechWorkflowStep::ReadPrimaryAuditSurface { alias } => {
            let value = match session.workers {
                Some(workers) => session
                    .world
                    .read_primary_audit_surface_with_workers(workers)?,
                None => session.world.read_primary_audit_surface()?,
            };
            session.named_audits.insert((*alias).to_string(), value);
            Ok(WorkflowStepOutcome::applied())
        }
        FintechWorkflowStep::InjectSyntheticRollback => {
            session.world.inject_primary_market_rollback()?;
            if let Some(injection) = injection {
                session
                    .failure_injections
                    .push(format!("{:?}:{}", injection.boundary, injection.location));
            }
            Ok(WorkflowStepOutcome {
                detail: Some("synthetic rollback captured".to_string()),
                request_checkpoint: false,
            })
        }
        FintechWorkflowStep::RestoreSnapshot {
            branch_alias,
            snapshot_alias,
        } => {
            let branch = session.branch(branch_alias)?;
            let snapshot = session.snapshot(snapshot_alias)?;
            session
                .world
                .runtime
                .restore_branch_snapshot(branch, &snapshot)?;
            Ok(WorkflowStepOutcome::applied())
        }
        FintechWorkflowStep::CaptureReplay {
            branch_alias,
            alias,
        } => {
            let branch = session.branch(branch_alias)?;
            let replay = session.world.replay_for_branch(branch);
            session.named_replays.insert((*alias).to_string(), replay);
            Ok(WorkflowStepOutcome::applied())
        }
        FintechWorkflowStep::CaptureReplayAroundSnapshot {
            snapshot_alias,
            alias,
        } => {
            let snapshot = session.snapshot(snapshot_alias)?;
            let replay = session.world.replay_around_saved_snapshot(&snapshot);
            session.named_replays.insert((*alias).to_string(), replay);
            Ok(WorkflowStepOutcome::applied())
        }
        FintechWorkflowStep::CaptureMainRiskLineage { alias } => {
            let lineage = session.world.main_risk_lineage();
            session.named_lineages.insert((*alias).to_string(), lineage);
            Ok(WorkflowStepOutcome::applied())
        }
    }
}

pub(super) fn create_checkpoint(
    session: &mut CertifiedFintechWorkflowSession,
    checkpoint: &WorkflowCheckpoint,
) -> Result<(), SignalError> {
    session.checkpoints.push(checkpoint.checkpoint_name.clone());
    Ok(())
}

pub(super) fn run_invariants(
    session: &CertifiedFintechWorkflowSession,
    _boundary: WorkflowState,
    checks: &[InvariantCheck],
) -> Result<Vec<InvariantReport>, SignalError> {
    checks
        .iter()
        .map(|check| independent_oracle::check_invariant(session, check))
        .collect()
}

pub(super) fn serialize_reproduction(
    session: &CertifiedFintechWorkflowSession,
    failure: &WorkflowFailureContext,
) -> Result<ReproductionMetadata, SignalError> {
    Ok(ReproductionMetadata {
        format: "json".to_string(),
        payload: json!({
            "state": format!("{:?}", failure.state),
            "step_index": failure.step_index,
            "failure_injection": failure.failure_injection.as_ref().map(|injection| {
                json!({
                    "boundary": format!("{:?}", injection.boundary),
                    "location": injection.location,
                    "detail": injection.detail,
                })
            }),
            "current_branch": session.world.current_branch().name,
            "known_branches": session.named_branches.keys().collect::<Vec<_>>(),
            "known_snapshots": session.named_snapshots.keys().collect::<Vec<_>>(),
            "known_audits": session.named_audits.keys().collect::<Vec<_>>(),
            "policy": format!("{:?}", session.policy.tier),
        })
        .to_string(),
    })
}
