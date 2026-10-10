use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use super::*;
use crate::domain_computation::{
    WorthQueryProviderExecutionDestructorDisposition,
    WorthQueryProviderExecutionDisposalDisposition,
};

struct AdmissionExecution;

impl WorthQueryGraphProviderExecution for AdmissionExecution {
    fn advance(
        &mut self,
        step: &mut WorthQueryGraphProviderStep,
    ) -> Result<WorthQueryGraphProviderStepDisposition, WorthQueryGraphProviderFailure> {
        step.perform_work_unit(|| Ok(()))?;
        WorthQueryGraphProviderStepDisposition::complete("admission-execution")
            .map_err(WorthQueryGraphProviderFailure::new)
    }

    fn dispose(&mut self) -> Result<(), WorthQueryGraphProviderFailure> {
        Ok(())
    }
}

struct ForeignAdmissionProvider {
    retained: Arc<Mutex<Option<WorthQueryCooperativeGraphProviderExecution<AdmissionExecution>>>>,
}

impl WorthQueryGraphParticipationProvider<ManagedGraph> for ForeignAdmissionProvider {
    type Execution = AdmissionExecution;

    fn execution_resource_support(
        &self,
    ) -> worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupport {
        crate::domain_computation::provider_session::execution_resource_support(
            "foreign-execution-admission",
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
        let mut retained = self.retained.lock().unwrap();
        if let Some(foreign) = retained.take() {
            return Ok(foreign);
        }
        *retained = Some(
            start
                .admit_cooperative_execution(|| AdmissionExecution)
                .map_err(step_failure)?,
        );
        Err(WorthQueryGraphProviderFailure::new(
            "retain the first execution admission for a hostile retry",
        ))
    }
}

struct MultipleAdmissionProvider;

#[derive(Clone, Copy)]
enum PostAdmissionBehavior {
    Reject,
    Panic,
}

struct PostAdmissionProvider {
    behavior: PostAdmissionBehavior,
    disposal_attempts: Arc<AtomicUsize>,
    destructor_attempts: Arc<AtomicUsize>,
    destructor_panics: bool,
}

struct PostAdmissionExecution {
    disposal_attempts: Arc<AtomicUsize>,
    destructor_attempts: Arc<AtomicUsize>,
    destructor_panics: bool,
}

impl WorthQueryGraphProviderExecution for PostAdmissionExecution {
    fn advance(
        &mut self,
        _step: &mut WorthQueryGraphProviderStep,
    ) -> Result<WorthQueryGraphProviderStepDisposition, WorthQueryGraphProviderFailure> {
        unreachable!("post-admission failure must prevent provider advancement")
    }

    fn dispose(&mut self) -> Result<(), WorthQueryGraphProviderFailure> {
        self.disposal_attempts.fetch_add(1, Ordering::AcqRel);
        Ok(())
    }
}

impl Drop for PostAdmissionExecution {
    fn drop(&mut self) {
        self.destructor_attempts.fetch_add(1, Ordering::AcqRel);
        assert!(
            !self.destructor_panics,
            "post-admission provider execution destructor panicked"
        );
    }
}

impl WorthQueryGraphParticipationProvider<ManagedGraph> for PostAdmissionProvider {
    type Execution = PostAdmissionExecution;

    fn execution_resource_support(
        &self,
    ) -> worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupport {
        crate::domain_computation::provider_session::execution_resource_support(
            "post-admission-failure",
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
        let _admission = start
            .admit_cooperative_execution(|| PostAdmissionExecution {
                disposal_attempts: Arc::clone(&self.disposal_attempts),
                destructor_attempts: Arc::clone(&self.destructor_attempts),
                destructor_panics: self.destructor_panics,
            })
            .map_err(step_failure)?;
        match self.behavior {
            PostAdmissionBehavior::Reject => Err(WorthQueryGraphProviderFailure::new(
                "provider rejected after execution admission",
            )),
            PostAdmissionBehavior::Panic => {
                panic!("provider panicked after execution admission")
            }
        }
    }
}

impl WorthQueryGraphParticipationProvider<ManagedGraph> for MultipleAdmissionProvider {
    type Execution = AdmissionExecution;

    fn execution_resource_support(
        &self,
    ) -> worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupport {
        crate::domain_computation::provider_session::execution_resource_support(
            "multiple-execution-admission",
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
        let first = start
            .admit_cooperative_execution(|| AdmissionExecution)
            .map_err(step_failure)?;
        let _ignored_denial = start.admit_cooperative_execution(|| AdmissionExecution);
        Ok(first)
    }
}

#[test]
fn admission_from_a_prior_start_cannot_enter_a_fresh_provider_start() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let retained = Arc::new(Mutex::new(None));
        let (running, graph) = managed_graph_run_with_provider(
            WorthQueryOperationGraphAccess::Observe,
            ForeignAdmissionProvider {
                retained: Arc::clone(&retained),
            },
            resource_request,
        );
        let first_failure = match running.begin_graph_execution(
            execution,
            &graph,
            WorthQueryManagedGraphCallRequest::new(
                WorthQueryGraphProviderCallKind::Observe,
                "retain-first-admission",
            ),
        ) {
            Ok(_) => panic!("the hostile provider admitted after retaining the first admission"),
            Err(failure) => failure,
        };
        assert_eq!(
        first_failure.kind(),
        crate::domain_computation::WorthQueryDirectGraphExecutionStartFailureKind::ProviderStart
    );

