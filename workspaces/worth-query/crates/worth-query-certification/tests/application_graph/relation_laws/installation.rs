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
    pub leaves: u64,
    creation_attempt: u64,
}
pub fn install<L: LawPolicy>(nodes: u64, edges: &[(u64, u64)]) -> World<L> {
    install_with_feeds(nodes, edges, 0, &[])
}
pub fn install_with_feeds<L: LawPolicy>(
    nodes: u64,
    edges: &[(u64, u64)],
    leaves: u64,
    feeds: &[(u64, u64)],
) -> World<L> {
    assert!(nodes > 0);
    let schema = GraphSchema::<L>::declaration().unwrap();
    let program = ApplicationProgramAuthoring::<GraphSchema<L>, GraphProgram<L>>::begin()
        .with_schema(&schema)
        .validated_program()
        .unwrap();
    install_program(program, nodes, edges, leaves, feeds).unwrap()
}
pub fn install_program<L: LawPolicy>(
    program: ValidatedApplicationProgram<GraphSchema<L>, GraphProgram<L>>,
    nodes: u64,
    edges: &[(u64, u64)],
    leaves: u64,
    feeds: &[(u64, u64)],
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
            for leaf in 0..leaves {
                graph.bind_entity(
                    WorthQueryApplicationEntitySeed::new(Leaf::reference(), leaf_key::<L>(leaf))
                        .field(LeafKey::reference(), leaf),
                )?;
            }
            for (index, &(source, target)) in feeds.iter().enumerate() {
                graph.bind_relation(WorthQueryApplicationRelationSeed::new(
                    Feeds::reference(),
                    format!("feed-seed-{index}"),
                    key::<L>(source),
                    leaf_key::<L>(target),
                ))?;
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
        creation_attempt: 0,
        leaves,
    })
}
fn key<L: LawPolicy>(node: u64) -> WorthQueryApplicationEntityKey<GraphSchema<L>, Node> {
    WorthQueryApplicationEntityKey::new(format!("node-{node}")).unwrap()
}
fn leaf_key<L: LawPolicy>(leaf: u64) -> WorthQueryApplicationEntityKey<GraphSchema<L>, Leaf> {
    WorthQueryApplicationEntityKey::new(format!("leaf-{leaf}")).unwrap()
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
    pub fn edit_in_scope(
        &self,
        serial: u64,
        edges: &[(u64, u64)],
        scope: &WorthQueryRequestScope,
    ) -> Result<
        WorthQueryApplicationMutationOutcome<GraphDenial, Edited>,
        WorthQueryApplicationRequestMutationDenial,
    > {
        self.apply_edit_in_scope(
            EdgeEdit {
                serial,
                edges: edges.to_vec(),
                removals: Vec::new(),
            },
            scope,
        )
    }
    pub fn replace_edges(
        &self,
        serial: u64,
        edges: &[(u64, u64)],
        removals: &[(u64, u64)],
    ) -> WorthQueryApplicationMutationOutcome<GraphDenial, Edited> {
        self.apply_edit_in_scope(
            EdgeEdit {
                serial,
                edges: edges.to_vec(),
                removals: removals.to_vec(),
            },
            &request_scope(),
        )
        .unwrap()
    }
    pub fn create_node(
        &mut self,
        context: u64,
        in_context: bool,
        link: bool,
    ) -> WorthQueryApplicationMutationOutcome<GraphDenial, Edited> {
        self.create_with(super::creation::CreateNodeInput {
            key: self.nodes,
            context,
            in_context,
            link,
            backlink: false,
            self_cycle: false,
        })
    }
    pub fn create_with(
        &mut self,
        input: super::creation::CreateNodeInput,
    ) -> WorthQueryApplicationMutationOutcome<GraphDenial, Edited> {
        use super::creation::Create;
        assert_eq!(input.key, self.nodes);
        let scope = request_scope();
        let principal = authenticate_operator(self.application.installed_schema(), &scope);
        let command = self.creation_attempt;
        self.creation_attempt += 1;
        let result = self
            .application
            .runtime()
            .request(&principal, &scope)
            .on_branch(self.application.current_world())
            .advanced()
            .mutate(Create::<L> {
                input,
                marker: PhantomData,
            })
            .without_source()
            .idempotency(&command)
            .execute_in_program(
                &self.application,
                ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap();
        if matches!(
            result,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ) {
            self.nodes += 1;
        }
        result
    }
    pub fn edit_feeds(
        &mut self,
        serial: u64,
        context: u64,
        change: super::feeding::FeedsChange,
    ) -> WorthQueryApplicationMutationOutcome<GraphDenial, Edited> {
        use super::feeding::{FeedsChange, FeedsEdit, FeedsEditInput};
        let replacement = match &change {
            FeedsChange::Replace { first_leaf, count } => {
                assert_eq!(*first_leaf, self.leaves);
                Some(u64::from(*count))
            }
            _ => None,
        };
        let scope = request_scope();
        let principal = authenticate_operator(self.application.installed_schema(), &scope);
        let result = self
            .application
            .runtime()
            .request(&principal, &scope)
            .on_branch(self.application.current_world())
            .advanced()
            .mutate(FeedsEdit::<L> {
                input: FeedsEditInput {
                    serial,
                    context,
                    change,
                },
                marker: PhantomData,
            })
            .without_source()
            .idempotency(&serial)
            .execute_in_program(
                &self.application,
                ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap();
        if matches!(
            result,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ) {
            self.leaves += replacement.unwrap_or(0);
        }
        result
    }
    // compiled-example: begin
    pub fn apply_edit_in_scope(
        &self,
        input: EdgeEdit,
        scope: &WorthQueryRequestScope,
    ) -> Result<
        WorthQueryApplicationMutationOutcome<GraphDenial, Edited>,
        WorthQueryApplicationRequestMutationDenial,
    > {
        let serial = input.serial;
        let principal =
            authenticate_operator(self.application.installed_schema(), &request_scope());
        self.application
            .runtime()
            .request(&principal, scope)
            .on_branch(self.application.current_world())
            .advanced()
            .mutate(Edit::<L> {
                input,
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
