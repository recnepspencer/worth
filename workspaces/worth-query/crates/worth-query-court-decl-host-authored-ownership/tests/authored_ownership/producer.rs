use super::*;
use query::{Input, InputBinding, Row, UnitBinding};
pub struct Operation;
impl<S: Scenario> ApplicationOperationMarkerIdentity<S> for Operation {
    type InputBinding = InputBinding;
    const IDENTIFIER: &'static str = "CreateDerived";
}
impl<const CONTENT: bool> OperationReads<Operation> for RawId<CONTENT> {}
impl<const CONTENT: bool> OperationCreates<Operation> for NativeNode<CONTENT> {}
impl OperationLinks<Operation> for Peer {}
impl<const CONTENT: bool> OperationWrites<Operation> for RawId<CONTENT> {}
impl<const CONTENT: bool> OperationWrites<Operation> for RawValue<CONTENT> {}
pub struct Outputs;
impl<S: Scenario> WorthQueryApplicationOutputRole for Role<S> {
    type Schema = S;
    type Contract = Outputs;
    type Entity = Node<S>;
    type Action = WorthQueryCreateOutput;
    type Cardinality = WorthQueryExactlyOneOutput;
    const NAME: &'static str = "derived";
}
pub struct Cell<S>(PhantomData<fn() -> S>);
impl<S: Scenario> WorthQueryApplicationOutputRole for Cell<S> {
    type Schema = S;
    type Contract = Outputs;
    type Entity = Node<S>;
    type Action = WorthQueryCreateOutput;
    type Cardinality = WorthQueryExactlyOneOutput;
    const NAME: &'static str = "cell-or-child-source";
}
pub struct Role<S>(PhantomData<fn() -> S>);
impl<S: Scenario> ApplicationMutationOutputContract<S> for Outputs {
    const ROLE_FAMILIES: &'static [ApplicationMutationOutputRoleFamilyDescriptor] = if matches!(
        S::MODE,
        69 | 72
    ) {
        &[<sheet_boundary::InitialNodes<S> as WorthQueryApplicationDeclaredOutputRoleFamily>::DESCRIPTOR]
    } else if S::MODE == 70 {
        &[
            <sheet_boundary::InitialNodes<S> as WorthQueryApplicationDeclaredOutputRoleFamily>::DESCRIPTOR,
            <sheet_boundary::split_walk::InitialFork<S> as WorthQueryApplicationDeclaredOutputRoleFamily>::DESCRIPTOR,
        ]
    } else {
        &[]
    };
    const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] = &[
        <Role<S> as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR,
        <Cell<S> as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR,
    ];
}
type Scope<S> = ApplicationMutationFieldScope<Id<S>>;
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
    type Decision = InitialTargets<S>;
    type Denial = ();
    type DenialBinding = UnitBinding;
    type Output = Outputs;
    type ScopeBinding = Scope<S>;
    type PrincipalBinding = Actor;
    type Mapping = Mapping;
    type Principal = Node<S>;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type SourceExpectation = ApplicationQueryMutationSource<query::Query>;
    const IDENTITY: &'static str = "authored-ownership.create.v1";
    const HANDLER_IDENTITY: &'static str = "authored-ownership.handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str = "authored-ownership.key.v1";
    fn workflow_execution_requirements() -> ApplicationWorkflowExecutionRequirements {
        ApplicationWorkflowExecutionRequirements::not_applicable()
    }
    fn scope_field() -> Id<S> {
        schema::id::<S>()
    }
    fn principal_binding(
    ) -> ApplicationPrincipalBindingRef<S, Actor, Mapping, Node<S>, u64, U64ApplicationValueBinding>
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
        Scope::<S>::new(schema::id::<S>(), self.id)
    }
}
pub struct Family;
impl<S: Scenario> WorthQueryProducerOutputFamily<S> for Family {
    type Source = query::Binding<S>;
    type Entity = Node<S>;
    const IDENTITY: &'static str = "authored-ownership.family.v1";
    const SUPPORTED: &'static [WorthQueryProducerApplicability] = if S::COUPLED {
        &[
            WorthQueryProducerApplicability::new(
                "node",
                WorthQueryProducerLifecyclePosture::Initial,
            ),
            WorthQueryProducerApplicability::new(
                "node",
                WorthQueryProducerLifecyclePosture::Preserve,
            ),
        ]
    } else {
        &[WorthQueryProducerApplicability::new(
            "node",
            WorthQueryProducerLifecyclePosture::Initial,
        )]
    };
    fn profile_kind(_: &Row) -> &'static str {
        "node"
    }
}
pub struct Producer<S>(PhantomData<fn() -> S>);
pub struct InitialTargets<S: Scenario> {
    source: WorthQueryInvariantMutationTarget<S, Node<S>>,
    stage: Option<WorthQueryInvariantMutationTarget<S, Node<S>>>,
    other_stage: Option<WorthQueryInvariantMutationTarget<S, Node<S>>>,
}
pub struct Provider;
impl<S: Scenario> WorthQueryApplicationProducerBinding<S> for Producer<S> {
    type Operation = Binding<S>;
    type OutputFamily = Family;
    type Provider = Provider;
    type OutputRole = Role<S>;
    const IDENTITY: &'static str = "authored-ownership.producer.v1";
    const APPLICABILITY: &'static [WorthQueryProducerApplicability] =
        &[WorthQueryProducerApplicability::new(
            "node",
            WorthQueryProducerLifecyclePosture::Initial,
        )];
    const REQUIRED_INVARIANTS: &'static [WorthQueryProducerInvariantRequirement] = &[];
    const REUSE_POLICY: &'static str = "exact-source";
    const OWNERSHIP: ProducerPublicationOwnership = if matches!(S::MODE, 3 | 65) {
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
    ) -> HandlerResult<InitialTargets<S>, ()> {
        let result = (|| {
            let source = reader.resolve_entity(schema::id::<S>(), input.id)?;
            let source = reader.mutation_target(&source)?;
            let stage = if S::COUPLED {
                let stage = reader.resolve_entity(
                    schema::id::<S>(),
                    if S::MODE == 72 {
                        if input.id == 10 {
                            20
                        } else if input.id == 40 {
                            50
                        } else {
                            input.id
                        }
                    } else if S::MODE == 17 && input.id == 11 {
                        30
                    } else {
                        20
                    },
                )?;
                Some(reader.mutation_target(&stage)?)
            } else {
                None
            };
            let other_stage = if matches!(S::MODE, 60 | 61 | 64 | 30 | 40 | 41) {
                reader
                    .resolve_optional_entity(schema::id::<S>(), 30)?
                    .map(|stage| reader.mutation_target(&stage))
                    .transpose()?
            } else {
                None
            };
            Ok::<_, HandlerExecutionDenial>(InitialTargets {
                source,
                stage,
                other_stage,
            })
        })();
        match result {
            Ok(target) => HandlerResult::Completed(target),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
    fn build_candidate(
        &self,
        input: &Input,
        target: InitialTargets<S>,
        writer: &mut CandidateWriter<'_, S, Binding<S>>,
    ) -> HandlerResult<(), ()> {
        let Some(id) = input.id.checked_mul(10).and_then(|id| id.checked_add(1)) else {
            return HandlerResult::DomainDenied(());
        };
        let build = (|| {
            let source = writer.projected_entity(&target.source)?;
            let created = writer.create_entity_in_context(
                &source,
                schema::node::<S>(),
                WorthQueryApplicationEntityKey::new(format!("derived-{id}")).unwrap(),
            )?;
            writer.initialize_field(&created, schema::id::<S>(), id)?;
            writer.initialize_field(&created, schema::value::<S>(), 39)?;
            if S::MODE == 43 {
                writer
                    .initialize_content(
                        &created,
                        content::field::<S>().expect("the content scenario declares its field"),
                        &39,
                    )
                    .expect("valid installed word content");
            }
            writer.create_output::<Role<S>>(&created)?;
            if S::MODE == 5 {
                writer.link(
                    Peer::reference(),
                    format!("owned-source:{}", input.id),
                    &created,
                    &source,
                )?;
            }
            let cell = writer.create_entity_in_context(
                &source,
                schema::node::<S>(),
                WorthQueryApplicationEntityKey::new(format!("derived-{}", id + 1)).unwrap(),
            )?;
            writer.initialize_field(&cell, schema::id::<S>(), id + 1)?;
            writer.initialize_field(&cell, schema::value::<S>(), 39)?;
            if S::MODE == 13 {
                writer.link(
                    Peer::reference(),
                    "component-business-edge",
                    &created,
                    &cell,
                )?;
            }
            if let Some(other) = target.other_stage {
                let other = writer.projected_entity(&other)?;
                writer.link(Peer::reference(), "body-other-stage", &cell, &other)?;
            }
            if let Some(stage) = &target.stage {
                let stage = writer.projected_entity(stage)?;
                if S::MODE == 66 {
                    writer.link(
                        Peer::reference(),
                        "external-to-owned-member",
                        &stage,
                        &created,
                    )?;
                } else {
                    writer.link(Peer::reference(), "body-stage", &cell, &stage)?;
                }
            }
            if matches!(S::MODE, 42 | 64) {
                writer.link(OwnCell::reference(), "owned-cell", &cell, &created)?;
            }
            let retained_wire_body = S::MODE == 72 && matches!(input.id, 10 | 40);
            if matches!(S::MODE, 22 | 23 | 26 | 27 | 28) || retained_wire_body {
                writer.link(StageVisible::reference(), "body-visible", &source, &created)?;
                writer.link(OwnCell::reference(), "owned-cell", &cell, &created)?;
            }
            if S::MODE == 40 {
                writer.link(
                    PreviousStage::reference(),
                    "owned-stage-predecessor",
                    &cell,
                    &created,
                )?;
            }
            if S::MODE == 43 {
                writer
                    .initialize_content(
                        &cell,
                        content::field::<S>().expect("the content scenario declares its field"),
                        &39,
                    )
                    .expect("valid installed word content");
            }
            <S::Boundary as sheet_boundary::Profile>::initial(writer, &source)?;
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
        .operation_read_field(operation, schema::id::<S>())
        .operation_create(operation, schema::node::<S>())
        .operation_link(operation, Peer::reference())
        .operation_write(operation, schema::id::<S>())
        .operation_write(operation, schema::value::<S>())
        .application_mutation_binding::<Binding<S>>()
}
