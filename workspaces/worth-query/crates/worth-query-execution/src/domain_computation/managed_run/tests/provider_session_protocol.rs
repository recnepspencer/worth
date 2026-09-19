use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use super::*;
use crate::domain_computation::{
    WorthQueryProviderExecutionPlanView, WorthQueryProviderSessionAffinityIdentity,
    WorthQueryProviderSessionDenialKind, WorthQueryProviderSessionFailure,
    WorthQueryProviderSessionLifecycle, WorthQueryProviderSessionProtocolCounters,
    WorthQueryProviderSessionProtocolStage, WorthQueryProviderSessionRecoveryPosture,
    WorthQueryProviderSessionToken, WorthQueryProviderSessionTokenAdmission,
    WorthQueryProviderSessionView, WorthQuerySessionCommitOrAbortOutcome,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum SessionFailurePoint {
    None,
    ReadmissionRejection,
    ReadmissionPanic,
    PreparationRejection,
    PreparationPanic,
    StagedPreparationRejection,
    StagedPreparationPanic,
    CommitRejection,
    CommitPanic,
    AbortRejection,
    AbortPanic,
}

#[derive(Default)]
pub(super) struct SessionCallCounts {
    readmissions: AtomicUsize,
    preparations: AtomicUsize,
    staged_preparations: AtomicUsize,
    commits: AtomicUsize,
    aborts: AtomicUsize,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(super) struct SessionCallObservation {
    pub(super) readmissions: usize,
    pub(super) preparations: usize,
    pub(super) staged_preparations: usize,
    pub(super) commits: usize,
    pub(super) aborts: usize,
}

impl SessionCallCounts {
    pub(super) fn observe(&self) -> SessionCallObservation {
        SessionCallObservation {
            readmissions: self.readmissions.load(Ordering::Acquire),
            preparations: self.preparations.load(Ordering::Acquire),
            staged_preparations: self.staged_preparations.load(Ordering::Acquire),
            commits: self.commits.load(Ordering::Acquire),
            aborts: self.aborts.load(Ordering::Acquire),
        }
    }
}

struct SessionProtocolProvider {
    failure: SessionFailurePoint,
    calls: Arc<SessionCallCounts>,
}

struct UnusedSessionExecution;

impl WorthQueryGraphProviderExecution for UnusedSessionExecution {
    fn advance(
        &mut self,
        _step: &mut WorthQueryGraphProviderStep,
    ) -> Result<WorthQueryGraphProviderStepDisposition, WorthQueryGraphProviderFailure> {
        unreachable!("session protocol tests do not enter the legacy graph callback")
    }

    fn dispose(&mut self) -> Result<(), WorthQueryGraphProviderFailure> {
        Ok(())
    }
}

impl WorthQueryGraphParticipationProvider<ManagedGraph> for SessionProtocolProvider {
    type Execution = UnusedSessionExecution;

    fn execution_resource_support(
        &self,
    ) -> worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupport {
        crate::domain_computation::provider_session::execution_resource_support(
            "session-protocol",
            8,
        )
    }

    fn begin(
        &self,
        _call: &WorthQueryGraphProviderCall,
        _start: &mut WorthQueryGraphProviderExecutionStart,
    ) -> Result<
        WorthQueryCooperativeGraphProviderExecution<Self::Execution>,
        WorthQueryGraphProviderFailure,
    > {
        unreachable!("sealed session protocol must not route through the one-shot callback")
    }
}

impl WorthQueryProviderSessionLifecycle for SessionProtocolProvider {
    fn readmit_provider_plan(
        &self,
        plan: &WorthQueryProviderExecutionPlanView<'_>,
        admission: WorthQueryProviderSessionTokenAdmission,
    ) -> Result<WorthQueryProviderSessionToken, WorthQueryProviderSessionFailure> {
        self.calls.readmissions.fetch_add(1, Ordering::AcqRel);
        assert_eq!(plan.contract().provider_role(), "managed-graph");
        match self.failure {
            SessionFailurePoint::ReadmissionRejection => {
                Err(provider_rejection("readmission rejected"))
            }
            SessionFailurePoint::ReadmissionPanic => panic!("readmission panic"),
            _ => admission.admit("physical-session"),
        }
    }

    fn prepare_provider_session(
        &self,
        _session: &WorthQueryProviderSessionView<'_>,
    ) -> Result<(), WorthQueryProviderSessionFailure> {
        self.calls.preparations.fetch_add(1, Ordering::AcqRel);
        match self.failure {
            SessionFailurePoint::PreparationRejection => {
                Err(provider_rejection("preparation rejected"))
            }
            SessionFailurePoint::PreparationPanic => panic!("preparation panic"),
            _ => Ok(()),
        }
    }

    fn prepare_staged_session(
        &self,
        _session: &WorthQueryProviderSessionView<'_>,
    ) -> Result<(), WorthQueryProviderSessionFailure> {
        self.calls
            .staged_preparations
            .fetch_add(1, Ordering::AcqRel);
        match self.failure {
            SessionFailurePoint::StagedPreparationRejection => {
                Err(provider_rejection("staged preparation rejected"))
            }
            SessionFailurePoint::StagedPreparationPanic => panic!("staged preparation panic"),
            _ => Ok(()),
        }
    }

    fn commit_prepared_session(
        &self,
        _session: &WorthQueryProviderSessionView<'_>,
    ) -> Result<
        crate::domain_computation::WorthQueryProviderTerminalDescription,
        crate::domain_computation::WorthQueryProviderSessionCommitStop,
    > {
        self.calls.commits.fetch_add(1, Ordering::AcqRel);
        match self.failure {
            SessionFailurePoint::CommitRejection => {
                Err(provider_rejection("commit rejected").into())
            }
            SessionFailurePoint::CommitPanic => panic!("commit panic"),
            _ => Ok(
                crate::domain_computation::WorthQueryProviderTerminalDescription::new(
                    "provider commit",
                )
                .expect("fixture description is valid"),
            ),
        }
    }

    fn abort_provider_session(
        &self,
        _session: &WorthQueryProviderSessionView<'_>,
    ) -> Result<
        crate::domain_computation::WorthQueryProviderTerminalDescription,
        WorthQueryProviderSessionFailure,
    > {
        self.calls.aborts.fetch_add(1, Ordering::AcqRel);
        match self.failure {
            SessionFailurePoint::AbortRejection => Err(provider_rejection("abort rejected")),
            SessionFailurePoint::AbortPanic => panic!("abort panic"),
            _ => Ok(
                crate::domain_computation::WorthQueryProviderTerminalDescription::new(
                    "provider abort",
                )
                .expect("fixture description is valid"),
            ),
        }
    }
}

#[test]
fn sealed_plan_prepares_session_binds_work_and_aborts() {
    let calls = Arc::new(SessionCallCounts::default());
    let (mut running, graph) = session_run(SessionFailurePoint::None, Arc::clone(&calls), true);
    let expected_run = running.identity().to_owned();
    let expected_basis = running
        .provider_plan_bridge_basis()
        .identity()
        .as_str()
        .to_owned();
    let expected_snapshot = running.execution_snapshot_reference();
    {
        let plan = running
            .admit_provider_execution_plan(&graph)
            .expect("exact managed authorities should admit a provider plan");
        assert_eq!(plan.contract().provider_role(), "managed-graph");
        assert_eq!(plan.contract().managed_run_identity(), expected_run);
        assert_eq!(plan.contract().execution_basis_identity(), expected_basis);
        assert_eq!(plan.contract().snapshot_identity(), expected_snapshot);
        assert_eq!(
            plan.contract().graph_authority_identity(),
            graph.authority_identity()
        );
        assert_eq!(plan.contract().effect_closure(), ["mutation"]);
        assert_eq!(plan.counters().authority_checks(), 1);
        let readmitted = plan.readmit().expect("provider should readmit the plan");
        let prepared = readmitted
            .prepare()
            .expect("provider should prepare the session");
        let staged = prepared.bind_reads_and_effects();
        assert_eq!(
            staged.read_authority().token_identity(),
            staged.effect_authority().token_identity()
        );
        assert_eq!(staged.counters().provider_calls(), 2);
        let outcome = staged.abort();
        assert_eq!(
            outcome.recovery_posture(),
            WorthQueryProviderSessionRecoveryPosture::Closed
        );
        assert!(matches!(
            outcome,
            WorthQuerySessionCommitOrAbortOutcome::Aborted(_)
        ));
    }
    assert_eq!(calls.readmissions.load(Ordering::Acquire), 1);
    assert_eq!(calls.preparations.load(Ordering::Acquire), 1);
    assert_eq!(calls.aborts.load(Ordering::Acquire), 1);
    cleanup(running);
}

#[test]
fn independently_admitted_sessions_carry_distinct_opaque_affinities() {
    assert_ne!(closed_session_affinity(), closed_session_affinity());
}

fn closed_session_affinity() -> WorthQueryProviderSessionAffinityIdentity {
    let (mut running, graph) = session_run(
        SessionFailurePoint::None,
        Arc::new(SessionCallCounts::default()),
        true,
    );
    let staged = staged_session(&mut running, &graph);
    let affinity = staged.provider_session_affinity();
    assert_eq!(affinity.plan(), staged.plan());
    let identity = affinity.identity();
    assert!(matches!(
        staged.abort(),
        WorthQuerySessionCommitOrAbortOutcome::Aborted(_)
    ));
    cleanup(running);
    identity
}

#[test]
fn provider_without_session_lifecycle_is_denied_before_any_protocol_call() {
    let (mut running, graph) =
        managed_graph_run_with_provider(WorthQueryOperationGraphAccess::Observe, GraphOnlyProvider);
    let failure = running
        .admit_provider_execution_plan(&graph)
        .expect_err("one-shot-only provider must not enter the session lane");
    assert_eq!(
        failure.kind(),
        WorthQueryProviderSessionDenialKind::SessionProtocolUnsupported
    );
    cleanup(running);
}

struct GraphOnlyProvider;

impl WorthQueryGraphParticipationProvider<ManagedGraph> for GraphOnlyProvider {
    type Execution = UnusedSessionExecution;

    fn execution_resource_support(
        &self,
    ) -> worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupport {
        crate::domain_computation::provider_session::execution_resource_support("graph-only", 8)
    }

    fn begin(
        &self,
        _call: &WorthQueryGraphProviderCall,
        _start: &mut WorthQueryGraphProviderExecutionStart,
    ) -> Result<
        WorthQueryCooperativeGraphProviderExecution<Self::Execution>,
        WorthQueryGraphProviderFailure,
    > {
        unreachable!("unsupported session provider must deny before provider work")
    }
}

pub(super) fn session_run(
    failure: SessionFailurePoint,
    calls: Arc<SessionCallCounts>,
    touch: bool,
) -> (
    WorthQueryRunningDirectRun,
    WorthQueryInstalledGraphParticipationAuthority,
) {
    managed_session_graph_run_with_provider(
        WorthQueryOperationGraphAccess::Observe,
        SessionProtocolProvider { failure, calls },
        touch,
    )
}

pub(super) fn staged_session<'run>(
    running: &'run mut WorthQueryRunningDirectRun,
    graph: &WorthQueryInstalledGraphParticipationAuthority,
) -> crate::domain_computation::WorthQuerySessionBoundReadsAndEffects<'run> {
    running
        .admit_provider_execution_plan(graph)
        .expect("plan admission should succeed")
        .readmit()
        .expect("plan readmission should succeed")
        .prepare()
        .expect("session preparation should succeed")
        .bind_reads_and_effects()
}

pub(super) fn cleanup(running: WorthQueryRunningDirectRun) {
    running
        .terminate_for_convergence(WorthQueryManagedRunTerminalKind::Failed)
        .cleanup()
        .expect("session protocol fixture cleanup should complete");
}

fn provider_rejection(detail: &'static str) -> WorthQueryProviderSessionFailure {
    WorthQueryProviderSessionFailure::new(
        WorthQueryProviderSessionDenialKind::ProviderRejected,
        WorthQueryProviderSessionProtocolStage::PlanReadmission,
        detail,
        WorthQueryProviderSessionProtocolCounters::default(),
    )
}
