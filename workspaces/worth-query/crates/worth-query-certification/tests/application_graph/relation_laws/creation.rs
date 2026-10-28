//! Ordinary issued/context creation, with an optional edge in the same proposal.
use super::schema::*;
use std::marker::PhantomData;
use worth_query_decl::facade::{
    worth_query_operation_creates, worth_query_operation_links, worth_query_operation_reads,
    worth_query_operation_writes,
};
use worth_query_host::facade::{
    declaration::{application_operation::*, application_schema::*},
    primary_graph::*,
    worth_query_structured_value_binding,
};

#[derive(Clone, Debug, serde::Serialize)]
pub struct CreateNodeInput {
    pub key: u64,
    pub context: u64,
    pub in_context: bool,
    pub link: bool,
    pub backlink: bool,
}
worth_query_structured_value_binding!(pub CreateNodeInputBinding for CreateNodeInput { identity: "worth.query.relation-laws.create-node.v2" });
use super::operation::{Edited, EditedBinding, GraphDenial, GraphDenialBinding};

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
worth_query_operation_reads!(CreateNode => [NodeKey]);
worth_query_operation_links!(CreateNode => [Precedes]);
worth_query_operation_creates!(CreateNode => [Node]);
worth_query_operation_writes!(CreateNode => [NodeKey]);

pub struct CreateBinding<L: LawPolicy>(PhantomData<fn() -> L>);
type NodeScope<L> = ApplicationMutationFieldScope<
    GraphSchema<L>,
    Node,
    NodeFacts,
    NodeKey,
    u64,
    ReadOnly,
    NoApplicationUnit,
>;
impl<L: LawPolicy> ApplicationMutationBinding<GraphSchema<L>> for CreateBinding<L> {
    type Input = CreateNodeInput;
    type InputBinding = CreateNodeInputBinding;
    type Result = Edited;
    type ResultBinding = EditedBinding;
    type IdempotencyKey = u64;
    type Operation = CreateNode;
    type Decision = WorthQueryInvariantMutationTarget<GraphSchema<L>, Node>;
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
    const HANDLER_IDENTITY: &'static str = "worth.query.relation-laws.create-handler.v2";
    const IDEMPOTENCY_IDENTITY: &'static str = "worth.query.relation-laws.create-command.v1";
    fn workflow_execution_requirements() -> ApplicationWorkflowExecutionRequirements {
        ApplicationWorkflowExecutionRequirements::not_applicable()
    }
    fn scope_field() -> ApplicationFieldRef<
        GraphSchema<L>,
        Node,
        NodeFacts,
        NodeKey,
        u64,
        ReadOnly,
        EqualityPredicate,
        NoApplicationUnit,
    > {
        NodeKey::reference()
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
        NodeScope::new(NodeKey::reference(), self.input.context)
    }
}

pub struct CreateHandler;
impl<L: LawPolicy> OperationHandler<GraphSchema<L>, CreateBinding<L>> for CreateHandler {
    fn decide(
        &self,
        input: &CreateNodeInput,
        reader: &mut DecisionReader<'_, '_, '_, GraphSchema<L>, CreateBinding<L>>,
    ) -> HandlerResult<WorthQueryInvariantMutationTarget<GraphSchema<L>, Node>, GraphDenial> {
        let source = match reader.resolve_entity(NodeKey::reference(), input.context) {
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
        source: WorthQueryInvariantMutationTarget<GraphSchema<L>, Node>,
        writer: &mut CandidateWriter<'_, GraphSchema<L>, CreateBinding<L>>,
    ) -> HandlerResult<Edited, GraphDenial> {
        let build = |writer: &mut CandidateWriter<'_, GraphSchema<L>, CreateBinding<L>>| {
            let source = writer.projected_entity(&source)?;
            let key = WorthQueryApplicationEntityKey::new(format!("node-{}", input.key)).unwrap();
            let target = if input.in_context {
                writer.create_entity_in_context(&source, Node::reference(), key)?
            } else {
                writer.create_entity(Node::reference(), key)?
            };
            writer.initialize_field(&target, NodeKey::reference(), input.key)?;
            if input.link {
                writer.link(
                    Precedes::reference(),
                    format!("created-edge-{}", input.key),
                    &source,
                    &target,
                )?;
            }
            if input.backlink {
                writer.link(
                    Precedes::reference(),
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
