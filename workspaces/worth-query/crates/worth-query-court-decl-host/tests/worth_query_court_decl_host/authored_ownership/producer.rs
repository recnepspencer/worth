use super::*;
use query::{Input, InputBinding, Row, UnitBinding};
pub struct Operation;
impl<S: Scenario> ApplicationOperationMarkerIdentity<S> for Operation {
    type InputBinding = InputBinding;
    const IDENTIFIER: &'static str = "CreateDerived";
}
impl OperationReads<Operation> for Id {}
impl OperationCreates<Operation> for Node {}
impl OperationWrites<Operation> for Id {}
impl OperationWrites<Operation> for Value {}
pub struct Outputs;
impl<S: Scenario> WorthQueryApplicationOutputRole for Role<S> {
    type Schema = S;
    type Contract = Outputs;
    type Entity = Node;
    type Action = WorthQueryCreateOutput;
    type Cardinality = WorthQueryExactlyOneOutput;
    const NAME: &'static str = "derived";
}
pub struct Cell<S>(PhantomData<fn() -> S>);
impl<S: Scenario> WorthQueryApplicationOutputRole for Cell<S> {
    type Schema = S;
    type Contract = Outputs;
    type Entity = Node;
    type Action = WorthQueryCreateOutput;
    type Cardinality = WorthQueryExactlyOneOutput;
    const NAME: &'static str = "cell-or-child-source";
}
pub struct Role<S>(PhantomData<fn() -> S>);
impl<S: Scenario> ApplicationMutationOutputContract<S> for Outputs {
    const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] = &[
        <Role<S> as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR,
        <Cell<S> as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR,
    ];
}
type Scope<S> = ApplicationMutationFieldScope<S, Node, Facts, Id, u64, ReadOnly, NoApplicationUnit>;
pub struct Binding<S>(PhantomData<fn() -> S>);
impl<S: Scenario> ApplicationMutationAdmissionBinding<S> for Binding<S> {
    type Admission = ApplicationConventionalAdmission;
}
impl<S: Scenario> ApplicationMutationBinding<S> for Binding<S> {
    type Input = Input;
    type InputBinding = InputBinding;
    type Operation = Operation;
    type Result = ();
    type ResultBinding = UnitBinding;
    type IdempotencyKey = u64;
    type Decision = WorthQueryInvariantMutationTarget<S, Node>;
    type Denial = ();
    type DenialBinding = UnitBinding;
    type Output = Outputs;
    type ScopeBinding = Scope<S>;
    type PrincipalBinding = Actor;
    type Mapping = Mapping;
    type Principal = Node;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type SourceExpectation = ApplicationQueryMutationSource<query::Query>;
    const IDENTITY: &'static str = "authored-ownership.create.v1";
    const HANDLER_IDENTITY: &'static str = "authored-ownership.handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str = "authored-ownership.key.v1";
    fn workflow_execution_requirements() -> ApplicationWorkflowExecutionRequirements {
        ApplicationWorkflowExecutionRequirements::not_applicable()
    }
    fn scope_field(
    ) -> ApplicationFieldRef<S, Node, Facts, Id, u64, ReadOnly, EqualityPredicate, NoApplicationUnit>
    {
        Id::reference()
    }
    fn principal_binding(
    ) -> ApplicationPrincipalBindingRef<S, Actor, Mapping, Node, u64, U64ApplicationValueBinding>
    {
        actor()
    }
}
impl<S: Scenario> ApplicationMutationIntent<S> for Input {
    type Binding = Binding<S>;
    fn input(&self) -> &Self {
        self
    }
    fn scope_binding(&self) -> Scope<S> {
        Scope::new(Id::reference(), self.id)
    }
}
pub struct Family;
impl<S: Scenario> WorthQueryProducerOutputFamily<S> for Family {
    type Source = query::Binding<S>;
    type Entity = Node;
    const IDENTITY: &'static str = "authored-ownership.family.v1";
    const SUPPORTED: &'static [WorthQueryProducerApplicability] =
        &[WorthQueryProducerApplicability::new(
            "node",
            WorthQueryProducerLifecyclePosture::Initial,
        )];
    fn profile_kind(_: &Row) -> &'static str {
        "node"
    }
}
pub struct Producer<S>(PhantomData<fn() -> S>);
pub struct Provider;
impl<S: Scenario> WorthQueryApplicationProducerBinding<S> for Producer<S> {
    type Operation = Binding<S>;
    type OutputFamily = Family;
    type Provider = Provider;
    type OutputRole = Role<S>;
    const IDENTITY: &'static str = "authored-ownership.producer.v1";
    const APPLICABILITY: &'static [WorthQueryProducerApplicability] =
        <Family as WorthQueryProducerOutputFamily<S>>::SUPPORTED;
    const REQUIRED_INVARIANTS: &'static [WorthQueryProducerInvariantRequirement] = &[];
    const REUSE_POLICY: &'static str = "exact-source";
    const OWNERSHIP: ProducerPublicationOwnership = if S::MODE == 3 {
        ProducerPublicationOwnership::Independent
    } else {
        ProducerPublicationOwnership::SourceScopeCascade
    };
}
impl<S: Scenario> WorthQueryApplicationProducerProvider<S, Producer<S>> for Provider {
    type DenialValue = WorthQueryUndeclaredProducerDenial;
    const SEMANTIC_IDENTITY: &'static str = "authored-ownership.provider.v1";
    fn operation_input(&self, source: &Row) -> Input {
        Input { id: source.id }
    }
    fn idempotency_key(&self, _: &Row, source: &[u8; 32]) -> u64 {
        u64::from_le_bytes(source[..8].try_into().unwrap())
    }
}
impl<S: Scenario> OperationHandler<S, Binding<S>> for Provider {
    fn decide(
        &self,
        input: &Input,
        reader: &mut DecisionReader<'_, '_, '_, S, Binding<S>>,
    ) -> HandlerResult<WorthQueryInvariantMutationTarget<S, Node>, ()> {
        let result = reader
            .resolve_entity(Id::reference(), input.id)
            .and_then(|source| reader.mutation_target(&source));
        match result {
            Ok(target) => HandlerResult::Completed(target),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
    fn build_candidate(
        &self,
        input: &Input,
        target: WorthQueryInvariantMutationTarget<S, Node>,
        writer: &mut CandidateWriter<'_, S, Binding<S>>,
    ) -> HandlerResult<(), ()> {
        let Some(id) = input.id.checked_mul(10).and_then(|id| id.checked_add(1)) else {
            return HandlerResult::DomainDenied(());
        };
        let build = (|| {
            let source = writer.projected_entity(&target)?;
            let created = writer.create_entity_in_context(
                &source,
                Node::reference(),
                WorthQueryApplicationEntityKey::new(format!("derived-{id}")).unwrap(),
            )?;
            writer.initialize_field(&created, Id::reference(), id)?;
            writer.initialize_field(&created, Value::reference(), 39)?;
            writer.create_output::<Role<S>>(&created)?;
            let cell = writer.create_entity_in_context(
                &source,
                Node::reference(),
                WorthQueryApplicationEntityKey::new(format!("derived-{}", id + 1)).unwrap(),
            )?;
            writer.initialize_field(&cell, Id::reference(), id + 1)?;
            writer.initialize_field(&cell, Value::reference(), 39)?;
            writer.create_output::<Cell<S>>(&cell)
        })();
        match build {
            Ok(()) => HandlerResult::Completed(()),
            Err(cause) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause)),
        }
    }
}
pub fn declare<S: Scenario>(
    schema: ApplicationSchemaDeclarationBuilder<S>,
) -> ApplicationSchemaDeclarationBuilder<S> {
    let operation = ApplicationOperationRef::<S, Operation, Input>::from_declaration();
    schema
        .operation(
            operation
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_read_field(operation, Id::reference())
        .operation_create(operation, Node::reference())
        .operation_write(operation, Id::reference())
        .operation_write(operation, Value::reference())
        .application_mutation_binding::<Binding<S>>()
}
