//! A normal authenticated action returns the actual managed count.
use super::super::operation::{GraphDenial, GraphDenialBinding};
use super::*;
use std::marker::PhantomData;
use worth_query_decl::facade::worth_query_operation_reads;
use worth_query_host::facade::{
    declaration::application_operation::*, worth_query_structured_value_binding,
};

#[derive(Clone, Debug, serde::Serialize)]
pub(crate) struct Input {
    pub(crate) root: u64,
    pub(crate) bad_reference: bool,
}
worth_query_structured_value_binding!(pub(crate) InputBinding for Input { identity:"worth.query.derived.count-input.v1" });
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Counted {
    pub(crate) count: u64,
}
worth_query_structured_value_binding!(pub(crate) ResultBinding for Counted { identity:"worth.query.derived.count-result.v1" });
pub(crate) struct CountTwoSteps;
impl<S: GraphSchemaBinding> ApplicationOperationMarkerIdentity<S> for CountTwoSteps {
    type InputBinding = InputBinding;
    const IDENTIFIER: &'static str = "CountTwoSteps";
}
impl CountTwoSteps {
    pub(crate) fn reference<S: GraphSchemaBinding>() -> ApplicationOperationRef<S, Self, Input> {
        ApplicationOperationRef::from_declaration()
    }
}
worth_query_operation_reads!(CountTwoSteps => [NodeKey]);
pub(crate) struct CountBinding<L: LawPolicy>(PhantomData<fn() -> L>);
type Scope<L> = ApplicationMutationFieldScope<
    GraphSchema<L>,
    Node,
    NodeFacts,
    NodeKey,
    u64,
    ReadOnly,
    NoApplicationUnit,
>;
impl<L: LawPolicy> ApplicationMutationBinding<GraphSchema<L>> for CountBinding<L> {
    type Input = Input;
    type InputBinding = InputBinding;
    type Result = Counted;
    type ResultBinding = ResultBinding;
    type IdempotencyKey = u64;
    type Operation = CountTwoSteps;
    type Decision = u64;
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
    const IDENTITY: &'static str = "worth.query.derived.count-binding.v1";
    const HANDLER_IDENTITY: &'static str = "worth.query.derived.count-handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str = "worth.query.derived.count-command.v1";
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
pub(crate) struct Intent<L: LawPolicy> {
    pub(crate) input: Input,
    pub(crate) marker: PhantomData<fn() -> L>,
}
impl<L: LawPolicy> ApplicationMutationIntent<GraphSchema<L>> for Intent<L> {
    type Binding = CountBinding<L>;
    fn input(&self) -> &Input {
        &self.input
    }
    fn scope_binding(&self) -> Scope<L> {
        Scope::new(NodeKey::reference(), self.input.root)
    }
}
