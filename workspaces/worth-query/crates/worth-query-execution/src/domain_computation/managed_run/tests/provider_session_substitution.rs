use std::sync::{Arc, Mutex};

use super::*;
use crate::domain_computation::{
    WorthQueryProviderExecutionPlanView, WorthQueryProviderSessionDenialKind,
    WorthQueryProviderSessionFailure, WorthQueryProviderSessionLifecycle,
    WorthQueryProviderSessionProtocolCounters, WorthQueryProviderSessionProtocolStage,
    WorthQueryProviderSessionRecoveryPosture, WorthQueryProviderSessionToken,
    WorthQueryProviderSessionTokenAdmission, WorthQueryProviderSessionView,
};

struct TokenSubstitutionProvider {
    captured: Arc<Mutex<Option<WorthQueryProviderSessionToken>>>,
}

struct TokenSubstitutionExecution;

struct TokenSubstitutionWorld {
    runtime: WorthQueryExecutionRuntime,
    graph: WorthQueryInstalledGraphParticipationAuthority,
    provider_support:
        worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupport,
}

impl WorthQueryGraphProviderExecution for TokenSubstitutionExecution {
    fn advance(
        &mut self,
        _step: &mut WorthQueryGraphProviderStep,
    ) -> Result<WorthQueryGraphProviderStepDisposition, WorthQueryGraphProviderFailure> {
        unreachable!("token substitution never enters one-shot graph execution")
    }

    fn dispose(&mut self) -> Result<(), WorthQueryGraphProviderFailure> {
        Ok(())
    }
}

impl WorthQueryGraphParticipationProvider<ManagedGraph> for TokenSubstitutionProvider {
    type Execution = TokenSubstitutionExecution;

    fn execution_resource_support(
        &self,
    ) -> worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupport {
        crate::domain_computation::provider_session::execution_resource_support(
            "token-substitution",
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
        unreachable!("session protocol must not route through the one-shot callback")
    }
}

impl WorthQueryProviderSessionLifecycle for TokenSubstitutionProvider {
    fn readmit_provider_plan(
        &self,
        _plan: &WorthQueryProviderExecutionPlanView<'_>,
        admission: WorthQueryProviderSessionTokenAdmission,
    ) -> Result<WorthQueryProviderSessionToken, WorthQueryProviderSessionFailure> {
        let mut captured = self
            .captured
            .lock()
            .expect("token substitution test mutex should remain healthy");
        if let Some(foreign) = captured.take() {
            return Ok(foreign);
        }
        *captured = Some(admission.admit("captured-physical-session")?);
        Err(provider_rejection("provider retained the first plan token"))
    }

    fn prepare_provider_session(
        &self,
        _session: &WorthQueryProviderSessionView<'_>,
    ) -> Result<(), WorthQueryProviderSessionFailure> {
        unreachable!("a substituted token must fail before preparation")
    }

    fn prepare_staged_session(
        &self,
        _session: &WorthQueryProviderSessionView<'_>,
    ) -> Result<(), WorthQueryProviderSessionFailure> {
        unreachable!("a substituted token must fail before staged preparation")
    }

    fn commit_prepared_session(
        &self,
        _session: &WorthQueryProviderSessionView<'_>,
        _allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<
        crate::domain_computation::WorthQueryProviderTerminalDescription,
        crate::domain_computation::WorthQueryProviderSessionCommitStop,
    > {
        unreachable!("a substituted token must fail before commit")
    }

    fn abort_provider_session(
        &self,
        _session: &WorthQueryProviderSessionView<'_>,
    ) -> Result<
        crate::domain_computation::WorthQueryProviderTerminalDescription,
        WorthQueryProviderSessionFailure,
    > {
        unreachable!("a substituted token must fail before abort")
    }
}

#[test]
fn token_minted_for_an_earlier_plan_cannot_open_a_later_plan() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let world = token_substitution_world();
        let first_plan = world.admitted_plan();
        let first_operation = direct_authority_with_graph(
            &world.runtime,
            &first_plan,
            &world.graph,
            WorthQueryOperationGraphAccess::Observe,
        );
        let mut first = start_run(
            &world.runtime,
            &first_operation,
            first_plan,
            resource_request,
        );
        let first_identity = first.identity().to_owned();
        let first_failure = first
            .admit_provider_execution_plan(&world.graph)
            .expect("first plan should admit")
            .readmit(execution)
            .expect_err("provider intentionally retains the first token");
        assert_eq!(
            first_failure.kind(),
            WorthQueryProviderSessionDenialKind::ProviderRejected
        );
        cleanup(first);

