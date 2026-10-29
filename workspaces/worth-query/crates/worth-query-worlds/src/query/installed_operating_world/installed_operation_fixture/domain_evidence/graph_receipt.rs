pub use worth_query::facade::domain;

#[derive(Clone, Copy, Debug)]
pub struct EvidenceGraph;

pub struct EvidenceGraphProvider;

impl domain::WorthQueryGraphParticipationProvider<EvidenceGraph> for EvidenceGraphProvider {
    type Execution =
        crate::facade::query::suite::graph_provider_step::FixtureGraphProviderExecution;

    fn execution_resource_support(&self) -> domain::WorthQueryExecutionResourceSupport {
        super::super::execution_resource_support()
    }

    fn begin(
        &self,
        call: &domain::WorthQueryGraphProviderCall,
        start: &mut domain::WorthQueryGraphProviderExecutionStart,
    ) -> Result<
        domain::WorthQueryCooperativeGraphProviderExecution<Self::Execution>,
        domain::WorthQueryGraphProviderFailure,
    > {
        assert_eq!(
            call.kind(),
            domain::WorthQueryGraphProviderCallKind::Observe
        );
        start
            .admit_cooperative_execution(|| Self::Execution::read("evidence-graph-observe"))
            .map_err(|denial| domain::WorthQueryGraphProviderFailure::new(denial.detail()))
    }
}

pub fn evidence_graph_definition() -> domain::WorthQueryGraphParticipationDefinition<EvidenceGraph>
{
    domain::WorthQueryGraphParticipationDefinition::new(
        "evidence-graph",
        domain::WorthQueryGraphParticipationContract {
            observation: domain::WorthQueryGraphObservationPosture::Snapshot,
            projection: domain::WorthQueryGraphProjectionPosture::NotRequired,
            mutation: domain::WorthQueryGraphMutationPosture::NotRequired,
            identity: domain::WorthQueryGraphIdentityPosture::Opaque,
            locality: domain::WorthQueryGraphLocalityPosture::ExternalBoundary,
            budget: domain::WorthQueryGraphBudgetPosture::ExternalBoundary,
            commit: domain::WorthQueryGraphCommitPosture::ReadOnly,
            failure: domain::WorthQueryGraphFailureTopology::BoundaryFailure,
        },
    )
}
