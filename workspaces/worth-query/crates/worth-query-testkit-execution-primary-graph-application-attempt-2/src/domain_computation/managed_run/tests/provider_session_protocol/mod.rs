use crate::domain_computation::managed_run::tests::managed_session_graph_run_with_provider;
use crate::domain_computation::managed_run::tests::ManagedGraph;
use worth_query_execution::facade::domain_computation::WorthQueryCooperativeGraphProviderExecution;
use worth_query_execution::facade::domain_computation::WorthQueryGraphProviderCall;
use worth_query_execution::facade::domain_computation::WorthQueryGraphProviderExecutionStart;
use worth_query_execution::facade::domain_computation::WorthQueryGraphProviderFailure;
use worth_query_execution::facade::domain_computation::WorthQueryGraphProviderStep;
use worth_query_execution::facade::domain_computation::WorthQueryGraphProviderStepDisposition;
use worth_query_execution::facade::domain_computation::WorthQueryManagedRunTerminalKind;
use worth_query_execution::facade::domain_computation::WorthQueryRunningDirectRun;
use worth_query_installation::facade::WorthQueryInstalledGraphParticipationAuthority;
use worth_query_installation::facade::WorthQueryOperationGraphAccess;

use crate::domain_computation::managed_run::tests::*;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use worth_query_execution::court_doors_for_test::domain_computation::provider_session::protocol::session_affinity::WorthQueryProviderSessionAffinityIdentity;
use worth_query_execution::facade::domain_computation::WorthQueryProviderExecutionPlanView;
use worth_query_execution::facade::domain_computation::WorthQueryProviderSessionDenialKind;
use worth_query_execution::facade::domain_computation::WorthQueryProviderSessionFailure;
use worth_query_execution::facade::domain_computation::WorthQueryProviderSessionLifecycle;
use worth_query_execution::facade::domain_computation::WorthQueryProviderSessionProtocolCounters;
use worth_query_execution::facade::domain_computation::WorthQueryProviderSessionProtocolStage;
use worth_query_execution::facade::domain_computation::WorthQueryProviderSessionToken;
use worth_query_execution::facade::domain_computation::WorthQueryProviderSessionTokenAdmission;
use worth_query_execution::facade::domain_computation::WorthQueryProviderSessionView;
use worth_query_execution::facade::domain_computation::WorthQuerySessionAbortOutcome;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SessionFailurePoint {
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
pub struct SessionCallCounts {
    pub readmissions: AtomicUsize,
    pub preparations: AtomicUsize,
    pub staged_preparations: AtomicUsize,
    pub commits: AtomicUsize,
    pub aborts: AtomicUsize,
}
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SessionCallObservation {
    pub readmissions: usize,
    pub preparations: usize,
    pub staged_preparations: usize,
    pub commits: usize,
    pub aborts: usize,
}
impl SessionCallCounts {
    pub fn observe(&self) -> SessionCallObservation {
        SessionCallObservation {
            readmissions: self.readmissions.load(Ordering::Acquire),
            preparations: self.preparations.load(Ordering::Acquire),
            staged_preparations: self.staged_preparations.load(Ordering::Acquire),
            commits: self.commits.load(Ordering::Acquire),
            aborts: self.aborts.load(Ordering::Acquire),
        }
    }
}
pub struct SessionProtocolProvider {
    pub failure: SessionFailurePoint,
    pub calls: Arc<SessionCallCounts>,
}
pub struct UnusedSessionExecution;
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
        crate::domain_computation::provider_session::tests::execution_resource_support(
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
        _allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<
        worth_query_execution::facade::domain_computation::WorthQueryProviderTerminalDescription,
        worth_query_execution::facade::domain_computation::WorthQueryProviderSessionCommitStop,
    > {
        self.calls.commits.fetch_add(1, Ordering::AcqRel);
        match self.failure {
            SessionFailurePoint::CommitRejection => {
                Err(provider_rejection("commit rejected").into())
            }
            SessionFailurePoint::CommitPanic => panic!("commit panic"),
            _ => Ok(
                worth_query_execution::facade::domain_computation::WorthQueryProviderTerminalDescription::new(
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
        worth_query_execution::facade::domain_computation::WorthQueryProviderTerminalDescription,
        WorthQueryProviderSessionFailure,
    > {
        self.calls.aborts.fetch_add(1, Ordering::AcqRel);
        match self.failure {
            SessionFailurePoint::AbortRejection => Err(provider_rejection("abort rejected")),
            SessionFailurePoint::AbortPanic => panic!("abort panic"),
            _ => Ok(
                worth_query_execution::facade::domain_computation::WorthQueryProviderTerminalDescription::new(
                    "provider abort",
                )
                .expect("fixture description is valid"),
            ),
        }
    }
}
pub fn closed_session_affinity(
    execution: &worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::test_host_policy::TestHostRequest<'_>,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> WorthQueryProviderSessionAffinityIdentity {
    let (mut running, graph) = session_run(
        SessionFailurePoint::None,
        Arc::new(SessionCallCounts::default()),
        true,
        resource_request,
    );
    let staged = staged_session(execution, &mut running, &graph);
    let affinity = staged.provider_session_affinity();
    assert_eq!(affinity.plan(), staged.plan());
    let identity = affinity.identity();
    assert!(matches!(
        staged.abort(),
        WorthQuerySessionAbortOutcome::Aborted(_)
    ));
    cleanup(running);
    identity
}
pub struct GraphOnlyProvider;
impl WorthQueryGraphParticipationProvider<ManagedGraph> for GraphOnlyProvider {
    type Execution = UnusedSessionExecution;

    fn execution_resource_support(
        &self,
    ) -> worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupport {
        crate::domain_computation::provider_session::tests::execution_resource_support(
            "graph-only",
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
        unreachable!("unsupported session provider must deny before provider work")
    }
}
pub fn session_run(
    failure: SessionFailurePoint,
    calls: Arc<SessionCallCounts>,
    touch: bool,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> (
    WorthQueryRunningDirectRun,
    WorthQueryInstalledGraphParticipationAuthority,
) {
    managed_session_graph_run_with_provider(
        WorthQueryOperationGraphAccess::Observe,
        SessionProtocolProvider { failure, calls },
        touch,
        resource_request,
    )
}
pub fn staged_session<'run>(
    execution: &'run worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::test_host_policy::TestHostRequest<'_>,

    running: &'run mut WorthQueryRunningDirectRun,
    graph: &WorthQueryInstalledGraphParticipationAuthority,
) -> worth_query_execution::facade::domain_computation::WorthQuerySessionBoundReadsAndEffects<'run>
{
    {
        let installed_fixture_phase = running.fixture_phase_for_test(execution);
        running
            .admit_provider_execution_plan(graph)
            .expect("plan admission should succeed")
            .readmit(&installed_fixture_phase)
    }
    .expect("plan readmission should succeed")
    .prepare()
    .expect("session preparation should succeed")
    .bind_reads_and_effects()
}
pub fn cleanup(running: WorthQueryRunningDirectRun) {
    running
        .terminate_for_convergence_for_test(WorthQueryManagedRunTerminalKind::Failed)
        .cleanup()
        .expect("session protocol fixture cleanup should complete");
}
pub fn provider_rejection(detail: &'static str) -> WorthQueryProviderSessionFailure {
    WorthQueryProviderSessionFailure::new(
        WorthQueryProviderSessionDenialKind::ProviderRejected,
        WorthQueryProviderSessionProtocolStage::PlanReadmission,
        detail,
        WorthQueryProviderSessionProtocolCounters::default(),
    )
}
