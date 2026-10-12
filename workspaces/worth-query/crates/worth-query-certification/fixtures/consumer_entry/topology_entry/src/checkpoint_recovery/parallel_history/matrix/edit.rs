//! One member request prepares and computes, then commits its value once.
use super::application::*;
use crate::principal::*;
use std::marker::PhantomData;
use worth_query_decl::facade::{
    application_operation::*, application_schema::*, worth_query_operation,
    worth_query_operation_reads, worth_query_operation_writes,
    worth_query_structured_value_binding,
};
use worth_query_host::facade::primary_graph::*;
#[derive(Clone, Debug, serde::Serialize)]
pub struct Input {
    pub key: u64,
    pub revision: u64,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Outcome {
    pub revision: u64,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Denied {
    pub partition: u64,
}
worth_query_structured_value_binding!(pub InputBinding for Input {identity:"matrix.edit.member.input"});
worth_query_structured_value_binding!(pub ResultBinding for Outcome {identity:"matrix.edit.member.result"});
worth_query_structured_value_binding!(pub DenialBinding for Denied {identity:"matrix.edit.member.denial"});
worth_query_operation!(pub Edit for Schema:SchemaBinding,input InputBinding);
worth_query_operation_reads!(Edit=>[Node,NodeKey,Revision]);
worth_query_operation_writes!(Edit=>[Revision]);
pub struct Binding<S>(PhantomData<fn() -> S>);
pub type Scope<S> =
    ApplicationMutationFieldScope<S, Node, NodeFacts, NodeKey, String, ReadOnly, NoApplicationUnit>;
pub struct Decision<S> {
    target: WorthQueryInvariantMutationTarget<S, Node>,
    revision: u64,
}
impl<S: SchemaBinding> ApplicationMutationBinding<S> for Binding<S> {
    type Input = Input;
    type InputBinding = InputBinding;
    type Result = Outcome;
    type ResultBinding = ResultBinding;
    type IdempotencyKey = u64;
    type Operation = Edit;
    type Decision = Decision<S>;
    type Denial = Denied;
    type DenialBinding = DenialBinding;
    type Output = NoApplicationMutationOutputs;
    type ScopeBinding = Scope<S>;
    type PrincipalBinding = ConsumerPrincipalBinding;
    type Mapping = ExternalPrincipalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type SourceExpectation = ApplicationQueryMutationSource<super::source::Query>;
    const IDENTITY: &'static str = "matrix.edit.apply";
    const HANDLER_IDENTITY: &'static str = "matrix.edit.apply.handler";
    const IDEMPOTENCY_IDENTITY: &'static str = "matrix.edit.apply.command";
    const CANDIDATES: ApplicationCandidateRequirements =
        ApplicationCandidateRequirements::fixed_shape(
            ApplicationCandidateCardinalityCeiling::fixed(0, 1, 0, 0, 2, 0),
            ApplicationCandidateResourceCeiling::representation_bytes(4096),
        );
    fn scope_field() -> ApplicationFieldRef<
        S,
        Node,
        NodeFacts,
        NodeKey,
        String,
        ReadOnly,
        EqualityPredicate,
        NoApplicationUnit,
    > {
        NodeKey::reference()
    }
    fn principal_binding() -> ApplicationPrincipalBindingRef<
        S,
        ConsumerPrincipalBinding,
        ExternalPrincipalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        ConsumerPrincipalBinding::reference()
    }
}
impl<S: SchemaBinding> ApplicationMutationIntent<S> for Input {
    type Binding = Binding<S>;
    fn input(&self) -> &Self {
        self
    }
    fn scope_binding(&self) -> Scope<S> {
        Scope::new(NodeKey::reference(), format!("member-{:03}", self.key))
    }
}

pub struct Handler;
impl OperationHandler<Schema, Binding<Schema>> for Handler {
    fn decide(
        &self,
        input: &Input,
        reader: &mut DecisionReader<'_, '_, '_, Schema, Binding<Schema>>,
    ) -> HandlerResult<Decision<Schema>, Denied> {
        let node = reader
            .resolve_entity(NodeKey::reference(), format!("member-{:03}", input.key))
            .unwrap();
        let _previous = reader.field(&node, Revision::reference()).unwrap();
        HandlerResult::Completed(Decision {
            target: reader.mutation_target(&node).unwrap(),
            revision: input.revision,
        })
    }
    fn candidate_requirements(
        &self,
        _: &Input,
        _: &Decision<Schema>,
    ) -> ApplicationCandidateRequirements {
        Binding::<Schema>::CANDIDATES
    }
    fn build_candidate(
        &self,
        _: &Input,
        decision: Decision<Schema>,
        writer: &mut CandidateWriter<'_, Schema, Binding<Schema>>,
    ) -> HandlerResult<Outcome, Denied> {
        let node = writer.projected_entity(&decision.target).unwrap();
        writer
            .write_field(&node, Revision::reference(), decision.revision)
            .unwrap();
        HandlerResult::Completed(Outcome {
            revision: decision.revision,
        })
    }
}
pub fn declare<S: SchemaBinding>(
    builder: ApplicationSchemaDeclarationBuilder<S>,
) -> ApplicationSchemaDeclarationBuilder<S> {
    let op = Edit::reference::<S>();
    builder
        .operation(op.definition().no_external_effect().no_aftermath().finish())
        .operation_projection_work_budget(op, 4096)
        .operation_read_entity(op, Node::reference())
        .operation_read_field(op, NodeKey::reference())
        .operation_read_field(op, Revision::reference())
        .operation_write(op, Revision::reference())
        .application_mutation_binding::<Binding<S>>()
}
