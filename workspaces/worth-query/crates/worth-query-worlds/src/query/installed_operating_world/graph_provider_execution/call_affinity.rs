pub use std::sync::{Arc, Mutex};

pub use worth_query::facade::domain;

pub use super::{
    configured_runtime_for_package, federated_package, graph_read_material, read_definition,
    FederatedRead, GeometryDomain, ReadFamily, RemoteA, RemoteB,
};

pub struct RetainingProvider {
    pub retained: Arc<Mutex<Option<domain::WorthQueryGraphProviderCall>>>,
    pub projection_label: &'static str,
}

impl<G> domain::WorthQueryGraphParticipationProvider<G> for RetainingProvider {
    type Execution =
        crate::facade::query::suite::graph_provider_step::FixtureGraphProviderExecution;

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
        let mut retained = self.retained.lock().unwrap();
        retained.get_or_insert_with(|| call.clone());
        let execution = match call.kind() {
            domain::WorthQueryGraphProviderCallKind::Observe => Self::Execution::read("observe"),
            domain::WorthQueryGraphProviderCallKind::Project => Self::Execution::projection(
                self.projection_label,
                graph_read_material(self.projection_label),
            ),
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
