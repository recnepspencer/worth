//! Two model-scoped families, each split into Initial and Preserve producers that
//! publish exactly one derived output owned by its source model (SourceScopeCascade).
use super::*;
#[derive(Clone, Debug)]
pub enum Denial {
    InitialHasPriorOutput,
    PreserveMissingOutput,
}
worth_query_structured_value_binding!(pub DenialBinding for Denial { identity: "owned-outputs.producer-denial.v1" });
worth_query_structured_value_binding!(pub ResultBinding for () { identity: "owned-outputs.producer-result.v1" });
pub type Scope = ApplicationMutationFieldScope<
    Schema,
    Model,
    ModelFacts,
    ModelKey,
    String,
    ReadOnly,
    NoApplicationUnit,
>;
pub type Request<'a, 'p, 's> = WorthQueryApplicationRequest<'a, 'p, 's, Schema>;
macro_rules! phase {
    ($module:ident, $operation:ident, $preserve:expr, $posture:ident, $action:ty, $writer:ident, $identity:literal,
     $entity:ident, $facts:ident, $key:ident, $value:ident, $of:ident) => {
        pub mod $module {
            use super::*;
            #[derive(Clone, serde::Serialize)] pub struct Input(pub query::Row);
            worth_query_structured_value_binding!(pub InputBinding for Input { identity: $identity });
            worth_query_operation!(pub $operation for Schema, input InputBinding; access {
                read_entities: [Model, $entity]; read_fields: [ModelKey, ModelValue, $key, $value];
                read_relations: [$of]; writes: [$key, $value]; creates: [$entity]; links: [$of];
            });
            pub type Operation = $operation;
            pub struct Outputs;
            pub struct Output;
            impl WorthQueryApplicationOutputRole for Output {
                type Schema = Schema; type Contract = Outputs; type Entity = $entity; type Action = $action;
                type Cardinality = WorthQueryExactlyOneOutput; const NAME: &'static str = "derived";
            }
            impl ApplicationMutationOutputContract<Schema> for Outputs {
                const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] = &[<Output as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR];
                const ROLE_FAMILIES: &'static [ApplicationMutationOutputRoleFamilyDescriptor] = &[];
            }
            pub struct Decision { source: WorthQueryInvariantMutationTarget<Schema, Model>, output: Option<WorthQueryInvariantMutationTarget<Schema, $entity>> }
            pub struct Binding;
            impl ApplicationMutationBinding<Schema> for Binding {
                type Input = Input; type InputBinding = InputBinding; type Result = (); type ResultBinding = ResultBinding;
                type IdempotencyKey = u64; type Operation = Operation; type Decision = Decision;
                type Denial = Denial; type DenialBinding = DenialBinding; type Output = Outputs; type ScopeBinding = Scope;
                type PrincipalBinding = Actor; type Mapping = Mapping; type Principal = Principal;
                type PrincipalIdentity = u64; type PrincipalIdentityBinding = U64ApplicationValueBinding;
                type SourceExpectation = ApplicationQueryMutationSource<query::model::Query>;
                const IDENTITY: &'static str = $identity; const HANDLER_IDENTITY: &'static str = $identity; const IDEMPOTENCY_IDENTITY: &'static str = $identity;
                fn workflow_execution_requirements() -> ApplicationWorkflowExecutionRequirements { ApplicationWorkflowExecutionRequirements::not_applicable() }
                fn scope_field() -> ApplicationFieldRef<Schema, Model, ModelFacts, ModelKey, String, ReadOnly, EqualityPredicate, NoApplicationUnit> { ModelKey::reference() }
                fn principal_binding() -> ApplicationPrincipalBindingRef<Schema, Actor, Mapping, Principal, u64, U64ApplicationValueBinding> { Actor::reference() }
            }
            impl ApplicationMutationIntent<Schema> for Input {
                type Binding = Binding;
                fn input(&self) -> &Self { self }
                fn scope_binding(&self) -> Scope { Scope::new(ModelKey::reference(), self.0.key.clone()) }
            }
            pub struct Producer;
            pub struct Provider;
            impl WorthQueryApplicationProducerBinding<Schema> for Producer {
                type Operation = Binding; type OutputFamily = Family; type Provider = Provider; type OutputRole = Output;
                const IDENTITY: &'static str = $identity;
                const APPLICABILITY: &'static [WorthQueryProducerApplicability] = &[WorthQueryProducerApplicability::new(PROFILE, WorthQueryProducerLifecyclePosture::$posture)];
                const REQUIRED_INVARIANTS: &'static [WorthQueryProducerInvariantRequirement] = &[];
                const OWNERSHIP: ProducerPublicationOwnership = ProducerPublicationOwnership::SourceScopeCascade;
                const REUSE_POLICY: &'static str = "exact-source";
            }
            impl WorthQueryApplicationProducerProvider<Schema, Producer> for Provider {
                type DenialValue = WorthQueryUndeclaredProducerDenial;
                const SEMANTIC_IDENTITY: &'static str = $identity;
                fn operation_input(&self, row: &query::Row) -> Input { Input(row.clone()) }
                fn idempotency_key(&self, _: &query::Row, identity: &[u8; 32]) -> u64 { u64::from_le_bytes(identity[..8].try_into().unwrap()) }
                fn domain_denial_reason(&self, denial: &Denial) -> Option<&'static str> { Some(match denial {
                    Denial::InitialHasPriorOutput => "InitialHasPriorOutput", Denial::PreserveMissingOutput => "PreserveMissingOutput",
                }) }
            }
            use worth_query_host::facade::domain;
            #[derive(Clone, Copy, Debug, Eq, PartialEq)] pub struct ReadyDomain;
            #[derive(Clone, Copy, Debug, Eq, PartialEq)] pub struct ReadyOperation;
            #[derive(Clone, Copy, Debug, Eq, PartialEq)] pub struct ReadyFamily;
            worth_query_host::facade::worth_query_conditional_node!(pub ReadyNode in ReadyDomain, ReadyOperation, ReadyFamily => operation $identity);
            pub struct Readiness;
            type Contract = WorthQueryOutputReadinessContract<ReadyDomain, ReadyOperation, ReadyFamily>;
            fn readiness_contract(declaration: &ErasedApplicationSchemaDeclaration) -> Result<Contract, WorthQueryOutputReadinessContractDenial> {
                WorthQueryOutputReadinessContractBuilder::for_field(declaration, $value::reference(),
                    domain::WorthQueryDomainOperationIdentity::new($identity, 1), $identity,
                    readiness_query(stringify!($entity), stringify!($facts), stringify!($value)),
                    domain::WorthQueryOperationProjectionRole::new("derived").unwrap(),
                    domain::WorthQueryExecutionStrategyName::new($identity).unwrap(), $identity)?
                    .field_dependency(declaration, $value::reference(), WorthQueryReadinessFieldUse::DeliveryAndEligibility)?.build_typed()
            }
            fn conditional_binding(contract: &Contract) -> domain::WorthQueryApplicationConditionalOperationBinding<Schema, Operation, Input, ReadyDomain, ReadyOperation, ReadyFamily> {
                domain::WorthQueryApplicationConditionalOperationBinding::declare(Operation::reference(), contract.definition().reference())
            }
            impl WorthQueryApplicationConditionalBinding<Schema> for Readiness {
                type Configuration = (); type Installed = (); type Operation = Operation;
                const IDENTITY: &'static str = $identity;
                const REQUIRED_PRODUCERS: &'static [&'static str] = &[$identity];
                fn package_contract(declaration: &ErasedApplicationSchemaDeclaration) -> Result<WorthQueryApplicationConditionalPackageContract, WorthQueryOutputReadinessContractDenial> {
                    let contract = readiness_contract(declaration)?;
                    let binding = conditional_binding(&contract);
                    Ok(contract.into_package_contract(binding.portable().clone()))
                }
                fn install(_: (), _: &WorthQueryApplicationConditionalProducerAccess<'_, Schema>, installation: &mut WorthQueryConditionalApplicationRuntimeInstallation<Schema>) -> Result<(), WorthQueryConditionalRuntimeInstallationDenial> {
                    let contract = readiness_contract(Schema::declaration().unwrap().erased()).unwrap();
                    let binding = conditional_binding(&contract);
                    let operation = installation.installed_schema().installed_operation(Operation::reference()).unwrap();
                    let node = installation.installed_packages().bind_conditional_application_operation(operation, &binding).unwrap().bind_node(ReadyNode::reference()).unwrap();
                    let ordinal = contract.delivery_ordinal(installation.installed_schema(), node.declaration(), $value::reference()).unwrap();
                    installation.bind_output_readiness::<Producer, _, _, _, _, _, _>(node, ordinal)
                }
            }
            pub struct Handler;
            impl OperationHandler<Schema, Binding> for Handler {
                fn decide(&self, input: &Input, reader: &mut DecisionReader<'_, '_, '_, Schema, Binding>) -> HandlerResult<Decision, Denial> {
                    let result = (|| {
                        let source = reader.resolve_entity(ModelKey::reference(), input.0.key.clone())?;
                        reader.field(&source, ModelValue::reference())?;
                        let prior = if $preserve { Some(reader.prior_output_preserved_or_created::<preserve::Binding, preserve::Output, initial::Binding, initial::Output>()?) }
                            else { reader.resolve_optional_entity($key::reference(), output_key(&input.0))? };
                        if !$preserve && prior.is_some() { return Ok(Err(Denial::InitialHasPriorOutput)); }
                        if $preserve && prior.is_none() { return Ok(Err(Denial::PreserveMissingOutput)); }
                        let output = match prior {
                            Some(prior) => { reader.field(&prior, $value::reference())?; Some(reader.mutation_target(&prior)?) }
                            None => None,
                        };
                        Ok(Ok(Decision { source: reader.mutation_target(&source)?, output }))
                    })();
                    match result { Ok(Ok(decision)) => HandlerResult::Completed(decision), Ok(Err(denial)) => HandlerResult::DomainDenied(denial), Err(error) => HandlerResult::ExecutionDenied(error) }
                }
                fn build_candidate(&self, input: &Input, decision: Decision, writer: &mut CandidateWriter<'_, Schema, Binding>) -> HandlerResult<(), Denial> {
                    let result = (|| {
                        let source = writer.projected_entity(&decision.source)?;
                        let value = derived(input.0.value);
                        let output = match &decision.output {
                            Some(prior) => { let output = writer.projected_entity(prior)?; writer.write_field(&output, $value::reference(), value)?; output }
                            None => {
                                let key = output_key(&input.0);
                                let output = writer.create_entity_in_context(&source, $entity::reference(), WorthQueryApplicationEntityKey::new(key.clone()).unwrap())?;
                                writer.initialize_field(&output, $key::reference(), key.clone())?;
                                writer.initialize_field(&output, $value::reference(), value)?;
                                writer.link($of::reference(), format!("of:{key}"), &output, &source)?;
                                output
                            }
                        };
                        writer.$writer::<Output>(&output)?;
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
fn readiness_query(
    entity: &str,
    aspect: &str,
    field: &str,
) -> worth_query_decl::facade::canonicalization::CanonicalQueryBundle {
    use worth_query_decl::facade::{
        authoring::*, binding::QueryBindingDescriptor, canonicalization::canonicalize_request,
    };
    let query = DetailQueryBuilder::new(RootEntityKey::new(entity).unwrap())
        .project(AspectFieldSelector::new(aspect, field).unwrap())
        .build()
        .unwrap()
        .into_raw();
    let shape = DetailResultShapeBuilder::new()
        .field(AuthoredResultShapeField::new(aspect, field, "value").unwrap())
        .build()
        .unwrap()
        .into_raw();
    canonicalize_request(
        AuthoredQueryBundleRequest::for_ordinary_read(query, shape, QueryBindingDescriptor::new())
            .unwrap(),
    )
    .unwrap()
}
macro_rules! family {
    ($module:ident, $profile:literal, $initial:literal, $preserve:literal, $initial_op:ident, $preserve_op:ident,
     $entity:ident, $facts:ident, $key:ident, $value:ident, $of:ident, $tree:ident, $derive:expr) => {
        pub mod $module {
            use super::*;
            const PROFILE: &str = $profile;
            pub fn output_key(row: &query::Row) -> String {
                format!("{}:{}", PROFILE, row.key)
            }
            pub fn derived(value: u64) -> u64 {
                ($derive)(value)
            }
            pub struct Family;
            impl WorthQueryProducerOutputFamily<Schema> for Family {
                type Source = query::model::Binding;
                type Entity = $entity;
                const IDENTITY: &'static str = PROFILE;
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
                fn profile_kind(_: &query::Row) -> &'static str {
                    PROFILE
                }
            }
            #[derive(Clone)]
            pub struct Demand(pub String);
            impl WorthQueryApplicationOutputDemand<Schema> for Demand {
                type OutputFamily = Family;
                fn source_intent(&self) -> query::model::Read {
                    query::model::Read(self.0.clone())
                }
            }
            phase!(
                initial,
                $initial_op,
                false,
                Initial,
                WorthQueryCreateOutput,
                create_output,
                $initial,
                $entity,
                $facts,
                $key,
                $value,
                $of
            );
            phase!(
                preserve,
                $preserve_op,
                true,
                Preserve,
                WorthQueryPreserveOutput,
                preserve_output,
                $preserve,
                $entity,
                $facts,
                $key,
                $value,
                $of
            );
            pub fn declare(
                schema: ApplicationSchemaDeclarationBuilder<Schema>,
            ) -> ApplicationSchemaDeclarationBuilder<Schema> {
                preserve::declare(initial::declare(schema))
            }
            pub fn contracts(
                contracts: &mut WorthQueryApplicationContributionContracts<Schema>,
            ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
                contracts.producer::<initial::Producer>()?;
                contracts.producer::<preserve::Producer>()?;
                contracts.conditional::<initial::Readiness>()?;
                contracts.conditional::<preserve::Readiness>()?;
                Ok(())
            }
            pub fn install(
                setup: &mut WorthQueryApplicationContributionSetup<'_, Schema>,
            ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
                setup.handler::<initial::Binding, _>(initial::Handler)?;
                setup.handler::<preserve::Binding, _>(preserve::Handler)?;
                setup.producer::<initial::Producer>(initial::Provider)?;
                setup.producer::<preserve::Producer>(preserve::Provider)?;
                setup.conditional::<initial::Readiness>(())?;
                setup.conditional::<preserve::Readiness>(())
            }
            /// Demand and settle this family for one model on the request's branch.
            pub fn settle(host: &world::Host, request: &Request<'_, '_, '_>, model: &str) {
                let mut operation = request
                    .demand(Demand(model.into()))
                    .retain_program_outputs::<program::Program, program::$tree>(host)
                    .unwrap_or_else(|e| panic!("{:?}", e.denial()));
                for _ in 0..128 {
                    match operation.advance(request) {
                        WorthQueryProgramOutputOperationAdvance::Pending(next) => operation = next,
                        WorthQueryProgramOutputOperationAdvance::Settled(settled) => {
                            settled.deliver_current(request).unwrap();
                            settled
                                .close(request)
                                .unwrap_or_else(|e| panic!("{:?}", e.denial()));
                            return;
                        }
                        WorthQueryProgramOutputOperationAdvance::Refused { denial, .. } => {
                            panic!("{denial:?}")
                        }
                        WorthQueryProgramOutputOperationAdvance::Terminal(t) => {
                            panic!("{:?}", t.denial())
                        }
                    }
                }
                panic!("{} did not settle", PROFILE);
            }
        }
    };
}
family!(
    first,
    "first",
    "owned-outputs.first.initial",
    "owned-outputs.first.preserve",
    FirstInitial,
    FirstPreserve,
    FirstOutput,
    FirstFacts,
    FirstKey,
    FirstValue,
    FirstOf,
    FirstTree,
    |value: u64| value * 10
);
family!(
    second,
    "second",
    "owned-outputs.second.initial",
    "owned-outputs.second.preserve",
    SecondInitial,
    SecondPreserve,
    SecondOutput,
    SecondFacts,
    SecondKey,
    SecondValue,
    SecondOf,
    SecondTree,
    |value: u64| value + 1000
);
