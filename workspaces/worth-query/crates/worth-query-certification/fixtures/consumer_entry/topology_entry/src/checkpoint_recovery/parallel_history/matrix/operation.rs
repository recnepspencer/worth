//! One member request prepares and computes, then commits its value once.
use super::super::expected_history::Partition;
use super::{application::*, computation, control::Control};
use crate::principal::*;
use std::marker::PhantomData;
use std::sync::Arc;
use worth_query_decl::facade::{
    application_operation::*, application_schema::*, worth_query_operation,
    worth_query_operation_reads, worth_query_operation_writes,
    worth_query_structured_value_binding,
};
use worth_query_host::facade::{application_contribution::*, primary_graph::*};
#[derive(Clone, Debug, serde::Serialize)]
pub struct Input {
    pub revision: u64,
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
    pub phase: super::control::Phase,
}
worth_query_structured_value_binding!(pub InputBinding for Input {identity:"matrix.member.input"});
worth_query_structured_value_binding!(pub ResultBinding for Outcome {identity:"matrix.member.result"});
worth_query_structured_value_binding!(pub DenialBinding for Denied {identity:"matrix.member.denial"});
worth_query_operation!(pub Apply for Schema:SchemaBinding,input InputBinding);
worth_query_operation_reads!(Apply=>[Node,NodeKey,Revision,Value]);
worth_query_operation_writes!(Apply=>[Value]);
pub struct Binding<S>(PhantomData<fn() -> S>);
pub type Scope<S> =
    ApplicationMutationFieldScope<S, Node, NodeFacts, NodeKey, String, ReadOnly, NoApplicationUnit>;
