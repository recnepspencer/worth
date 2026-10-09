use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use worth_query::facade::domain;

use super::super::graph_read_material::graph_read_material;

#[derive(Clone, Copy, Debug)]
pub(super) struct RemoteA;
#[derive(Clone, Copy, Debug)]
pub(super) struct RemoteB;
#[derive(Clone, Copy, Debug)]
pub(super) struct RemoteALookalike;
#[derive(Clone, Copy, Debug)]
pub(super) struct SharedCommit;
#[derive(Clone, Copy, Debug)]
pub(super) struct OtherCommit;

pub(super) struct Provider {
    contacts: Arc<AtomicUsize>,
    resource_support: domain::WorthQueryExecutionResourceSupport,
}

impl Provider {
    pub(super) fn effect_free(contacts: Arc<AtomicUsize>) -> Self {
        Self {
            contacts,
            resource_support: super::super::installed_operation_fixture::execution_resource_support(
            ),
        }
    }

    pub(super) fn partial_effects(contacts: Arc<AtomicUsize>) -> Self {
        Self {
            contacts,
            resource_support:
                super::super::installed_operation_fixture::partial_effect_execution_resource_support(
                ),
        }
    }

    fn receipt_label(&self) -> &'static str {
        self.contacts.fetch_add(1, Ordering::Relaxed);
        "provider"
    }
}

impl<G> domain::WorthQueryGraphParticipationProvider<G> for Provider {
    type Execution = super::super::graph_provider_step::FixtureGraphProviderExecution;

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
        let execution = match call.kind() {
            domain::WorthQueryGraphProviderCallKind::Observe => {
                Self::Execution::read(self.receipt_label())
            }
            domain::WorthQueryGraphProviderCallKind::Project => {
                self.contacts.fetch_add(1, Ordering::Relaxed);
                Self::Execution::projection(
                    "provider-projection",
                    graph_read_material("operating-world-graph-projection"),
                )
            }
            domain::WorthQueryGraphProviderCallKind::TouchEffect => {
                Self::Execution::effect(self.receipt_label())
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

impl<C> domain::WorthQueryGraphCommitProvider<C> for Provider {
    fn execution_resource_support(&self) -> domain::WorthQueryExecutionResourceSupport {
        self.resource_support.clone()
    }

    fn admit_commit(
        &self,
        _execution: worth_execution::ExecutionRequest<'_, '_>,
        call: &domain::WorthQueryGraphCommitCall,
    ) -> Result<domain::WorthQueryGraphProviderReceipt, domain::WorthQueryGraphProviderFailure>
    {
        if call.graph_roles() != ["remote-a", "remote-b"] {
            return Err(domain::WorthQueryGraphProviderFailure::new(
                "commit provider did not receive the complete atomic graph group",
            ));
        }
        Ok(call.completed(
            self.receipt_label(),
            super::super::provider_commit_admission_work_report(),
        ))
    }
}
