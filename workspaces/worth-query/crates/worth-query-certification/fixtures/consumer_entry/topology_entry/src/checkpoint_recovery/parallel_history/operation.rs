//! One member request prepares and computes, then commits its value once.
use super::{application::*, computation, expected_history::Partition};
use crate::principal::*;
use std::marker::PhantomData;

use worth_query_decl::facade::{
    application_operation::*, application_schema::*, worth_query_operation,
    worth_query_operation_reads, worth_query_operation_writes,
    worth_query_structured_value_binding,
};
use worth_query_host::facade::{application_contribution::*, primary_graph::*};
#[derive(Clone, Debug, serde::Serialize)]
pub struct Input {
    pub key: u64,
    pub upstream: Vec<u64>,
    pub partitions: Vec<Partition>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Outcome {
    pub value: u64,
    pub computation_work: u64,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Denied {
    pub partition: u64,
}
worth_query_structured_value_binding!(pub InputBinding for Input {identity:"neutral.member.input"});
worth_query_structured_value_binding!(pub ResultBinding for Outcome {identity:"neutral.member.result"});
worth_query_structured_value_binding!(pub DenialBinding for Denied {identity:"neutral.member.denial"});
worth_query_operation!(pub Apply for Schema:SchemaBinding,input InputBinding);
worth_query_operation_reads!(Apply=>[Node,NodeKey,Value,Applied]);
worth_query_operation_writes!(Apply=>[Value,Applied]);
pub struct Binding<S>(PhantomData<fn() -> S>);
pub type Scope<S> =
    ApplicationMutationFieldScope<S, Node, NodeFacts, NodeKey, u64, ReadOnly, NoApplicationUnit>;
pub struct Decision<S> {
    target: WorthQueryInvariantMutationTarget<S, Node>,
    value: u64,
    applied: u64,
    computation_work: u64,
}
impl<S: SchemaBinding> ApplicationMutationBinding<S> for Binding<S> {
    fn workflow_execution_requirements(
    ) -> worth_query_decl::facade::application_operation::ApplicationWorkflowExecutionRequirements
    {
        worth_query_decl::facade::application_operation::ApplicationWorkflowExecutionRequirements::not_applicable()
    }

    type Input = Input;
    type InputBinding = InputBinding;
    type Result = Outcome;
    type ResultBinding = ResultBinding;
    type IdempotencyKey = u64;
    type Operation = Apply;
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
    type SourceExpectation = NoApplicationMutationSource;
    const IDENTITY: &'static str = "neutral.apply";
    const HANDLER_IDENTITY: &'static str = "neutral.apply.handler";
    const IDEMPOTENCY_IDENTITY: &'static str = "neutral.apply.command";

    fn scope_field() -> ApplicationFieldRef<
        S,
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
        Scope::new(NodeKey::reference(), self.key)
    }
}
pub struct Handler(
    pub  WorthQueryInstalledPartitionedComputation<
        Schema,
        Feature,
        computation::Computation,
        computation::Owner,
    >,
);
impl OperationHandler<Schema, Binding<Schema>> for Handler {
    fn decide(
        &self,
        input: &Input,
        reader: &mut DecisionReader<'_, '_, '_, Schema, Binding<Schema>>,
    ) -> HandlerResult<Decision<Schema>, Denied> {
        let decided = (|| {
            let node = reader.resolve_entity(NodeKey::reference(), input.key)?;
            let value = reader.field(&node, Value::reference())?.unwrap();
            let applied = reader.field(&node, Applied::reference())?.unwrap();
            Ok(Decision {
                target: reader.mutation_target(&node)?,
                value,
                applied: applied + 1,
                computation_work: 0,
            })
        })();
        match decided {
            Ok(value) => HandlerResult::Completed(value),
            Err(error) => HandlerResult::ExecutionDenied(error),
        }
    }
    fn prepare_decision(
        &self,
        input: &Input,
        reader: &mut DecisionReader<'_, '_, '_, Schema, Binding<Schema>>,
    ) -> WorthQueryDecisionPlan<Decision<Schema>, Denied> {
        let upstream = match input
            .upstream
            .iter()
            .map(|key| reader.resolve_entity(NodeKey::reference(), *key))
            .collect::<Result<Vec<_>, _>>()
        {
            Ok(upstream) => upstream,
            Err(error) => {
                return WorthQueryDecisionPlan::immediate(HandlerResult::ExecutionDenied(error))
            }
        };
        let facts = computation::ReadInput {
            upstream,
            partitions: input.partitions.clone(),
        };
        let prepared = self.0.prepare(reader, &facts);
        let decided = self.decide(input, reader);
        let finish = move |computed: Result<
            WorthQueryCompletedPartitionedComputation<
                Schema,
                Feature,
                computation::Computation,
                computation::Owner,
            >,
            _,
        >| {
            let computed = match computed {
                Ok(computed) => computed,
                Err(WorthQueryPartitionedComputationDenial::Partition {
                    cause: WorthQueryComputationPartitionStop::Owner(partition),
                    ..
                }) => return HandlerResult::DomainDenied(Denied { partition }),
                Err(error) => panic!("unexpected computation denial: {error:?}"),
            };
            let computation_work = computed.charged_work();
            let value = computed.complete().unwrap();
            match decided {
                HandlerResult::Completed(mut decision) => {
                    decision.value = value;
                    decision.computation_work = computation_work;
                    HandlerResult::Completed(decision)
                }
                HandlerResult::DomainDenied(cause) => HandlerResult::DomainDenied(cause),
                HandlerResult::ExecutionDenied(cause) => HandlerResult::ExecutionDenied(cause),
                HandlerResult::Cancelled => HandlerResult::Cancelled,
                HandlerResult::DeadlineExceeded => HandlerResult::DeadlineExceeded,
            }
        };
        match prepared {
            Ok(prepared) => prepared.into_decision_plan(finish),
            Err(error) => WorthQueryDecisionPlan::immediate(finish(Err(error))),
        }
    }

    fn build_candidate(
        &self,
        _: &Input,
        decision: Decision<Schema>,
        writer: &mut CandidateWriter<'_, Schema, Binding<Schema>>,
    ) -> HandlerResult<Outcome, Denied> {
        let written = writer.projected_entity(&decision.target).and_then(|node| {
            writer.write_field(&node, Value::reference(), decision.value)?;
            writer.write_field(&node, Applied::reference(), decision.applied)
        });
        match written {
            Ok(()) => HandlerResult::Completed(Outcome {
                value: decision.value,
                computation_work: decision.computation_work,
            }),
            Err(error) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error)),
        }
    }
}
pub fn declare<S: SchemaBinding>(
    builder: ApplicationSchemaDeclarationBuilder<S>,
) -> ApplicationSchemaDeclarationBuilder<S> {
    let operation = Apply::reference::<S>();
    builder
        .operation(
            operation
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_projection_work_budget(operation, 4096)
        .operation_read_entity(operation, Node::reference())
        .operation_read_field(operation, NodeKey::reference())
        .operation_read_field(operation, Value::reference())
        .operation_read_field(operation, Applied::reference())
        .operation_write(operation, Value::reference())
        .operation_write(operation, Applied::reference())
        .application_mutation_binding::<Binding<S>>()
}
