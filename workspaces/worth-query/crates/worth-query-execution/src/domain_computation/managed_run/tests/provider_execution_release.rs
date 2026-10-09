use super::*;
use crate::domain_computation::{
    WorthQueryProviderExecutionDestructorDisposition,
    WorthQueryProviderExecutionDisposalDisposition,
};

#[derive(Clone, Copy)]
enum AdvanceBehavior {
    Complete,
    Panic,
}

#[derive(Clone, Copy)]
enum DisposalBehavior {
    Complete,
    Reject,
}

struct PhysicalReleaseProvider {
    advance: AdvanceBehavior,
    disposal: DisposalBehavior,
    destructor_panics: bool,
    disposal_attempts: Arc<AtomicUsize>,
    destructor_attempts: Arc<AtomicUsize>,
}

struct PhysicalReleaseExecution {
    advance: AdvanceBehavior,
    disposal: DisposalBehavior,
    destructor_panics: bool,
    disposal_attempts: Arc<AtomicUsize>,
    destructor_attempts: Arc<AtomicUsize>,
}

impl WorthQueryGraphProviderExecution for PhysicalReleaseExecution {
    fn advance(
        &mut self,
        step: &mut WorthQueryGraphProviderStep,
    ) -> Result<WorthQueryGraphProviderStepDisposition, WorthQueryGraphProviderFailure> {
        match self.advance {
            AdvanceBehavior::Complete => {
                step.perform_work_unit(|| Ok(()))?;
                WorthQueryGraphProviderStepDisposition::complete("physical-release")
                    .map_err(WorthQueryGraphProviderFailure::new)
            }
            AdvanceBehavior::Panic => {
                step.perform_work_unit(|| -> Result<(), WorthQueryGraphProviderFailure> {
                    panic!("provider invocation panicked")
                })?;
                unreachable!("panicking provider work cannot return")
            }
        }
    }

    fn dispose(&mut self) -> Result<(), WorthQueryGraphProviderFailure> {
        self.disposal_attempts.fetch_add(1, Ordering::AcqRel);
        match self.disposal {
            DisposalBehavior::Complete => Ok(()),
            DisposalBehavior::Reject => Err(WorthQueryGraphProviderFailure::new(
                "managed provider disposal rejected",
            )),
        }
    }
}

impl Drop for PhysicalReleaseExecution {
    fn drop(&mut self) {
        self.destructor_attempts.fetch_add(1, Ordering::AcqRel);
        assert!(
            !self.destructor_panics,
            "managed provider execution destructor panicked"
        );
    }
}

impl WorthQueryGraphParticipationProvider<ManagedGraph> for PhysicalReleaseProvider {
    type Execution = PhysicalReleaseExecution;

    fn execution_resource_support(
        &self,
    ) -> worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupport {
        crate::domain_computation::provider_session::execution_resource_support(
            "managed-physical-release",
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
        admit_provider_execution(
            start,
            PhysicalReleaseExecution {
                advance: self.advance,
                disposal: self.disposal,
                destructor_panics: self.destructor_panics,
                disposal_attempts: Arc::clone(&self.disposal_attempts),
                destructor_attempts: Arc::clone(&self.destructor_attempts),
            },
        )
    }
}

#[test]
fn physical_release_failure_terminalizes_with_exact_recovery_evidence() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let phase = &active_phase;
        let execution = phase;

