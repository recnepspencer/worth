//! Ordinary installation, bootstrap authority and authenticated native mutations.
use super::{operation::*, program::GraphProgram, schema::*};
use crate::document_retention_model::{host::host_limits, operator_identity::*};
use std::marker::PhantomData;
use worth_query_host::facade::{
    admission::authenticated_principal::WorthQueryRequestScope, application_entry::*,
    application_installation::*, declaration::application_program::*,
    declaration::application_schema::*,
    declaration::authentication::WorthQueryPrincipalMappingStatus, primary_graph::*,
    runtime::ExecutionAllocationPolicy,
};

pub struct World<L: LawPolicy> {
    pub application: WorthQueryProgramApplicationRuntime<GraphSchema<L>, GraphProgram<L>>,
    pub projection: WorthQueryApplicationInvariantProjectionAuthority<GraphSchema<L>>,
    pub nodes: u64,
}
pub fn install<L: LawPolicy>(nodes: u64, edges: &[(u64, u64)]) -> World<L> {
    assert!(nodes > 0);
    let schema = GraphSchema::<L>::declaration().unwrap();
    let program = ApplicationProgramAuthoring::<GraphSchema<L>, GraphProgram<L>>::begin()
        .with_schema(&schema)
        .validated_program()
        .unwrap();
    install_program(program, nodes, edges).unwrap()
}
pub fn install_program<L: LawPolicy>(
    program: ValidatedApplicationProgram<GraphSchema<L>, GraphProgram<L>>,
    nodes: u64,
    edges: &[(u64, u64)],
) -> Result<World<L>, WorthQueryInMemoryApplicationDenial> {
    let schema = GraphSchema::<L>::declaration().unwrap();
    let mut projection = None;
    let application = in_memory_program(
        program,
        schema,
        ((),),
        ApplicationBuildIdentity::parse("11111111-1111-1111-1111-111111111111").unwrap(),
        host_limits(),
        |_, graph, installed| {
            projection = Some(graph.retain_invariant_projection_authority());
            let binding = installed
                .principal_binding(GraphPrincipal::reference())
                .unwrap();
            graph.bind_principal(
                &binding,
                WorthQueryApplicationPrincipalKey::new("graph-operator").unwrap(),
                1_u64,
                operator_identity(),
                WorthQueryPrincipalMappingStatus::Enabled,
            )?;
            for node in 0..nodes {
                graph.bind_entity(
                    WorthQueryApplicationEntitySeed::new(Node::reference(), key::<L>(node))
                        .field(NodeKey::reference(), node),
                )?;
            }
            for (index, &(source, target)) in edges.iter().enumerate() {
                graph.bind_relation(WorthQueryApplicationRelationSeed::new(
                    Precedes::reference(),
                    format!("seed-{index}"),
                    key::<L>(source),
                    key::<L>(target),
                ))?;
            }
            Ok(())
        },
    )?;
    Ok(World {
        application,
        projection: projection.unwrap(),
        nodes,
    })
}
fn key<L: LawPolicy>(node: u64) -> WorthQueryApplicationEntityKey<GraphSchema<L>, Node> {
    WorthQueryApplicationEntityKey::new(format!("node-{node}")).unwrap()
}
impl<L: LawPolicy> World<L> {
    pub fn edit(
        &self,
        serial: u64,
        edges: &[(u64, u64)],
    ) -> WorthQueryApplicationMutationOutcome<GraphDenial, Edited> {
        let scope = request_scope();
        self.edit_in_scope(serial, edges, &scope).unwrap()
    }
    // compiled-example: begin
    pub fn edit_in_scope(
        &self,
        serial: u64,
        edges: &[(u64, u64)],
        scope: &WorthQueryRequestScope,
    ) -> Result<
        WorthQueryApplicationMutationOutcome<GraphDenial, Edited>,
        WorthQueryApplicationRequestMutationDenial,
    > {
        let principal =
            authenticate_operator(self.application.installed_schema(), &request_scope());
        self.application
            .runtime()
            .request(&principal, scope)
            .on_branch(self.application.current_world())
            .advanced()
            .mutate(Edit::<L> {
                input: EdgeEdit {
                    serial,
                    edges: edges.to_vec(),
                },
                marker: PhantomData,
            })
            .without_source()
            .idempotency(&serial)
            .execute_in_program(
                &self.application,
                ExecutionAllocationPolicy::SystemAllocation,
            )
    }
    // compiled-example: end
    pub fn close(mut self) {
        assert!(self.application.close_live_delivery().owner_terminal());
        self.application.close_conditional_runtime().unwrap();
    }
}
