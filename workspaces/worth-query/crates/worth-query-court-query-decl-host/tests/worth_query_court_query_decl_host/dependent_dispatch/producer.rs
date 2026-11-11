use super::*;
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Denial {
    InitialHasPriorOutput,
    PreserveMissingOutput,
    WrongPriorOutput,
    WrongParent,
}
worth_query_structured_value_binding!(pub DenialBinding for Denial { identity: "dispatch.denial" });
worth_query_structured_value_binding!(pub ResultBinding for () { identity: "dispatch.result" });
pub type Target = WorthQueryInvariantMutationTarget<Schema, Node>;
pub struct Decision {
    pub source: Target,
    pub children: Vec<Option<Target>>,
}
pub type Scope = ApplicationMutationFieldScope<Key>;
macro_rules! phase {
    ($module:ident, $operation:ident, $preserve:expr, $posture:ident, $identity:literal) => {
        pub mod $module {
            use super::*;
            #[derive(Clone, serde::Serialize)]
            pub struct Input(pub Row);
            worth_query_structured_value_binding!(pub InputBinding for Input { identity: $identity });
            worth_query_operation!(pub $operation for Schema, input InputBinding;
                access { read_entities: [Node]; read_fields: [Key::at("Key"), ParentName::at("ParentName"), Value::at("Value"), Width::at("Width"), Produced::at("Produced")];
                    writes: [Key::at("Key"), ParentName::at("ParentName"), Value::at("Value"), Width::at("Width"), Produced::at("Produced")]; creates: [Node]; read_relations: [GeneratedBy]; links: [GeneratedBy]; }
            );
            pub type Operation = $operation;
            pub struct Binding;
            impl ApplicationMutationBinding<Schema> for Binding {
                type Input = Input; type InputBinding = InputBinding;
                type Result = (); type ResultBinding = ResultBinding;
                type IdempotencyKey = u64; type Operation = Operation;
                type Decision = Decision; type Denial = Denial; type DenialBinding = DenialBinding;
                type Output = Outputs; type ScopeBinding = Scope;
                type PrincipalBinding = PrincipalBinding; type Mapping = Mapping; type Principal = Principal;
                type PrincipalIdentity = u64; type PrincipalIdentityBinding = U64ApplicationValueBinding;
                type SourceExpectation = ApplicationQueryMutationSource<Query>;
                const IDENTITY: &'static str = $identity;
                const HANDLER_IDENTITY: &'static str = $identity;
                const IDEMPOTENCY_IDENTITY: &'static str = $identity;
                fn workflow_execution_requirements() -> ApplicationWorkflowExecutionRequirements { ApplicationWorkflowExecutionRequirements::not_applicable() }
                fn scope_field() -> Key { Key::at("Key") }
                fn principal_binding() -> ApplicationPrincipalBindingRef<Schema, PrincipalBinding, Mapping, Principal, u64, U64ApplicationValueBinding> { PrincipalBinding::reference() }
            }
            impl ApplicationMutationIntent<Schema> for Input {
                type Binding = Binding;
                fn input(&self) -> &Self { self }
                fn scope_binding(&self) -> Scope { Scope::new(Key::at("Key"), self.0.key.clone()) }
            }
            pub struct Producer;
            #[derive(Clone)]
            pub struct Provider;
            impl WorthQueryApplicationProducerBinding<Schema> for Producer {
                type Operation = Binding; type OutputFamily = Family; type Provider = Provider; type OutputRole = Output;
                const IDENTITY: &'static str = $identity;
                const APPLICABILITY: &'static [WorthQueryProducerApplicability] = &[WorthQueryProducerApplicability::new(PROFILE, WorthQueryProducerLifecyclePosture::$posture)];
                const REQUIRED_INVARIANTS: &'static [WorthQueryProducerInvariantRequirement] = &[];
                const REUSE_POLICY: &'static str = "exact-source";
            }
            impl WorthQueryApplicationProducerProvider<Schema, Producer> for Provider {
                type DenialValue = WorthQueryUndeclaredProducerDenial;
                const SEMANTIC_IDENTITY: &'static str = $identity;
                fn operation_input(&self, source: &Row) -> Input { Input(source.clone()) }
                fn idempotency_key(&self, _: &Row, identity: &[u8; 32]) -> u64 { u64::from_le_bytes(identity[..8].try_into().unwrap()) }
                fn domain_denial_reason(&self, denial: &Denial) -> Option<&'static str> {
                    Some(match denial { Denial::InitialHasPriorOutput => "InitialHasPriorOutput", Denial::PreserveMissingOutput => "PreserveMissingOutput", Denial::WrongPriorOutput => "WrongPriorOutput", Denial::WrongParent => "WrongParent" })
                }
            }
            readiness!($identity);
            pub struct Handler(pub Selections);
            impl OperationHandler<Schema, Binding> for Handler {
                fn decide(&self, input: &Input, reader: &mut DecisionReader<'_, '_, '_, Schema, Binding>) -> HandlerResult<Decision, Denial> {
                    let result = (|| {
                        let source = reader.resolve_entity(Key::at("Key"), input.0.key.clone())?;
                        if TIER < 2 { let _previous_value = reader.field(&source, Produced::at("Produced"))?; }
                        let prior_initial = reader.prior_output_if_present::<initial::Binding, Output>()?;
                        let prior_preserve = reader.prior_output_if_present::<preserve::Binding, Output>()?;
                        let prior = prior_preserve.or(prior_initial);
                        self.0.lock().unwrap().push(Selection { key: input.0.key.clone(), tier: TIER, preserve: $preserve, prior: prior.is_some() });
                        if !$preserve && prior.is_some() { return Ok(Err(Denial::InitialHasPriorOutput)); }
                        if $preserve && prior.is_none() { return Ok(Err(Denial::PreserveMissingOutput)); }
                        if prior.as_ref().is_some_and(|prior| *prior != source) { return Ok(Err(Denial::WrongPriorOutput)); }
                        let parent = reader.field(&source, ParentName::at("ParentName"))?.unwrap();
                        if parent != input.0.generated_by { return Ok(Err(Denial::WrongParent)); }
                        let custody = reader.relations_from(GeneratedBy::reference(), &source).map_err(HandlerExecutionDenial::new)?;
                        if input.0.generated_by.is_empty() {
                            if !custody.is_empty() { return Ok(Err(Denial::WrongParent)); }
                        } else {
                            let expected = reader.resolve_entity(Key::at("Key"), input.0.generated_by.clone())?;
                            if custody.len() != 1 || custody[0].to().entity_id() != expected.entity_id() { return Ok(Err(Denial::WrongParent)); }
                        }
                        let mut children = Vec::new();
                        if TIER < 2 {
                        for index in 0..input.0.width {
                            let key = format!("{}.{}", input.0.key, index);
                            let child = reader.resolve_optional_entity(Key::at("Key"), key)?;
                            if let Some(child) = &child {
                                if reader.field(child, ParentName::at("ParentName"))?.as_deref() != Some(input.0.key.as_str()) { return Ok(Err(Denial::WrongParent)); }
                            }
                            children.push(child.as_ref().map(|child| reader.mutation_target(child)).transpose()?);
                        }
                        }
                        Ok(Ok(Decision { source: reader.mutation_target(&source)?, children }))
                    })();
                    match result { Ok(Ok(target)) => HandlerResult::Completed(target), Ok(Err(denial)) => HandlerResult::DomainDenied(denial), Err(error) => HandlerResult::ExecutionDenied(error) }
                }
                fn build_candidate(&self, input: &Input, target: Decision, writer: &mut CandidateWriter<'_, Schema, Binding>) -> HandlerResult<(), Denial> {
                    let result = (|| {
                        let source = writer.projected_entity(&target.source)?;
                        if TIER < 2 { writer.write_field(&source, Produced::at("Produced"), input.0.value + 10)?; }
                        writer.preserve_output::<Output>(&source)?;
                        for (index, prior) in target.children.iter().enumerate() {
                                let key = format!("{}.{}", input.0.key, index);
                                if let Some(prior) = prior {
                                    let child = writer.projected_entity(prior)?;
                                    writer.preserve_member::<Children>(&key, &child)?;
                                    continue;
                                }
                                let child = writer.create_entity_in_context(&source, Node::reference(), WorthQueryApplicationEntityKey::new(&key).unwrap())?;
                                writer.initialize_field(&child, Key::at("Key"), key.clone())?;
                                writer.initialize_field(&child, ParentName::at("ParentName"), input.0.key.clone())?;
                                writer.initialize_field(&child, Value::at("Value"), input.0.value + 1)?;
                                writer.initialize_field(&child, Width::at("Width"), 1)?;
                                writer.link(GeneratedBy::reference(), format!("parent:{key}"), &child, &source)?;
                                writer.create_member::<Children>(&key, &child)?;
                        }
                        Ok::<_, WorthQueryApplicationAttemptDenial>(())
                    })();
                    match result { Ok(()) => HandlerResult::Completed(()), Err(error) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error)) }
                }
            }
            pub fn declare(schema: ApplicationSchemaDeclarationBuilder<Schema>) -> ApplicationSchemaDeclarationBuilder<Schema> {
                schema.declared_operation::<Operation>(Operation::reference().definition().no_external_effect().no_aftermath().finish()).application_mutation_binding::<Binding>()
            }
        }
    };
}
macro_rules! family {
    ($module:ident, $tier:expr, $profile:literal, $initial:literal, $preserve:literal, $initial_op:ident, $preserve_op:ident) => {
        pub mod $module {
            use super::*;
            pub const TIER: u8 = $tier;
            pub const PROFILE: &str = $profile;
            pub struct Outputs;
            pub struct Output;
            impl WorthQueryApplicationOutputRole for Output {
                type Schema = Schema;
                type Contract = Outputs;
                type Entity = Node;
                type Action = WorthQueryPreserveOutput;
                type Cardinality = WorthQueryExactlyOneOutput;
                const NAME: &'static str = "node";
            }
            pub struct Children;
            impl WorthQueryApplicationOutputRoleFamily for Children {
                type Schema = Schema;
                type Contract = Outputs;
                type Entity = Node;
                const PREFIX: &'static str = "child.";
                const POSTURES: ApplicationMutationOutputPostureSet =
                    ApplicationMutationOutputPostureSet::ALL;
                const MINIMUM: usize = 0;
            }
            impl ApplicationMutationOutputContract<Schema> for Outputs {
                const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] =
                    &[<Output as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR];
                const ROLE_FAMILIES: &'static [ApplicationMutationOutputRoleFamilyDescriptor] =
                    &[<Children as WorthQueryApplicationDeclaredOutputRoleFamily>::DESCRIPTOR];
            }
            pub struct Family;
            impl WorthQueryProducerOutputFamily<Schema> for Family {
                type Source = ReadBinding;
                type Entity = Node;
                const IDENTITY: &'static str = $profile;
                const SUPPORTED: &'static [WorthQueryProducerApplicability] = &[
                    WorthQueryProducerApplicability::new(
                        PROFILE,
                        WorthQueryProducerLifecyclePosture::Initial,
                    ),
                    WorthQueryProducerApplicability::new(
                        PROFILE,
                        WorthQueryProducerLifecyclePosture::Preserve,
                    ),
                ];
                fn profile_kind(_: &Row) -> &'static str {
                    PROFILE
                }
            }
            #[derive(Clone)]
            pub struct Demand(pub String);
            impl WorthQueryApplicationOutputDemand<Schema> for Demand {
                type OutputFamily = Family;
                fn source_intent(&self) -> Read {
                    Read(self.0.clone())
                }
            }
            phase!(initial, $initial_op, false, Initial, $initial);
            phase!(preserve, $preserve_op, true, Preserve, $preserve);
            pub fn declare(
                schema: ApplicationSchemaDeclarationBuilder<Schema>,
            ) -> ApplicationSchemaDeclarationBuilder<Schema> {
                preserve::declare(initial::declare(schema))
            }
            pub fn install(
                setup: &mut WorthQueryApplicationContributionSetup<'_, Schema>,
                selections: Selections,
            ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
                setup.handler::<initial::Binding, _>(initial::Handler(selections.clone()))?;
                setup.handler::<preserve::Binding, _>(preserve::Handler(selections.clone()))?;
                setup.producer::<initial::Producer>(initial::Provider)?;
                setup.producer::<preserve::Producer>(preserve::Provider)?;
                setup.conditional::<initial::Readiness>(())?;
                setup.conditional::<preserve::Readiness>(())
            }
        }
    };
}
family!(
    parent,
    0,
    "parent",
    "dispatch.parent.initial",
    "dispatch.parent.preserve",
    ParentInitial,
    ParentPreserve
);
family!(
    child,
    1,
    "child",
    "dispatch.child.initial",
    "dispatch.child.preserve",
    ChildInitial,
    ChildPreserve
);
family!(
    leaf,
    2,
    "leaf",
    "dispatch.leaf.initial",
    "dispatch.leaf.preserve",
    LeafInitial,
    LeafPreserve
);
