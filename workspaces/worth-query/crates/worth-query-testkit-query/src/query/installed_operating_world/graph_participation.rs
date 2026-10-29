pub use std::sync::atomic::{AtomicUsize, Ordering};
pub use std::sync::Arc;

pub use worth_query::facade::domain;

pub use super::installed_operation_fixture::configured_runtime;

#[derive(Clone, Copy, Debug)]
pub struct RemoteGraph;

#[derive(Clone, Copy, Debug)]
pub struct OtherGraph;

#[derive(Clone, Copy, Debug)]
pub struct RemoteGraphLookalike;
#[derive(Clone, Copy, Debug)]
pub struct AtomicGraph;
#[derive(Clone, Copy, Debug)]
pub struct AtomicCommit;

pub struct CountingProvider {
    pub contacts: Arc<AtomicUsize>,
}

impl<G> domain::WorthQueryGraphParticipationProvider<G> for CountingProvider {
    type Execution = super::graph_provider_step::FixtureGraphProviderExecution;

    fn execution_resource_support(&self) -> domain::WorthQueryExecutionResourceSupport {
        super::installed_operation_fixture::execution_resource_support()
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
            domain::WorthQueryGraphProviderCallKind::Observe => {
                Self::Execution::read(self.receipt_label("observe"))
            }
            domain::WorthQueryGraphProviderCallKind::Project => {
                Self::Execution::read(self.receipt_label("project"))
            }
            domain::WorthQueryGraphProviderCallKind::TouchEffect => {
                Self::Execution::effect(self.receipt_label("touch-effect"))
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

impl CountingProvider {
    pub fn new(contacts: &Arc<AtomicUsize>) -> Self {
        Self {
            contacts: Arc::clone(contacts),
        }
    }

    pub fn receipt_label<'a>(&self, kind: &'a str) -> &'a str {
        self.contacts.fetch_add(1, Ordering::Relaxed);
        kind
    }
}

impl domain::WorthQueryGraphCommitProvider<AtomicCommit> for CountingProvider {
    fn execution_resource_support(&self) -> domain::WorthQueryExecutionResourceSupport {
        super::installed_operation_fixture::execution_resource_support()
    }

    fn admit_commit(
        &self,
        _execution: worth_execution::ExecutionRequest<'_, '_>,
        call: &domain::WorthQueryGraphCommitCall,
    ) -> Result<domain::WorthQueryGraphProviderReceipt, domain::WorthQueryGraphProviderFailure>
    {
        Ok(call.completed(
            self.receipt_label("commit"),
            super::provider_commit_admission_work_report(),
        ))
    }
}

pub fn definition<G>(role: &str) -> domain::WorthQueryGraphParticipationDefinition<G> {
    domain::WorthQueryGraphParticipationDefinition::new(
        role,
        domain::WorthQueryGraphParticipationContract {
            observation: domain::WorthQueryGraphObservationPosture::Snapshot,
            projection: domain::WorthQueryGraphProjectionPosture::NativeProjection,
            mutation: domain::WorthQueryGraphMutationPosture::NotRequired,
            identity: domain::WorthQueryGraphIdentityPosture::Opaque,
            locality: domain::WorthQueryGraphLocalityPosture::ExternalBoundary,
            budget: domain::WorthQueryGraphBudgetPosture::ExternalBoundary,
            commit: domain::WorthQueryGraphCommitPosture::ReadOnly,
            failure: domain::WorthQueryGraphFailureTopology::BoundaryFailure,
        },
    )
}

pub fn atomic_definition<G>(role: &str) -> domain::WorthQueryGraphParticipationDefinition<G> {
    domain::WorthQueryGraphParticipationDefinition::new(
        role,
        domain::WorthQueryGraphParticipationContract {
            observation: domain::WorthQueryGraphObservationPosture::Snapshot,
            projection: domain::WorthQueryGraphProjectionPosture::NativeProjection,
            mutation: domain::WorthQueryGraphMutationPosture::TouchAndEffect,
            identity: domain::WorthQueryGraphIdentityPosture::Opaque,
            locality: domain::WorthQueryGraphLocalityPosture::ExternalBoundary,
            budget: domain::WorthQueryGraphBudgetPosture::ExternalBoundary,
            commit: domain::WorthQueryGraphCommitPosture::AtomicAuthorityRequired,
            failure: domain::WorthQueryGraphFailureTopology::BoundaryFailure,
        },
    )
}
