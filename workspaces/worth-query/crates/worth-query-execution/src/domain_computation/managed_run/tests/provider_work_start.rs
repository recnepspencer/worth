use super::*;

enum StartBehavior {
    Retain,
    Deny,
    Panic,
}

struct StartProvider(StartBehavior);

struct StartExecution {
    _retained: WorthQueryGraphProviderRetainedMemory,
}

impl WorthQueryGraphProviderExecution for StartExecution {
    fn advance(
        &mut self,
        step: &mut WorthQueryGraphProviderStep,
    ) -> Result<WorthQueryGraphProviderStepDisposition, WorthQueryGraphProviderFailure> {
        step.perform_work_unit(|| Ok(()))?;
        WorthQueryGraphProviderStepDisposition::complete("start-retained")
            .map_err(WorthQueryGraphProviderFailure::new)
    }

    fn dispose(&mut self) -> Result<(), WorthQueryGraphProviderFailure> {
        Ok(())
    }
}

impl WorthQueryGraphParticipationProvider<ManagedGraph> for StartProvider {
    type Execution = StartExecution;

    fn execution_resource_support(
        &self,
    ) -> worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupport {
        crate::domain_computation::provider_session::execution_resource_support(
            "managed-provider-start",
            8,
        )
    }

    fn begin(
        &self,
        _call: &WorthQueryGraphProviderCall,
        start: &mut WorthQueryGraphProviderExecutionStart,
    ) -> Result<
        WorthQueryCooperativeGraphProviderExecution<Self::Execution>,
        WorthQueryGraphProviderFailure,
    > {
        match self.0 {
            StartBehavior::Retain => {
                let execution = StartExecution {
                    _retained: start.retain_bytes(4).map_err(start_step_failure)?,
                };
                admit_provider_execution(start, execution)
            }
            StartBehavior::Deny => {
                let _ = start.retain_bytes(4_097);
                Err(WorthQueryGraphProviderFailure::new(
                    "provider returned after ignoring start denial",
                ))
            }
            StartBehavior::Panic => {
                let _retained = start.retain_bytes(4).map_err(start_step_failure)?;
                panic!("provider construction panicked after governed retention")
            }
        }
    }
}

#[test]
fn governed_provider_start_classifies_denial_and_panic_before_execution() {
    let (running, graph) = managed_graph_run_with_provider(
        WorthQueryOperationGraphAccess::Observe,
        StartProvider(StartBehavior::Deny),
    );
    let denial = match running.begin_graph_execution(
        &graph,
        WorthQueryManagedGraphCallRequest::new(
            WorthQueryGraphProviderCallKind::Observe,
            "start-denial",
        ),
    ) {
        Ok(_) => panic!("ignored start-memory denial admitted provider construction"),
        Err(denial) => denial,
    };
    assert_eq!(
        denial.kind(),
        crate::domain_computation::WorthQueryDirectGraphExecutionStartFailureKind::
            ProviderStartContractDenied
    );
    assert_eq!(denial.provider_retained_bytes(), 0);
    assert_eq!(denial.provider_retained_allocation_count(), 0);

    let (running, graph) = managed_graph_run_with_provider(
        WorthQueryOperationGraphAccess::Observe,
        StartProvider(StartBehavior::Panic),
    );
    let panic = match running.begin_graph_execution(
        &graph,
        WorthQueryManagedGraphCallRequest::new(
            WorthQueryGraphProviderCallKind::Observe,
            "start-panic",
        ),
    ) {
        Ok(_) => panic!("provider construction panic escaped typed failure"),
        Err(panic) => panic,
    };
    assert_eq!(
        panic.kind(),
        crate::domain_computation::WorthQueryDirectGraphExecutionStartFailureKind::
            ProviderStartPanicked
    );
    assert_eq!(panic.provider_retained_bytes(), 0);
    assert_eq!(panic.provider_retained_allocation_count(), 0);
}

#[test]
fn active_abandonment_releases_start_retention_and_preserves_peak_evidence() {
    let (running, graph) = managed_graph_run_with_provider(
        WorthQueryOperationGraphAccess::Observe,
        StartProvider(StartBehavior::Retain),
    );
    let active = running
        .begin_graph_execution(
            &graph,
            WorthQueryManagedGraphCallRequest::new(
                WorthQueryGraphProviderCallKind::Observe,
                "abandon-start-retention",
            ),
        )
        .expect("governed start retention should admit within the installed ceiling");
    let terminal = match active.abandon() {
        WorthQueryDirectGraphStepOutcome::Failed(terminal) => terminal,
        _ => panic!("explicit active abandonment must produce a failed terminal"),
    };
    assert_eq!(terminal.provider_work().retained_bytes(), 0);
    assert_eq!(terminal.provider_work().peak_retained_bytes(), 4);
    assert_eq!(terminal.provider_work().abandoned_call_count(), 1);
    assert_eq!(
        terminal
            .provider_work()
            .provider_execution_release()
            .release_count(),
        1
    );
    let cleanup = terminal
        .cleanup()
        .expect("explicit abandonment preserves cleanup authority");
    assert_eq!(
        cleanup.inspection().disposition(),
        WorthQueryManagedRunCleanupDisposition::RecoveryRequired
    );
}

fn start_step_failure(
    denial: crate::domain_computation::WorthQueryGraphProviderStepDenial,
) -> WorthQueryGraphProviderFailure {
    WorthQueryGraphProviderFailure::new(denial.detail())
}
