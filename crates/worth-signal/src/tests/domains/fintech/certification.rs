mod artifact_matrix;
mod independent_oracle;
pub(super) mod invalidation;
mod workflow_adapter;
mod workflow_scenario;
mod workflow_session;

use worth_harness::facade::{
    ArtifactBundle, FailureInjectionPoint, InvariantCheck, InvariantReport, ReproductionMetadata,
    WorkflowCaptureRequest, WorkflowCertificationAdapter, WorkflowCertificationCapabilities,
    WorkflowCertificationRunner, WorkflowCheckpoint, WorkflowFailureContext, WorkflowPlan,
    WorkflowRuntimeProfile, WorkflowState, WorkflowStep, WorkflowStepOutcome,
};

use crate::facade::SignalError;

use self::workflow_scenario::development_parallel_profile;
use self::workflow_scenario::{development_serial_profile, hostile_branch_replay_and_audit_plan};
use self::workflow_session::{
    CertifiedFintechWorkflowSession, FintechWorkflowStep, SignalFintechWorkflowCertificationAdapter,
};

impl WorkflowCertificationAdapter for SignalFintechWorkflowCertificationAdapter {
    type Session = CertifiedFintechWorkflowSession;
    type Step = FintechWorkflowStep;
    type Error = SignalError;

    fn adapter_name(&self) -> &'static str {
        "worth-signal-fintech-workflow"
    }

    fn capabilities(&self) -> WorkflowCertificationCapabilities {
        artifact_matrix::capabilities()
    }

    fn initialize_session(
        &self,
        plan: &WorkflowPlan<Self::Step>,
        profile: &WorkflowRuntimeProfile,
    ) -> Result<Self::Session, Self::Error> {
        workflow_adapter::initialize_session(plan, profile)
    }

    fn execute_step(
        &self,
        session: &mut Self::Session,
        step: &WorkflowStep<Self::Step>,
        injection: Option<&FailureInjectionPoint>,
    ) -> Result<WorkflowStepOutcome, Self::Error> {
        workflow_adapter::execute_step(session, step, injection)
    }

    fn create_checkpoint(
        &self,
        session: &mut Self::Session,
        checkpoint: &WorkflowCheckpoint,
    ) -> Result<(), Self::Error> {
        workflow_adapter::create_checkpoint(session, checkpoint)
    }

    fn capture_artifacts(
        &self,
        session: &Self::Session,
        request: &WorkflowCaptureRequest,
    ) -> Result<Vec<ArtifactBundle>, Self::Error> {
        artifact_matrix::capture_artifacts(session, request)
    }

    fn run_invariants(
        &self,
        session: &Self::Session,
        boundary: WorkflowState,
        checks: &[InvariantCheck],
    ) -> Result<Vec<InvariantReport>, Self::Error> {
        workflow_adapter::run_invariants(session, boundary, checks)
    }

    fn serialize_reproduction(
        &self,
        session: &Self::Session,
        failure: &WorkflowFailureContext,
    ) -> Result<ReproductionMetadata, Self::Error> {
        workflow_adapter::serialize_reproduction(session, failure)
    }
}

#[test]
fn workflow_certification_runner_proves_hostile_fintech_branch_replay_and_audit() {
    let runner = WorkflowCertificationRunner::new(SignalFintechWorkflowCertificationAdapter);
    let plan = hostile_branch_replay_and_audit_plan();
    let report = runner
        .certify(&plan, &development_serial_profile())
        .unwrap();

    assert_eq!(report.session.state, WorkflowState::Completed);
    assert!(report.failure_bundle.is_none());
    assert!(report
        .session
        .session_data
        .named_replays
        .contains_key("analysis_replay_after"));
    assert!(report
        .session
        .session_data
        .named_lineages
        .contains_key("correction_lineage"));
}

#[test]
fn financial_aspect_causality_certification_seals_all_required_scenarios() {
    let run = invalidation::run_financial_causality_courtroom()
        .expect("all financial aspect-causality evidence should seal");
    assert_eq!(run.seed(), 41);
    assert_eq!(run.scenario_count(), 8);
    assert!(run.minimum_dependency_revision() > 0);
}

#[test]
fn financial_aspect_causality_certification_survives_leased_execution_composition() {
    let run = invalidation::run_financial_causality_courtroom()
        .expect("leased execution must preserve financial causality semantics");
    assert_eq!(run.scenario_count(), 8);
}

#[test]
fn workflow_certification_runner_keeps_serial_parallel_fintech_overlap_honest() {
    let runner = WorkflowCertificationRunner::new(SignalFintechWorkflowCertificationAdapter);
    let plan = hostile_branch_replay_and_audit_plan();
    let serial = runner
        .certify(&plan, &development_serial_profile())
        .unwrap();
    assert_eq!(serial.session.state, WorkflowState::Completed);
    let checked_serial = runner
        .certify(&plan, &development_parallel_profile(1))
        .unwrap();
    assert_eq!(checked_serial.session.state, WorkflowState::Completed);
    let serial_truth =
        independent_oracle::compare_signal_fintech_visible_overlap(&serial, &checked_serial);
    assert!(
        serial_truth.matched,
        "serial-vs-lease-1 fintech visible truth drift: {:?}",
        serial_truth.mismatches
    );
    for workers in [2, 4] {
        let leased = runner
            .certify(&plan, &development_parallel_profile(workers))
            .unwrap();
        assert_eq!(
            leased.session.state,
            WorkflowState::Completed,
            "lease workers {workers} stopped after {:?}",
            leased.session.step_trace
        );
        let visible = independent_oracle::compare_signal_fintech_visible_overlap(&serial, &leased);
        assert!(
            visible.matched,
            "serial-vs-lease-{workers} fintech visible truth drift: {:?}",
            visible.mismatches
        );
        // The checked runs share evaluation request boundaries, so their
        // complete replay and lineage identities must agree exactly.
        let outcome = independent_oracle::compare_signal_fintech_overlap(&checked_serial, &leased);
        assert!(
            outcome.matched,
            "lease-1-vs-lease-{workers} fintech overlap drift: {:?}",
            outcome.mismatches
        );
    }
}
