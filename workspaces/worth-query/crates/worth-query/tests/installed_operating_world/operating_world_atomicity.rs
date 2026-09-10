use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use worth_query::facade::domain;

use super::installed_operation_fixture::{
    mixed_mutation_workflow_runtime, GeometryDomain, MutationFamily, WorkflowMutation,
};

#[derive(Clone, Copy)]
struct RemoteGraph;
#[derive(Clone, Copy)]
struct SeparateCommit;

struct UncontactedProvider(Arc<AtomicUsize>);

impl domain::WorthQueryGraphParticipationProvider<RemoteGraph> for UncontactedProvider {
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
        self.0.fetch_add(1, Ordering::Relaxed);
        let execution = match call.kind() {
            domain::WorthQueryGraphProviderCallKind::Observe => Self::Execution::read("observe"),
            domain::WorthQueryGraphProviderCallKind::Project => Self::Execution::read("project"),
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

impl domain::WorthQueryGraphCommitProvider<SeparateCommit> for UncontactedProvider {
    fn execution_resource_support(&self) -> domain::WorthQueryExecutionResourceSupport {
        super::installed_operation_fixture::execution_resource_support()
    }

    fn admit_commit(
        &self,
        call: &domain::WorthQueryGraphCommitCall,
    ) -> Result<domain::WorthQueryGraphProviderReceipt, domain::WorthQueryGraphProviderFailure>
    {
        self.0.fetch_add(1, Ordering::Relaxed);
        Ok(call.completed("commit", super::provider_commit_admission_work_report()))
    }
}

#[test]
fn primary_and_separate_mutation_requires_declared_compensation() {
    let contacts = Arc::new(AtomicUsize::new(0));
    let uncompensated = runtime(Arc::clone(&contacts), "mixed-uncompensated");
    let installed = uncompensated.domain(GeometryDomain).unwrap();
    let denial = match uncompensated
        .prepare_mutation_operating_world(uncompensated.current_world())
        .unwrap()
        .family(MutationFamily)
        .bind(&installed, WorkflowMutation)
    {
        Ok(_) => panic!("separate commit authority cannot cover primary graph mutation"),
        Err(denial) => denial,
    };
    assert_eq!(
        denial.kind(),
        domain::WorthQueryOperationBindingDenialKind::CompensationUndeclared
    );
    assert_eq!(contacts.load(Ordering::Relaxed), 0);
}

fn runtime(
    contacts: Arc<AtomicUsize>,
    name: &str,
) -> worth_query::facade::runtime::WorthQueryWorkspace {
    mixed_mutation_workflow_runtime::<RemoteGraph>()
        .graph_participation(atomic_definition())
        .atomic_graph_participation_provider(
            RemoteGraph,
            UncontactedProvider(Arc::clone(&contacts)),
            SeparateCommit,
        )
        .graph_commit_provider(SeparateCommit, UncontactedProvider(contacts))
        .workspace(name)
        .unwrap()
}

fn atomic_definition() -> domain::WorthQueryGraphParticipationDefinition<RemoteGraph> {
    domain::WorthQueryGraphParticipationDefinition::new(
        "remote-a",
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