        let second_plan = world.admitted_plan();
        let second_operation = direct_authority_with_graph(
            &world.runtime,
            &second_plan,
            &world.graph,
            WorthQueryOperationGraphAccess::Observe,
        );
        let mut second = start_run(
            &world.runtime,
            &second_operation,
            second_plan,
            resource_request,
        );
        assert_ne!(second.identity(), first_identity);
        let substitution = second
            .admit_provider_execution_plan(&world.graph)
            .expect("second plan should admit")
            .readmit(execution)
            .expect_err("token minted for the first plan must not substitute");
        assert_eq!(
            substitution.kind(),
            WorthQueryProviderSessionDenialKind::TokenNotMintedForPlan
        );
        assert_eq!(
            substitution.recovery_posture(),
            WorthQueryProviderSessionRecoveryPosture::RecoveryRequired
        );
        cleanup(second);
    });
}

fn token_substitution_world() -> TokenSubstitutionWorld {
    let captured = Arc::new(Mutex::new(None));
    let installer = WorthQueryExecutionRuntimeInstaller::new();
    let anchor = Arc::new(
        crate::domain_computation::provider_session::graph_provider::bounded_step::provider_anchor::WorthQueryGraphProviderAnchor::install_session_capable::<ManagedGraph, _>(
            TokenSubstitutionProvider {
                captured: Arc::clone(&captured),
            },
        ),
    );
    let provider_support = anchor.resource_support().clone();
    let graph = WorthQueryInstalledGraphParticipationAuthority::install(
        installer.installation_runtime(),
        "managed-graph",
        anchor.provider_identity(),
        false,
        Option::<String>::None,
        anchor,
    )
    .expect("substitution graph should install");
    let runtime = installer
        .install(
            worth_query_installation::facade::WorthQueryInstallationGeneration::initial(),
            std::iter::empty(),
        )
        .expect("substitution runtime should install")
        .into_parts()
        .0;
    TokenSubstitutionWorld {
        runtime,
        graph,
        provider_support,
    }
}

impl TokenSubstitutionWorld {
    fn admitted_plan(
        &self,
    ) -> worth_query_admission::facade::resource_admission::WorthQueryAdmittedExecutionResourcePlan
    {
        admitted_plan_with_graph_support(
            "token-substitution-binding",
            8,
            self.graph.role(),
            self.provider_support.clone(),
        )
    }
}

fn start_run(
    runtime: &WorthQueryExecutionRuntime,
    operation: &crate::domain_computation::WorthQueryExecutionBoundOperationAuthority,
    plan: worth_query_admission::facade::resource_admission::WorthQueryAdmittedExecutionResourcePlan,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> WorthQueryRunningDirectRun {
    let attempt = runtime
        .start_direct_resource_attempt(operation, plan)
        .expect("substitution attempt should start");
    let lower = causal_fixture::managed_admission_context();
    runtime
        .managed_run_admission(&lower.bridge, &lower.relational)
        .admit_direct(operation, attempt, lower.read_request(), resource_request)
        .expect("substitution run should admit")
        .start()
}

fn cleanup(running: WorthQueryRunningDirectRun) {
    running
        .terminate_for_convergence(WorthQueryManagedRunTerminalKind::Failed)
        .cleanup()
        .expect("substitution fixture cleanup should complete");
}

fn provider_rejection(detail: &'static str) -> WorthQueryProviderSessionFailure {
    WorthQueryProviderSessionFailure::new(
        WorthQueryProviderSessionDenialKind::ProviderRejected,
        WorthQueryProviderSessionProtocolStage::PlanReadmission,
        detail,
        WorthQueryProviderSessionProtocolCounters::default(),
    )
}
