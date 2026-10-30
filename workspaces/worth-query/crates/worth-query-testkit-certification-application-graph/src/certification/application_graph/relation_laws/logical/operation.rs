//! A normal authenticated action returns the actual managed count.
pub use super::super::operation::{GraphDenial, GraphDenialBinding};
pub use super::*;
pub use std::marker::PhantomData;
pub use worth_query_decl::facade::worth_query_operation_reads;
pub use worth_query_host::facade::{
    declaration::application_operation::*, worth_query_structured_value_binding,
};

#[derive(Clone, Debug, serde::Serialize)]
pub struct Input {
    pub root: u64,
}
worth_query_structured_value_binding!(pub InputBinding for Input { identity:"worth.query.derived.logical-input.v1" });
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Checked {
    pub members: Vec<(
        worth_query_host::facade::application_invariants::EntityId,
        String,
    )>,
    pub distances: Vec<usize>,
    pub identity: [u8; 32],
}
worth_query_structured_value_binding!(pub ResultBinding for Checked { identity:"worth.query.derived.logical-result.v1" });
pub struct CheckUses;
impl<S: GraphSchemaBinding> ApplicationOperationMarkerIdentity<S> for CheckUses {
    type InputBinding = InputBinding;
    const IDENTIFIER: &'static str = "CheckUses";
}
impl CheckUses {
    pub fn reference<S: GraphSchemaBinding>() -> ApplicationOperationRef<S, Self, Input> {
        ApplicationOperationRef::from_declaration()
    }
}
worth_query_operation_reads!(CheckUses => [NodeKey]);
pub struct CheckBinding<L: LawPolicy>(pub PhantomData<fn() -> L>);
pub type Scope<L> = ApplicationMutationFieldScope<
    GraphSchema<L>,
    Node,
    NodeFacts,
    NodeKey,
    u64,
    ReadOnly,
    NoApplicationUnit,
>;
impl<L: LawPolicy> ApplicationMutationBinding<GraphSchema<L>> for CheckBinding<L> {
    type Input = Input;
    type InputBinding = InputBinding;
    type Result = Checked;
    type ResultBinding = ResultBinding;
    type IdempotencyKey = u64;
    type Operation = CheckUses;
    type Decision = Checked;
    type Denial = GraphDenial;
    type DenialBinding = GraphDenialBinding;
    type Output = NoApplicationMutationOutputs;
    type ScopeBinding = Scope<L>;
    type PrincipalBinding = GraphPrincipal;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type SourceExpectation = NoApplicationMutationSource;
    const IDENTITY: &'static str = "worth.query.derived.logical-binding.v1";
    const HANDLER_IDENTITY: &'static str = "worth.query.derived.logical-handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str = "worth.query.derived.logical-command.v1";
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
pub struct Intent<L: LawPolicy> {
    pub input: Input,
    pub marker: PhantomData<fn() -> L>,
}
impl<L: LawPolicy> ApplicationMutationIntent<GraphSchema<L>> for Intent<L> {
    type Binding = CheckBinding<L>;
    fn input(&self) -> &Input {
        &self.input
    }
    fn scope_binding(&self) -> Scope<L> {
        Scope::new(NodeKey::reference(), self.input.root)
    }
}
