use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use worth_query::facade::domain;

use super::installed_operation_fixture::configured_runtime;

#[derive(Clone, Copy, Debug)]
struct RemoteGraph;

#[derive(Clone, Copy, Debug)]
struct OtherGraph;

#[derive(Clone, Copy, Debug)]
struct RemoteGraphLookalike;
#[derive(Clone, Copy, Debug)]
struct AtomicGraph;
#[derive(Clone, Copy, Debug)]
struct AtomicCommit;

struct CountingProvider {
    contacts: Arc<AtomicUsize>,
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
    fn new(contacts: &Arc<AtomicUsize>) -> Self {
        Self {
            contacts: Arc::clone(contacts),
        }
    }

    fn receipt_label<'a>(&self, kind: &'a str) -> &'a str {
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

#[test]
fn adapter_declaration_order_converges_without_provider_contact() {
    let direct_contacts = Arc::new(AtomicUsize::new(0));
    let direct = configured_runtime()
        .graph_participation(definition::<RemoteGraph>("remote"))
        .graph_participation_provider(RemoteGraph, CountingProvider::new(&direct_contacts))
        .graph_participation(definition::<OtherGraph>("other"))
        .graph_participation_provider(OtherGraph, CountingProvider::new(&direct_contacts))
        .workspace("graph-order-direct")
        .unwrap();
    let reversed_contacts = Arc::new(AtomicUsize::new(0));
    let reversed = configured_runtime()
        .graph_participation_provider(OtherGraph, CountingProvider::new(&reversed_contacts))
        .graph_participation(definition::<OtherGraph>("other"))
        .graph_participation_provider(RemoteGraph, CountingProvider::new(&reversed_contacts))
        .graph_participation(definition::<RemoteGraph>("remote"))
        .workspace("graph-order-reversed")
        .unwrap();
    let direct_adapter = direct.graph_participation(RemoteGraph).unwrap();
    let reversed_adapter = reversed.graph_participation(RemoteGraph).unwrap();
    assert_eq!(direct_adapter.role(), reversed_adapter.role());
    assert_eq!(direct_adapter.contract(), reversed_adapter.contract());
    assert_eq!(direct_contacts.load(Ordering::Relaxed), 0);
    assert_eq!(reversed_contacts.load(Ordering::Relaxed), 0);
}

#[test]
fn definition_provider_sets_must_close_exactly() {
    let contacts = Arc::new(AtomicUsize::new(0));
    assert!(configured_runtime()
        .graph_participation(definition::<RemoteGraph>("remote"))
        .workspace("graph-missing-provider")
        .is_err());
    assert!(configured_runtime()
        .graph_participation_provider(RemoteGraph, CountingProvider::new(&contacts))
        .workspace("graph-extra-provider")
        .is_err());
    assert_eq!(contacts.load(Ordering::Relaxed), 0);
}

#[test]
fn atomic_graphs_require_one_exact_referenced_commit_provider() {
    let contacts = Arc::new(AtomicUsize::new(0));
    assert!(configured_runtime()
        .graph_participation(atomic_definition::<AtomicGraph>("atomic"))
        .atomic_graph_participation_provider(
            AtomicGraph,
            CountingProvider::new(&contacts),
            AtomicCommit,
        )
        .workspace("graph-missing-commit-provider")
        .is_err());
    assert!(configured_runtime()
        .graph_commit_provider(AtomicCommit, CountingProvider::new(&contacts))
        .workspace("graph-extra-commit-provider")
        .is_err());
    assert_eq!(contacts.load(Ordering::Relaxed), 0);
}

#[test]
fn matching_role_on_another_marker_has_no_authority() {
    let contacts = Arc::new(AtomicUsize::new(0));
    let world = configured_runtime()
        .graph_participation(definition::<RemoteGraph>("remote"))
        .graph_participation_provider(RemoteGraph, CountingProvider::new(&contacts))
        .workspace("graph-marker-lookalike")
        .unwrap();
    let denial = match world.graph_participation(RemoteGraphLookalike) {
        Ok(_) => panic!("matching role on a foreign marker must not resolve"),
        Err(denial) => denial,
    };
    assert_eq!(
        denial.kind(),
        domain::WorthQueryGraphParticipationLookupDenialKind::NotInstalled
    );
    assert_eq!(denial.counters().indexed_lookups, 1);
    assert_eq!(denial.counters().provider_contacts, 0);
    assert_eq!(contacts.load(Ordering::Relaxed), 0);
}

fn definition<G>(role: &str) -> domain::WorthQueryGraphParticipationDefinition<G> {
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

fn atomic_definition<G>(role: &str) -> domain::WorthQueryGraphParticipationDefinition<G> {
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
