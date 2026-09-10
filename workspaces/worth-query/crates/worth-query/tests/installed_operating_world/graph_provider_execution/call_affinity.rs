use std::sync::{Arc, Mutex};

use worth_query::facade::domain;

use super::{
    configured_runtime_for_package, federated_package, graph_read_material, read_definition,
    FederatedRead, GeometryDomain, ReadFamily, RemoteA, RemoteB,
};

struct RetainingProvider {
    retained: Arc<Mutex<Option<domain::WorthQueryGraphProviderCall>>>,
    projection_label: &'static str,
}

impl<G> domain::WorthQueryGraphParticipationProvider<G> for RetainingProvider {
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

#[test]
fn retained_call_cannot_replace_the_current_sealed_step() {
    let retained = Arc::new(Mutex::new(None));
    let mut workspace = configured_runtime_for_package(federated_package::<RemoteA, RemoteB>())
        .graph_participation(read_definition::<RemoteA>(
            "remote-a",
            domain::WorthQueryGraphProjectionPosture::NativeProjection,
        ))
        .graph_participation_provider(
            RemoteA,
            RetainingProvider {
                retained: Arc::clone(&retained),
                projection_label: "remote-a-projection",
            },
        )
        .graph_participation(read_definition::<RemoteB>(
            "remote-b",
            domain::WorthQueryGraphProjectionPosture::NativeProjection,
        ))
        .graph_participation_provider(
            RemoteB,
            RetainingProvider {
                retained: Arc::clone(&retained),
                projection_label: "remote-b-projection",
            },
        )
        .workspace("graph-provider-retained-call")
        .unwrap();
    let installed = workspace.domain(GeometryDomain).unwrap();

    for _ in 0..2 {
        workspace
            .observe_operating_world(workspace.current_world())
            .unwrap()
            .family(ReadFamily)
            .bind(&installed, FederatedRead)
            .unwrap()
            .admit_execution_resources(
                (),
                crate::suite::installed_operation_fixture::execution_resource_request(),
                &workspace,
            )
            .unwrap()
            .execute(&mut workspace)
            .unwrap();
    }

    let retained = retained
        .lock()
        .unwrap()
        .clone()
        .expect("provider retained the first descriptive call");
    assert_eq!(retained.graph_role(), "remote-a");
}