pub struct Decision<S> {
    target: WorthQueryInvariantMutationTarget<S, Node>,
    value: u64,
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
    type Output = Outputs;
    type ScopeBinding = Scope<S>;
    type PrincipalBinding = ConsumerPrincipalBinding;
    type Mapping = ExternalPrincipalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type SourceExpectation = ApplicationQueryMutationSource<super::source::Query>;
    const IDENTITY: &'static str = "matrix.apply";
    const HANDLER_IDENTITY: &'static str = "matrix.apply.handler";
    const IDEMPOTENCY_IDENTITY: &'static str = "matrix.apply.command";
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

pub struct Outputs;
pub struct NodeOutput<S>(PhantomData<fn() -> S>);
impl<S: SchemaBinding> WorthQueryApplicationOutputRole for NodeOutput<S> {
    type Schema = S;
    type Contract = Outputs;
    type Entity = Node;
    type Action = WorthQueryPreserveOutput;
    type Cardinality = WorthQueryExactlyOneOutput;
    const NAME: &'static str = "node";
}
impl<S: SchemaBinding> ApplicationMutationOutputContract<S> for Outputs {
    const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] =
        &[<NodeOutput<S> as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR];
    const ROLE_FAMILIES: &'static [ApplicationMutationOutputRoleFamilyDescriptor] = &[];
}
pub struct Handler {
    pub installed: WorthQueryInstalledPartitionedComputation<
        Schema,
        Feature,
        computation::Computation,
        computation::Owner,
    >,
    pub control: Arc<Control>,
}
impl OperationHandler<Schema, Binding<Schema>> for Handler {
    fn decide(
        &self,
        input: &Input,
        reader: &mut DecisionReader<'_, '_, '_, Schema, Binding<Schema>>,
    ) -> HandlerResult<Decision<Schema>, Denied> {
        let node = reader
            .resolve_entity(NodeKey::reference(), format!("member-{:03}", input.key))
            .unwrap();
        let _current = reader.field(&node, Value::reference()).unwrap();
        HandlerResult::Completed(Decision {
            target: reader.mutation_target(&node).unwrap(),
            value: 0,
            computation_work: 0,
        })
    }
    fn prepare_decision(
        &self,
        input: &Input,
        reader: &mut DecisionReader<'_, '_, '_, Schema, Binding<Schema>>,
    ) -> WorthQueryDecisionPlan<Decision<Schema>, Denied> {
        let decided = self.decide(input, reader);
        if self
            .control
            .fails(input.key, super::control::Phase::Prepare)
        {
            return WorthQueryDecisionPlan::immediate(HandlerResult::DomainDenied(Denied {
                partition: input.key,
                phase: super::control::Phase::Prepare,
            }));
        }
        self.control.declare_begin(input.key);
        let mut upstream = 0;
        for key in &input.upstream {
            self.control.declare_edge(input.key, *key);
            let node = reader
                .resolve_entity(NodeKey::reference(), format!("member-{:03}", key))
                .unwrap();
            match reader.current_output::<super::producer::Family, Node>(&node) {
                Ok(WorthQueryCurrentOutputSelection::Unique(output)) => {
                    upstream += reader.field(&output, Value::reference()).unwrap().unwrap()
                }
                Ok(_) => {
                    return WorthQueryDecisionPlan::immediate(HandlerResult::DomainDenied(Denied {
                        partition: input.key,
                        phase: super::control::Phase::Prepare,
                    }))
                }
                Err(error) => {
                    println!(
                        "MATRIX_UPSTREAM_DENIAL:key={} upstream={key} {error:?}",
                        input.key
                    );
                    return WorthQueryDecisionPlan::immediate(HandlerResult::ExecutionDenied(
                        error,
                    ));
                }
            }
        }
        let mut plan = WorthQueryDecisionPlan::immediate(HandlerResult::Completed(upstream));
        let frames = if self.control.nested {
            vec![input.partitions.clone()]
        } else {
            input
                .partitions
                .iter()
                .map(|part| vec![part.clone()])
                .collect()
        };
        for parts in frames.into_iter().filter(|parts| !parts.is_empty()) {
            let facts = computation::ReadInput {
                node: input.key,
                upstream_stamp: upstream,
                parts: parts.clone(),
            };
            let prepared = if self.control.reuse {
                self.installed.prepare(reader, &facts)
            } else {
                self.installed
                    .prepare_without_reuse_for_test(reader, &facts)
            };
            let node = input.key;
            let prepared = prepared.map(|prepared| {
                if !self.control.nested {
                    return prepared;
                }
                let control = self.control.clone();
                prepared.observe_kernel_cost_for_test(move |work, span| {
                    if !control.fails(node, super::control::Phase::Compute)
                        && control.state.lock().unwrap().armed
                    {
                        control.nested_meter(node, Some((u128::from(work), u128::from(span))));
                    }
                })
            });
            let next = match prepared {
                Ok(prepared) => prepared.into_decision_plan(move |computed| {
                    match computed.and_then(|value| value.complete()) {
                        Ok(value) => HandlerResult::Completed(value),
                        Err(WorthQueryPartitionedComputationDenial::Partition {
                            cause: WorthQueryComputationPartitionStop::Owner(key),
                            ..
                        }) => {
                            assert_eq!(
                                key, node,
                                "typed compute refusal names its member by value"
                            );
                            HandlerResult::DomainDenied(Denied {
                                partition: key,
                                phase: super::control::Phase::Compute,
                            })
                        }
                        Err(error) => panic!("unexpected computation denial: {error:?}"),
                    }
                }),
                Err(error) => panic!("matrix preparation is admissible: {error:?}"),
            };
            let next = if parts.iter().any(|part| part.key == 0) {
                let before = self.control.clone();
                let after = self.control.clone();
                next.observe_computes_for_test(
                    move || {
                        before.dispatched(node);
                    },
                    move || {
                        after.exit_member();
                    },
                    |_| {},
                )
            } else {
                next
            };
            plan = plan.zip(next).map(|(sum, next)| sum + next);
        }
        let control = self.control.clone();
        let key = input.key;
        plan.and_then(move |value| {
            control.completed(key);
            if control.fails(key, super::control::Phase::Complete) {
                return HandlerResult::DomainDenied(Denied {
                    partition: key,
                    phase: super::control::Phase::Complete,
                });
            }
            match decided {
                HandlerResult::Completed(mut decision) => {
                    decision.value = value;
                    HandlerResult::Completed(decision)
                }
                other => other,
            }
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
        writer.preserve_output::<NodeOutput<Schema>>(&node).unwrap();
        writer
            .write_field(&node, Value::reference(), decision.value)
            .unwrap();
        HandlerResult::Completed(Outcome {
            value: decision.value,
            computation_work: decision.computation_work,
        })
    }
}
pub fn declare<S: SchemaBinding>(
    builder: ApplicationSchemaDeclarationBuilder<S>,
) -> ApplicationSchemaDeclarationBuilder<S> {
    let op = Apply::reference::<S>();
    builder
        .operation(op.definition().no_external_effect().no_aftermath().finish())
        .operation_projection_work_budget(op, 4096)
        .operation_read_entity(op, Node::reference())
        .operation_read_field(op, NodeKey::reference())
        .operation_read_field(op, Revision::reference())
        .operation_read_field(op, Value::reference())
        .operation_write(op, Value::reference())
        .application_mutation_binding::<Binding<S>>()
}