        for (disposal, destructor_panics, expected_disposal, expected_destructor) in [
            (
                DisposalBehavior::Reject,
                false,
                WorthQueryProviderExecutionDisposalDisposition::Rejected,
                WorthQueryProviderExecutionDestructorDisposition::Completed,
            ),
            (
                DisposalBehavior::Complete,
                true,
                WorthQueryProviderExecutionDisposalDisposition::Completed,
                WorthQueryProviderExecutionDestructorDisposition::Panicked,
            ),
            (
                DisposalBehavior::Reject,
                true,
                WorthQueryProviderExecutionDisposalDisposition::Rejected,
                WorthQueryProviderExecutionDestructorDisposition::Panicked,
            ),
        ] {
            let (provider, disposal_attempts, destructor_attempts) =
                provider(AdvanceBehavior::Complete, disposal, destructor_panics);
            let (running, graph) =
                managed_graph_run_with_provider(WorthQueryOperationGraphAccess::Observe, provider);
            let active = running
                .begin_graph_execution(
                    execution,
                    &graph,
                    WorthQueryManagedGraphCallRequest::new(
                        WorthQueryGraphProviderCallKind::Observe,
                        "physical-release-failure",
                    ),
                )
                .expect("physical-release provider should start");
            let terminal = match active.advance(execution) {
                WorthQueryDirectGraphStepOutcome::Failed(terminal) => terminal,
                _ => panic!("physical-release failure returned reusable running authority"),
            };
            assert_eq!(terminal.provider_work().completed_work_units(), 1);
            assert_eq!(terminal.provider_work().admitted_receipt_count(), 1);
            let summary = terminal.provider_work().provider_execution_release();
            assert_eq!(summary.release_count(), 1);
            assert_eq!(
                summary.completed_disposal_count(),
                usize::from(
                    expected_disposal == WorthQueryProviderExecutionDisposalDisposition::Completed
                )
            );
            assert_eq!(
                summary.rejected_disposal_count(),
                usize::from(
                    expected_disposal == WorthQueryProviderExecutionDisposalDisposition::Rejected
                )
            );
            assert_eq!(summary.panicked_disposal_count(), 0);
            assert_eq!(
                summary.completed_destructor_count(),
                usize::from(
                    expected_destructor
                        == WorthQueryProviderExecutionDestructorDisposition::Completed
                )
            );
            assert_eq!(
                summary.panicked_destructor_count(),
                usize::from(
                    expected_destructor
                        == WorthQueryProviderExecutionDestructorDisposition::Panicked
                )
            );
            let recovery = summary
                .recovery_evidence()
                .expect("physical-release failure must remain exact");
            assert_eq!(recovery.disposal(), expected_disposal);
            assert_eq!(recovery.destructor(), expected_destructor);
            assert_eq!(disposal_attempts.load(Ordering::Acquire), 1);
            assert_eq!(destructor_attempts.load(Ordering::Acquire), 1);
            let cleanup = terminal
                .cleanup()
                .expect("physical-release recovery retains lower cleanup authority");
            assert_eq!(
                cleanup.inspection().disposition(),
                WorthQueryManagedRunCleanupDisposition::RecoveryRequired
            );
            assert!(cleanup.inspection().resources_released());
            assert_eq!(cleanup.inspection().released_reservation_count(), 2);
        }
    });
}

#[test]
fn workflow_disposal_rejection_terminalizes_before_returning_running_authority() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let phase = &active_phase;
        let execution = phase;

        let installer = WorthQueryExecutionRuntimeInstaller::new();
        let (provider, disposal_attempts, destructor_attempts) =
            provider(AdvanceBehavior::Complete, DisposalBehavior::Reject, false);
        let provider_anchor = Arc::new(
        crate::domain_computation::provider_session::graph_provider::bounded_step::provider_anchor::WorthQueryGraphProviderAnchor::install::<ManagedGraph, _>(
            provider,
        ),
    );
        let provider_support = provider_anchor.resource_support().clone();
        let graph = super::workflow_provider_steps::installed_graph(
            &installer,
            "workflow-release-graph",
            provider_anchor,
        );
        let runtime =
            super::workflow_provider_steps::installed_runtime(installer, "workflow release graph");
        let operation_resources = admitted_plan("workflow-release-binding", 8);
        let stage_resources = admitted_plan_with_graph_support(
            "workflow-release-binding:stage",
            4,
            graph.role(),
            provider_support,
        );
        let resources = WorthQueryAdmittedWorkflowResourcePlan::assemble(
            operation_resources,
            BTreeMap::from([("stage".to_owned(), stage_resources)]),
        );
        let operation = workflow_authority_with_stage_graph(
            &runtime,
            &resources,
            "stage",
            &graph,
            WorthQueryOperationGraphAccess::Observe,
        );
        let running =
            super::workflow_provider_steps::admitted_workflow(&runtime, &operation, resources);
        let active = running
            .begin_stage_graph_execution(
                execution,
                "stage",
                &graph,
                WorthQueryManagedGraphCallRequest::new(
                    WorthQueryGraphProviderCallKind::Observe,
                    "workflow-release-failure",
                ),
            )
            .expect("workflow release provider should start");
        let terminal = match active.advance(execution) {
            WorthQueryWorkflowGraphStepOutcome::Failed(terminal) => terminal,
            _ => panic!("workflow disposal rejection returned reusable running authority"),
        };
        assert_eq!(terminal.provider_work().completed_work_units(), 1);
        assert_eq!(terminal.provider_work().admitted_receipt_count(), 1);
        let recovery = terminal
            .provider_work()
            .provider_execution_release()
            .recovery_evidence()
            .expect("workflow terminal must retain exact release recovery");
        assert_eq!(
            recovery.disposal(),
            WorthQueryProviderExecutionDisposalDisposition::Rejected
        );
        assert_eq!(
            recovery.destructor(),
            WorthQueryProviderExecutionDestructorDisposition::Completed
        );
        assert_eq!(
            recovery.disposal_failure_detail(),
            Some("managed provider disposal rejected")
        );
        assert_eq!(disposal_attempts.load(Ordering::Acquire), 1);
        assert_eq!(destructor_attempts.load(Ordering::Acquire), 1);
        let cleanup = match terminal.cleanup() {
            WorthQueryWorkflowRunCleanupOutcome::Complete(cleanup) => cleanup,
            WorthQueryWorkflowRunCleanupOutcome::Pending(_) => {
                panic!("artifact-free workflow release recovery remained pending")
            }
            WorthQueryWorkflowRunCleanupOutcome::RecoveryRequired(failure) => {
                panic!("provider release recovery became lower cleanup failure: {failure:?}")
            }
        };
        assert_eq!(
            cleanup.inspection().disposition(),
            WorthQueryManagedRunCleanupDisposition::RecoveryRequired
        );
    });
}