        let foreign_failure = match first_failure.into_running().begin_graph_execution(
            execution,
            &graph,
            WorthQueryManagedGraphCallRequest::new(
                WorthQueryGraphProviderCallKind::Observe,
                "return-foreign-admission",
            ),
        ) {
            Ok(_) => panic!("an admission from another start arena entered the fresh start"),
            Err(failure) => failure,
        };
        assert_eq!(
        foreign_failure.kind(),
        crate::domain_computation::WorthQueryDirectGraphExecutionStartFailureKind::
            ProviderStartContractDenied
    );
    });
}

#[test]
fn ignored_second_execution_admission_denies_the_provider_start() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let (running, graph) = managed_graph_run_with_provider(
            WorthQueryOperationGraphAccess::Observe,
            MultipleAdmissionProvider,
            resource_request,
        );
        let failure = match running.begin_graph_execution(
            execution,
            &graph,
            WorthQueryManagedGraphCallRequest::new(
                WorthQueryGraphProviderCallKind::Observe,
                "multiple-admissions",
            ),
        ) {
            Ok(_) => panic!("ignoring a second-admission denial admitted the first execution"),
            Err(failure) => failure,
        };
        assert_eq!(
        failure.kind(),
        crate::domain_computation::WorthQueryDirectGraphExecutionStartFailureKind::
            ProviderStartContractDenied
    );
        assert!(
            failure.provider_execution_release().is_some(),
            "the denied execution must be explicitly released"
        );
    });
}

#[test]
fn rejection_after_admission_releases_the_runtime_owned_execution_explicitly() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let disposal_attempts = Arc::new(AtomicUsize::new(0));
        let destructor_attempts = Arc::new(AtomicUsize::new(0));
        let failure = post_admission_failure(
            execution,
            PostAdmissionBehavior::Reject,
            false,
            Arc::clone(&disposal_attempts),
            Arc::clone(&destructor_attempts),
            resource_request,
        );
        assert_eq!(
        failure.kind(),
        crate::domain_computation::WorthQueryDirectGraphExecutionStartFailureKind::ProviderStart
    );
        let release = failure
            .provider_execution_release()
            .expect("post-admission rejection must carry physical-release evidence");
        assert_eq!(
            release.disposal(),
            WorthQueryProviderExecutionDisposalDisposition::Completed
        );
        assert_eq!(
            release.destructor(),
            WorthQueryProviderExecutionDestructorDisposition::Completed
        );
        assert_eq!(disposal_attempts.load(Ordering::Acquire), 1);
        assert_eq!(destructor_attempts.load(Ordering::Acquire), 1);
        failure
            .into_running()
            .terminate_for_convergence(WorthQueryManagedRunTerminalKind::Failed)
            .cleanup()
            .expect("contained start rejection preserves cleanup authority");
    });
}

#[test]
fn panic_after_admission_contains_an_independent_destructor_panic() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let disposal_attempts = Arc::new(AtomicUsize::new(0));
        let destructor_attempts = Arc::new(AtomicUsize::new(0));
        let failure = post_admission_failure(
            execution,
            PostAdmissionBehavior::Panic,
            true,
            Arc::clone(&disposal_attempts),
            Arc::clone(&destructor_attempts),
            resource_request,
        );
        assert_eq!(
        failure.kind(),
        crate::domain_computation::WorthQueryDirectGraphExecutionStartFailureKind::
            ProviderStartReleaseRecoveryRequired
    );
        let release = failure
            .provider_execution_release()
            .expect("post-admission panic must carry physical-release evidence");
        assert_eq!(
            release.disposal(),
            WorthQueryProviderExecutionDisposalDisposition::Completed
        );
        assert_eq!(
            release.destructor(),
            WorthQueryProviderExecutionDestructorDisposition::Panicked
        );
        assert_eq!(disposal_attempts.load(Ordering::Acquire), 1);
        assert_eq!(destructor_attempts.load(Ordering::Acquire), 1);
        let cleanup = failure
            .into_running()
            .terminate_for_convergence(WorthQueryManagedRunTerminalKind::Failed)
            .cleanup()
            .expect("contained start panic preserves lower cleanup authority");
        assert_eq!(
            cleanup.inspection().disposition(),
            WorthQueryManagedRunCleanupDisposition::RecoveryRequired
        );
    });
}

fn post_admission_failure(
    execution: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,

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
    match running.begin_graph_execution(
        execution,
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

fn step_failure(
    denial: crate::domain_computation::WorthQueryGraphProviderStepDenial,
) -> WorthQueryGraphProviderFailure {
    WorthQueryGraphProviderFailure::new(denial.detail())
}
