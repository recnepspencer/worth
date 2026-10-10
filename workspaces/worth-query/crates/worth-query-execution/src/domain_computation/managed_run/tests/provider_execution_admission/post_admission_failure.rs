use super::*;

pub(super) fn post_admission_failure(
    execution: &crate::domain_computation::primary_graph::TestHostRequest<'_>,

    behavior: PostAdmissionBehavior,
    destructor_panics: bool,
    disposal_attempts: Arc<AtomicUsize>,
    destructor_attempts: Arc<AtomicUsize>,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> crate::domain_computation::WorthQueryDirectGraphExecutionStartFailure {
    let (running, graph) = managed_graph_run_with_provider(
        WorthQueryOperationGraphAccess::Observe,
        PostAdmissionProvider {
            behavior,
            disposal_attempts,
            destructor_attempts,
            destructor_panics,
        },
        resource_request,
    );
    let installed_running_phase = execution.phase_for_direct(&running);

    match running.begin_graph_execution(
        &installed_running_phase,
        &graph,
        WorthQueryManagedGraphCallRequest::new(
            WorthQueryGraphProviderCallKind::Observe,
            "post-admission-failure",
        ),
    ) {
        Ok(_) => panic!("post-admission provider failure returned active execution authority"),
        Err(failure) => failure,
    }
}