#[test]
fn invocation_panic_and_destructor_panic_are_independently_evidenced() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let phase = &active_phase;
        let execution = phase;

        for destructor_panics in [false, true] {
            let (provider, disposal_attempts, destructor_attempts) = provider(
                AdvanceBehavior::Panic,
                DisposalBehavior::Complete,
                destructor_panics,
            );
            let (running, graph) =
                managed_graph_run_with_provider(WorthQueryOperationGraphAccess::Project, provider);
            let active = running
                .begin_graph_execution(
                    execution,
                    &graph,
                    WorthQueryManagedGraphCallRequest::new(
                        WorthQueryGraphProviderCallKind::Project,
                        "invocation-release-boundaries",
                    ),
                )
                .expect("panic probe should start before its bounded step");
            let terminal = match active.advance(execution) {
                WorthQueryDirectGraphStepOutcome::Failed(terminal) => terminal,
                _ => panic!("provider invocation panic did not terminalize"),
            };
            assert_eq!(terminal.provider_work().completed_work_units(), 0);
            assert_eq!(terminal.provider_work().abandoned_call_count(), 1);
            let invocation = terminal.provider_work().last_step_failure().unwrap();
            assert_eq!(
                invocation.invocation(),
                WorthQueryGraphProviderStepInvocationDisposition::Panicked
            );
            assert_eq!(invocation.governed_denial_kind(), None);
            let summary = terminal.provider_work().provider_execution_release();
            assert_eq!(summary.release_count(), 1);
            assert_eq!(summary.recovery_evidence().is_some(), destructor_panics);
            assert_eq!(disposal_attempts.load(Ordering::Acquire), 1);
            assert_eq!(destructor_attempts.load(Ordering::Acquire), 1);
            let cleanup = terminal
                .cleanup()
                .expect("invocation panic retains lower cleanup authority");
            assert_eq!(
                cleanup.inspection().disposition(),
                WorthQueryManagedRunCleanupDisposition::RecoveryRequired
            );
        }
    });
}

fn provider(
    advance: AdvanceBehavior,
    disposal: DisposalBehavior,
    destructor_panics: bool,
) -> (PhysicalReleaseProvider, Arc<AtomicUsize>, Arc<AtomicUsize>) {
    let disposal_attempts = Arc::new(AtomicUsize::new(0));
    let destructor_attempts = Arc::new(AtomicUsize::new(0));
    (
        PhysicalReleaseProvider {
            advance,
            disposal,
            destructor_panics,
            disposal_attempts: Arc::clone(&disposal_attempts),
            destructor_attempts: Arc::clone(&destructor_attempts),
        },
        disposal_attempts,
        destructor_attempts,
    )
}
