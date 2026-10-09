use std::sync::{Arc, Mutex};

use worth_query::facade::domain;

use super::graph_read_material;

#[derive(Clone, Copy, Debug)]
pub(super) struct RemoteA;
#[derive(Clone, Copy, Debug)]
pub(super) struct RemoteB;
#[derive(Clone, Copy, Debug)]
pub(super) struct SharedCommit;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum FailAt {
    Observe,
    Project,
    Touch,
    Commit,
}

pub(super) struct SelectiveProvider {
    log: Arc<Mutex<Vec<&'static str>>>,
    fail_at: Option<FailAt>,
    expected_commit_roles: Option<Vec<&'static str>>,
    resource_support: domain::WorthQueryExecutionResourceSupport,
}

impl SelectiveProvider {
    pub(super) fn new(log: &Arc<Mutex<Vec<&'static str>>>, fail_at: Option<FailAt>) -> Self {
        Self {
            log: Arc::clone(log),
            fail_at,
            expected_commit_roles: None,
            resource_support: super::super::installed_operation_fixture::execution_resource_support(
            ),
        }
    }

    pub(super) fn new_with_support(
        log: &Arc<Mutex<Vec<&'static str>>>,
        resource_support: domain::WorthQueryExecutionResourceSupport,
    ) -> Self {
        Self {
            log: Arc::clone(log),
            fail_at: None,
            expected_commit_roles: None,
            resource_support,
        }
    }

    pub(super) fn partial_effect(
        log: &Arc<Mutex<Vec<&'static str>>>,
        fail_at: Option<FailAt>,
    ) -> Self {
        Self {
            log: Arc::clone(log),
            fail_at,
            expected_commit_roles: None,
            resource_support:
                super::super::installed_operation_fixture::partial_effect_execution_resource_support(
                ),
        }
    }

    pub(super) fn commit_with_support(
        log: &Arc<Mutex<Vec<&'static str>>>,
        expected_roles: Vec<&'static str>,
        resource_support: domain::WorthQueryExecutionResourceSupport,
    ) -> Self {
        Self {
            log: Arc::clone(log),
            fail_at: None,
            expected_commit_roles: Some(expected_roles),
            resource_support,
        }
    }

    pub(super) fn partial_effect_commit(
        log: &Arc<Mutex<Vec<&'static str>>>,
        fail_at: Option<FailAt>,
        expected_roles: Vec<&'static str>,
    ) -> Self {
        Self {
            log: Arc::clone(log),
            fail_at,
            expected_commit_roles: Some(expected_roles),
            resource_support:
                super::super::installed_operation_fixture::partial_effect_execution_resource_support(
                ),
        }
    }

    fn contact_label(
        &self,
        label: &'static str,
        kind: FailAt,
    ) -> Result<&'static str, domain::WorthQueryGraphProviderFailure> {
        self.log.lock().unwrap().push(label);
        if self.fail_at == Some(kind) {
            Err(domain::WorthQueryGraphProviderFailure::new(label))
        } else {
            Ok(label)
        }
    }

    fn assert_graph_call_resources(call: &domain::WorthQueryGraphProviderCall) {
        assert_eq!(call.execution_resources().strategy(), "fixture-bounded");
        assert_eq!(
            call.resource_envelope().cancellation_safe_point().as_str(),
            "fixture-chunk-boundary"
        );
    }
}

impl<G> domain::WorthQueryGraphParticipationProvider<G> for SelectiveProvider {
    type Execution = crate::suite::graph_provider_step::FixtureGraphProviderExecution;

    fn execution_resource_support(&self) -> domain::WorthQueryExecutionResourceSupport {
        self.resource_support.clone()
    }

    fn begin(
        &self,
        call: &domain::WorthQueryGraphProviderCall,
        start: &mut domain::WorthQueryGraphProviderExecutionStart,
    ) -> Result<
        domain::WorthQueryCooperativeGraphProviderExecution<Self::Execution>,
        domain::WorthQueryGraphProviderFailure,
    > {
        Self::assert_graph_call_resources(call);
        let execution = match call.kind() {
            domain::WorthQueryGraphProviderCallKind::Observe => {
                Self::Execution::read(self.contact_label("observe", FailAt::Observe)?)
            }
            domain::WorthQueryGraphProviderCallKind::Project => {
                self.log.lock().unwrap().push("project");
                if self.fail_at == Some(FailAt::Project) {
                    Self::Execution::failed("project")
                } else {
                    Self::Execution::projection(
                        "project",
                        graph_read_material("graph-provider-execution-projection"),
                    )
                }
            }
            domain::WorthQueryGraphProviderCallKind::TouchEffect => {
                Self::Execution::effect(self.contact_label("touch", FailAt::Touch)?)
            }
            domain::WorthQueryGraphProviderCallKind::CommitAdmission => {
                unreachable!("graph participation never receives commit admission")
            }
        };
        start
            .admit_cooperative_execution(|| execution)
            .map_err(|denial| domain::WorthQueryGraphProviderFailure::new(denial.detail()))
    }
}

impl domain::WorthQueryGraphCommitProvider<SharedCommit> for SelectiveProvider {
    fn execution_resource_support(&self) -> domain::WorthQueryExecutionResourceSupport {
        self.resource_support.clone()
    }

    fn admit_commit(
        &self,
        _execution: worth_execution::ExecutionRequest<'_, '_>,
        call: &domain::WorthQueryGraphCommitCall,
    ) -> Result<domain::WorthQueryGraphProviderReceipt, domain::WorthQueryGraphProviderFailure>
    {
        assert_eq!(call.execution_resources().strategy(), "fixture-bounded");
        assert_eq!(
            call.resource_envelope().cancellation_safe_point().as_str(),
            "fixture-chunk-boundary"
        );
        assert_eq!(
            call.graph_roles(),
            self.expected_commit_roles
                .as_deref()
                .expect("commit fixture declares the exact mutating role set")
        );
        Ok(call.completed(
            self.contact_label("commit", FailAt::Commit)?,
            crate::suite::provider_commit_admission_work_report(),
        ))
    }
}

pub(super) struct ReceiptOnlyProvider;

impl domain::WorthQueryGraphParticipationProvider<RemoteA> for ReceiptOnlyProvider {
    type Execution = crate::suite::graph_provider_step::FixtureGraphProviderExecution;

    fn execution_resource_support(&self) -> domain::WorthQueryExecutionResourceSupport {
        super::super::installed_operation_fixture::execution_resource_support()
    }

    fn begin(
        &self,
        call: &domain::WorthQueryGraphProviderCall,
        start: &mut domain::WorthQueryGraphProviderExecutionStart,
    ) -> Result<
        domain::WorthQueryCooperativeGraphProviderExecution<Self::Execution>,
        domain::WorthQueryGraphProviderFailure,
    > {
        let execution = match call.kind() {
            domain::WorthQueryGraphProviderCallKind::Observe => Self::Execution::read("observe"),
            domain::WorthQueryGraphProviderCallKind::Project => {
                Self::Execution::read("dishonest-projection-receipt")
            }
            domain::WorthQueryGraphProviderCallKind::TouchEffect => {
                Self::Execution::effect("touch")
            }
            domain::WorthQueryGraphProviderCallKind::CommitAdmission => {
                unreachable!("graph participation never receives commit admission")
            }
        };
        start
            .admit_cooperative_execution(|| execution)
            .map_err(|denial| domain::WorthQueryGraphProviderFailure::new(denial.detail()))
    }
}
