pub use crate::domain_computation::managed_run::tests::provider_execution_admission::post_admission_failure::post_admission_failure;
pub use std::sync::atomic::AtomicUsize;
pub use std::sync::atomic::Ordering;
pub use std::sync::Arc;
pub use std::sync::Mutex;

pub use crate::domain_computation::managed_run::tests::*;
pub use worth_query_execution::facade::domain_computation::WorthQueryProviderExecutionDestructorDisposition;
pub use worth_query_execution::facade::domain_computation::WorthQueryProviderExecutionDisposalDisposition;

pub struct AdmissionExecution;

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

pub struct ForeignAdmissionProvider {
    pub retained:
        Arc<Mutex<Option<WorthQueryCooperativeGraphProviderExecution<AdmissionExecution>>>>,
}

impl WorthQueryGraphParticipationProvider<ManagedGraph> for ForeignAdmissionProvider {
    type Execution = AdmissionExecution;

    fn execution_resource_support(
        &self,
    ) -> worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupport {
        worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::provider_session::tests::execution_resource_support(
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

pub struct MultipleAdmissionProvider;

#[derive(Clone, Copy)]
pub enum PostAdmissionBehavior {
    Reject,
    Panic,
}

pub struct PostAdmissionProvider {
    pub behavior: PostAdmissionBehavior,
    pub disposal_attempts: Arc<AtomicUsize>,
    pub destructor_attempts: Arc<AtomicUsize>,
    pub destructor_panics: bool,
}

pub struct PostAdmissionExecution {
    pub disposal_attempts: Arc<AtomicUsize>,
    pub destructor_attempts: Arc<AtomicUsize>,
    pub destructor_panics: bool,
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
        worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::provider_session::tests::execution_resource_support(
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
        worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::provider_session::tests::execution_resource_support(
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
    worth_query_execution::domain_computation::primary_graph::application_contribution::request_execution::advancement::with_test_advancement(|active_phase| {
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
        let installed_running_phase = execution.phase_for_direct(&running);

        let first_failure = match running.begin_graph_execution(
            &installed_running_phase,
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
        worth_query_execution::facade::domain_computation::WorthQueryDirectGraphExecutionStartFailureKind::ProviderStart
    );

        let foreign_failure = match first_failure.into_running().begin_graph_execution(
            &installed_running_phase,
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
        worth_query_execution::facade::domain_computation::WorthQueryDirectGraphExecutionStartFailureKind::ProviderStartContractDenied
    );
    });
}

#[test]
fn ignored_second_execution_admission_denies_the_provider_start() {
    worth_query_execution::domain_computation::primary_graph::application_contribution::request_execution::advancement::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let (running, graph) = managed_graph_run_with_provider(
            WorthQueryOperationGraphAccess::Observe,
            MultipleAdmissionProvider,
            resource_request,
        );
        let installed_running_phase = execution.phase_for_direct(&running);

        let failure = match running.begin_graph_execution(
            &installed_running_phase,
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
        worth_query_execution::facade::domain_computation::WorthQueryDirectGraphExecutionStartFailureKind::ProviderStartContractDenied
    );
        assert!(
            failure.provider_execution_release().is_some(),
            "the denied execution must be explicitly released"
        );
    });
}

#[test]
fn rejection_after_admission_releases_the_runtime_owned_execution_explicitly() {
    worth_query_execution::domain_computation::primary_graph::application_contribution::request_execution::advancement::with_test_advancement(|active_phase| {
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
        worth_query_execution::facade::domain_computation::WorthQueryDirectGraphExecutionStartFailureKind::ProviderStart
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
    worth_query_execution::domain_computation::primary_graph::application_contribution::request_execution::advancement::with_test_advancement(|active_phase| {
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
        worth_query_execution::facade::domain_computation::WorthQueryDirectGraphExecutionStartFailureKind::ProviderStartReleaseRecoveryRequired
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

pub fn step_failure(
    denial: worth_query_execution::facade::domain_computation::WorthQueryGraphProviderStepDenial,
) -> WorthQueryGraphProviderFailure {
    WorthQueryGraphProviderFailure::new(denial.detail())
}

#[path = "post_admission_failure/mod.rs"]
pub mod post_admission_failure;
