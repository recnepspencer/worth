//! One authenticated program action authors a whole proposed edge batch.
use super::schema::*;
use std::marker::PhantomData;
use worth_query_decl::facade::{worth_query_operation_links, worth_query_operation_reads};
use worth_query_host::facade::{
    declaration::{application_operation::*, application_schema::*},
    primary_graph::*,
    worth_query_structured_value_binding,
};

#[derive(Clone, Debug, serde::Serialize)]
pub struct EdgeEdit {
    pub serial: u64,
    pub edges: Vec<(u64, u64)>,
}
worth_query_structured_value_binding!(pub EdgeEditBinding for EdgeEdit { identity: "worth.query.relation-laws.edge-edit.v1" });
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Edited;
worth_query_structured_value_binding!(pub EditedBinding for Edited { identity: "worth.query.relation-laws.edited.v1" });
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GraphDenial;
worth_query_structured_value_binding!(pub GraphDenialBinding for GraphDenial { identity: "worth.query.relation-laws.no-domain-cause.v1" });

pub struct EditEdges;
impl<S: GraphSchemaBinding> ApplicationOperationMarkerIdentity<S> for EditEdges {
    type InputBinding = EdgeEditBinding;
    const IDENTIFIER: &'static str = "EditEdges";
}
impl EditEdges {
    pub fn reference<S: GraphSchemaBinding>() -> ApplicationOperationRef<S, Self, EdgeEdit> {
        ApplicationOperationRef::from_declaration()
    }
}
worth_query_operation_reads!(EditEdges => [NodeKey]);
worth_query_operation_links!(EditEdges => [Precedes]);

pub struct EditBinding<L: LawPolicy>(PhantomData<fn() -> L>);
pub type Targets<L> = Vec<(
    WorthQueryInvariantMutationTarget<GraphSchema<L>, Node>,
    WorthQueryInvariantMutationTarget<GraphSchema<L>, Node>,
)>;
type NodeScope<L> = ApplicationMutationFieldScope<
    GraphSchema<L>,
    Node,
    NodeFacts,
    NodeKey,
    u64,
    ReadOnly,
    NoApplicationUnit,
>;
impl<L: LawPolicy> ApplicationMutationBinding<GraphSchema<L>> for EditBinding<L> {
    type Input = EdgeEdit;
    type InputBinding = EdgeEditBinding;
    type Result = Edited;
    type ResultBinding = EditedBinding;
    type IdempotencyKey = u64;
    type Operation = EditEdges;
    type Decision = Targets<L>;
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
    const IDENTITY: &'static str = "worth.query.relation-laws.edit-binding.v1";
    const HANDLER_IDENTITY: &'static str = "worth.query.relation-laws.handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str = "worth.query.relation-laws.command.v1";
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
pub struct Edit<L: LawPolicy> {
    pub input: EdgeEdit,
    pub marker: PhantomData<fn() -> L>,
}
impl<L: LawPolicy> ApplicationMutationIntent<GraphSchema<L>> for Edit<L> {
    type Binding = EditBinding<L>;
    fn input(&self) -> &EdgeEdit {
        &self.input
    }
    fn scope_binding(&self) -> NodeScope<L> {
        NodeScope::new(
            NodeKey::reference(),
            self.input.edges.first().map_or(0, |edge| edge.0),
        )
    }
}
