//! Ordinary issued/context creation, with an optional edge in the same proposal.
pub use super::schema::*;
pub use std::marker::PhantomData;
pub use worth_query_decl::facade::{
    worth_query_operation_creates, worth_query_operation_links, worth_query_operation_reads,
    worth_query_operation_writes,
};
pub use worth_query_host::facade::{
    declaration::application_operation::*, primary_graph::*, worth_query_structured_value_binding,
};

#[derive(Clone, Debug, serde::Serialize)]
pub struct CreateNodeInput {
    pub key: u64,
    pub context: u64,
    pub in_context: bool,
    pub link: bool,
    pub backlink: bool,
    pub self_cycle: bool,
}
worth_query_structured_value_binding!(pub CreateNodeInputBinding for CreateNodeInput { identity: "worth.query.relation-laws.create-node.v3" });
pub use super::operation::{Edited, EditedBinding, GraphDenial, GraphDenialBinding};

pub struct CreateNode;
impl<S: GraphSchemaBinding> ApplicationOperationMarkerIdentity<S> for CreateNode {
    type InputBinding = CreateNodeInputBinding;
    const IDENTIFIER: &'static str = "CreateNode";
}
impl CreateNode {
    pub fn reference<S: GraphSchemaBinding>() -> ApplicationOperationRef<S, Self, CreateNodeInput> {
        ApplicationOperationRef::from_declaration()
    }
}

worth_query_operation_links!(CreateNode => [Precedes]);
worth_query_operation_creates!(CreateNode => [Node]);

pub struct CreateBinding<L: LawPolicy>(pub PhantomData<fn() -> L>);
pub type NodeScope<L> = ApplicationMutationFieldScope<GraphKey<GraphSchema<L>>>;
impl<L: LawPolicy> ApplicationMutationBinding<GraphSchema<L>> for CreateBinding<L> {
    type Input = CreateNodeInput;
    type InputBinding = CreateNodeInputBinding;
    type Result = Edited;
    type ResultBinding = EditedBinding;
    type IdempotencyKey = u64;
    type Operation = CreateNode;
    type Decision = WorthQueryInvariantMutationTarget<GraphSchema<L>, GraphNode<GraphSchema<L>>>;
    type Denial = GraphDenial;
    type DenialBinding = GraphDenialBinding;
    type Output = NoApplicationMutationOutputs;
    type ScopeBinding = NodeScope<L>;
    type PrincipalBinding = GraphPrincipal;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type SourceExpectation = NoApplicationMutationSource;
    const IDENTITY: &'static str = "worth.query.relation-laws.create-binding.v1";
    const HANDLER_IDENTITY: &'static str = "worth.query.relation-laws.create-handler.v3";
    const IDEMPOTENCY_IDENTITY: &'static str = "worth.query.relation-laws.create-command.v1";
    fn workflow_execution_requirements() -> ApplicationWorkflowExecutionRequirements {
        ApplicationWorkflowExecutionRequirements::not_applicable()
    }
    fn scope_field() -> GraphKey<GraphSchema<L>> {
        node_key::<GraphSchema<L>>()
    }
    fn principal_binding() -> ApplicationPrincipalBindingRef<
        GraphSchema<L>,
        GraphPrincipal,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        GraphPrincipal::reference()
    }
}
#[derive(Clone)]
pub struct Create<L: LawPolicy> {
    pub input: CreateNodeInput,
    pub marker: PhantomData<fn() -> L>,
}
impl<L: LawPolicy> ApplicationMutationIntent<GraphSchema<L>> for Create<L> {
    type Binding = CreateBinding<L>;
    fn input(&self) -> &CreateNodeInput {
        &self.input
    }
    fn scope_binding(&self) -> NodeScope<L> {
        NodeScope::<L>::new(node_key::<GraphSchema<L>>(), self.input.context)
    }
}

pub struct CreateHandler;
impl<L: LawPolicy> OperationHandler<GraphSchema<L>, CreateBinding<L>> for CreateHandler {
    fn decide(
        &self,
        input: &CreateNodeInput,
        reader: &mut DecisionReader<'_, '_, '_, GraphSchema<L>, CreateBinding<L>>,
    ) -> HandlerResult<
        WorthQueryInvariantMutationTarget<GraphSchema<L>, GraphNode<GraphSchema<L>>>,
        GraphDenial,
    > {
        let source = match reader.resolve_entity(node_key::<GraphSchema<L>>(), input.context) {
            Ok(value) => value,
            Err(error) => return HandlerResult::ExecutionDenied(error),
        };
        match reader.mutation_target(&source) {
            Ok(value) => HandlerResult::Completed(value),
            Err(error) => HandlerResult::ExecutionDenied(error),
        }
    }
    fn build_candidate(
        &self,
        input: &CreateNodeInput,
        source: WorthQueryInvariantMutationTarget<GraphSchema<L>, GraphNode<GraphSchema<L>>>,
        writer: &mut CandidateWriter<'_, GraphSchema<L>, CreateBinding<L>>,
    ) -> HandlerResult<Edited, GraphDenial> {
        let build = |writer: &mut CandidateWriter<'_, GraphSchema<L>, CreateBinding<L>>| {
            let source = writer.projected_entity(&source)?;
            let key = WorthQueryApplicationEntityKey::new(format!("node-{}", input.key)).unwrap();
            let target = if input.in_context {
                writer.create_entity_in_context(&source, node::<GraphSchema<L>>(), key)?
            } else {
                writer.create_entity(node::<GraphSchema<L>>(), key)?
            };
            writer.initialize_field(&target, node_key::<GraphSchema<L>>(), input.key)?;
            if input.self_cycle {
                writer.link(
                    precedes::<GraphSchema<L>>(),
                    format!("self-edge-{}", input.key),
                    &target,
                    &target,
                )?;
            }
            if input.link {
                writer.link(
                    precedes::<GraphSchema<L>>(),
                    format!("created-edge-{}", input.key),
                    &source,
                    &target,
                )?;
            }
            if input.backlink {
                writer.link(
                    precedes::<GraphSchema<L>>(),
                    format!("return-edge-{}", input.key),
                    &target,
                    &source,
                )?;
            }
            Ok::<(), WorthQueryApplicationAttemptDenial>(())
        };
        match build(writer) {
            Ok(()) => HandlerResult::Completed(Edited),
            Err(error) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error)),
        }
    }
}
