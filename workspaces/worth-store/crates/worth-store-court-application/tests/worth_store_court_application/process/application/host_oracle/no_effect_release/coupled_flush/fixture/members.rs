//! Two coupled output producers + their readiness conditionals.
use super::*;
use worth_query_decl::facade::application_operation::*;
use worth_query_decl::facade::application_schema::*;
use worth_query_decl::facade::{
    worth_query_operation, worth_query_operation_creates, worth_query_operation_reads,
    worth_query_structured_value_binding,
};
use worth_query_host::facade::{
    application_contribution::*,
    declaration::{
        authoring::*,
        binding::QueryBindingDescriptor,
        canonicalization::{canonicalize_request, CanonicalQueryBundle},
    },
    domain,
    primary_graph::*,
};

macro_rules! coupled_member {
    ($mod_name:ident, $op_name:ident, $entity:ident, $entity_name:literal, $facts_name:literal, $payload_field:ident, $tag:literal, $ready_op:literal, $mul:expr) => {
        pub mod $mod_name {
            use super::*;
            #[derive(Clone, Debug, serde::Serialize)]
            pub struct GenInput {
                pub payload: i64,
            }
            #[derive(Debug)]
            pub struct GenResult;
            #[derive(Debug)]
            pub enum GenDenial {}
            pub struct GenDecision {
                root: WorthQueryInvariantMutationTarget<Catalog, Model>,
            }
            worth_query_structured_value_binding!(pub InputBinding for GenInput { identity: concat!("worth.query.coupled-flush.", $tag, "-in.v1") });
            worth_query_structured_value_binding!(pub ResultBinding for GenResult { identity: concat!("worth.query.coupled-flush.", $tag, "-res.v1") });
            worth_query_structured_value_binding!(pub DenialBinding for GenDenial { identity: concat!("worth.query.coupled-flush.", $tag, "-den.v1") });
            worth_query_operation!(pub $op_name for Catalog, input InputBinding);
            pub type Operation = $op_name;
            worth_query_operation_reads!($op_name => [Model]);
            worth_query_operation_creates!($op_name => [$entity]);

            pub struct Outputs;
            pub struct OutputRole;
            impl WorthQueryApplicationOutputRole for OutputRole {
                type Schema = Catalog;
                type Contract = Outputs;
                type Entity = $entity;
                type Action = WorthQueryCreateOutput;
                type Cardinality = WorthQueryExactlyOneOutput;
                const NAME: &'static str = $tag;
            }
            impl ApplicationMutationOutputContract<Catalog> for Outputs {
                const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] =
                    &[<OutputRole as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR];
            }

            pub struct Binding;
            impl ApplicationMutationAdmissionBinding<Catalog> for Binding {
                type Admission = ApplicationConventionalAdmission;
            }
            impl ApplicationMutationBinding<Catalog> for Binding {
                type Input = GenInput;
                type InputBinding = InputBinding;
                type Result = GenResult;
                type ResultBinding = ResultBinding;
                type IdempotencyKey = u64;
                type Operation = Operation;
                type Decision = GenDecision;
                type Denial = GenDenial;
                type DenialBinding = DenialBinding;
                type Output = Outputs;
                type ScopeBinding = ApplicationMutationFieldScope<ModelKey>;
                type PrincipalBinding = PrincipalBinding;
                type Mapping = Mapping;
                type Principal = Principal;
                type PrincipalIdentity = u64;
                type PrincipalIdentityBinding = U64ApplicationValueBinding;
                type SourceExpectation = ApplicationQueryMutationSource<SourceQuery>;
                const IDENTITY: &'static str = concat!("worth.query.coupled-flush.", $tag, "-bind.v1");
                const HANDLER_IDENTITY: &'static str = concat!("worth.query.coupled-flush.", $tag, "-handler.v1");
                const IDEMPOTENCY_IDENTITY: &'static str = concat!("worth.query.coupled-flush.", $tag, "-idem.v1");
                const REQUIRES_APPLICATION_PROGRAM: bool = false;
                fn workflow_execution_requirements() -> ApplicationWorkflowExecutionRequirements {
                    ApplicationWorkflowExecutionRequirements::not_applicable()
                }
                fn scope_field() -> ModelKey {
                    ModelKey::at("Key")
                }
                fn principal_binding() -> ApplicationPrincipalBindingRef<
                    Catalog,
                    PrincipalBinding,
                    Mapping,
                    Principal,
                    u64,
                    U64ApplicationValueBinding,
                > {
                    principal_binding()
                }
            }
            impl ApplicationMutationIntent<Catalog> for GenInput {
                type Binding = Binding;
                fn input(&self) -> &Self {
                    self
                }
                fn scope_binding(
                    &self,
                ) -> <Binding as ApplicationMutationBinding<Catalog>>::ScopeBinding {
                    ApplicationMutationFieldScope::new(ModelKey::at("Key"), 1u64)
                }
            }

            pub struct Handler;
            impl OperationHandler<Catalog, Binding> for Handler {
                fn declared_durable_capabilities(
                ) -> Option<&'static [WorthQueryOperationDurableCapability]> {
                    Some(&[WorthQueryOperationDurableCapability::RelationalMutation])
                }
                fn decide(
                    &self,
                    _: &GenInput,
                    reader: &mut DecisionReader<'_, '_, '_, Catalog, Binding>,
                ) -> HandlerResult<GenDecision, GenDenial> {
                    match (|| {
                        let root = reader.resolve_entity(ModelKey::at("Key"), 1u64)?;
                        Ok(GenDecision {
                            root: reader.mutation_target(&root)?,
                        })
                    })() {
                        Ok(d) => HandlerResult::Completed(d),
                        Err(c) => HandlerResult::ExecutionDenied(c),
                    }
                }
                fn build_candidate(
                    &self,
                    input: &GenInput,
                    decision: GenDecision,
                    writer: &mut CandidateWriter<'_, Catalog, Binding>,
                ) -> HandlerResult<GenResult, GenDenial> {
                    let result = (|| {
                        let root = writer
                            .projected_entity(&decision.root)
                            .map_err(HandlerExecutionDenial::new)?;
                        let entity = writer
                            .create_entity_in_context(
                                &root,
                                $entity::reference(),
                                WorthQueryApplicationEntityKey::new(concat!("coupled-", $tag)).unwrap(),
                            )
                            .map_err(HandlerExecutionDenial::new)?;
                        writer
                            .initialize_field(&entity, $payload_field::at("Payload"), input.payload)
                            .map_err(HandlerExecutionDenial::new)?;
                        writer
                            .create_output::<OutputRole>(&entity)
                            .map_err(HandlerExecutionDenial::new)?;
                        Ok(GenResult)
                    })();
                    match result {
                        Ok(r) => HandlerResult::Completed(r),
                        Err(c) => HandlerResult::ExecutionDenied(c),
                    }
                }
            }

            pub struct Family;
            const INITIAL: WorthQueryProducerApplicability = WorthQueryProducerApplicability::new(
                $tag,
                WorthQueryProducerLifecyclePosture::Initial,
            );
            impl WorthQueryProducerOutputFamily<Catalog> for Family {
                type Source = SourceBinding;
                type Entity = $entity;
                const IDENTITY: &'static str = concat!("worth.query.coupled-flush.", $tag, "-family.v1");
                const SUPPORTED: &'static [WorthQueryProducerApplicability] = &[INITIAL];
                fn profile_kind(_: &SourceRow) -> &'static str {
                    $tag
                }
            }
            #[derive(Clone)]
            pub struct Demand;
            impl WorthQueryApplicationOutputDemand<Catalog> for Demand {
                type OutputFamily = Family;
                fn source_intent(&self) -> ReadSource {
                    ReadSource
                }
            }
            pub struct Provider;
            pub struct Producer;
            impl WorthQueryApplicationProducerProvider<Catalog, Producer> for Provider {
                type DenialValue = WorthQueryUndeclaredProducerDenial;
                const SEMANTIC_IDENTITY: &'static str = concat!("worth.query.coupled-flush.", $tag, "-provider.v1");
                fn operation_input(&self, source: &SourceRow) -> GenInput {
                    GenInput {
                        payload: source.data * $mul,
                    }
                }
                fn idempotency_key(&self, _: &SourceRow, source_identity: &[u8; 32]) -> u64 {
                    u64::from_le_bytes(source_identity[..8].try_into().unwrap())
                }
            }
            impl WorthQueryApplicationProducerBinding<Catalog> for Producer {
                type Operation = Binding;
                type OutputFamily = Family;
                type Provider = Provider;
                type OutputRole = OutputRole;
                const IDENTITY: &'static str = concat!("worth.query.coupled-flush.", $tag, "-producer.v1");
                const APPLICABILITY: &'static [WorthQueryProducerApplicability] = &[INITIAL];
                const REQUIRED_INVARIANTS: &'static [WorthQueryProducerInvariantRequirement] = &[];
                const REUSE_POLICY: &'static str = "exact-source";
                const INPUT_REUSE: Option<WorthQueryProducerInputReuseContract> =
                    Some(WorthQueryProducerInputReuseContract::canonical_bitwise(
                        WorthQueryDecisionContextDependencies::NONE,
                    ));
            }

            #[derive(Clone, Copy, Debug, Eq, PartialEq)]
            pub struct ReadyDomain;
            #[derive(Clone, Copy, Debug, Eq, PartialEq)]
            pub struct ReadyOp;
            #[derive(Clone, Copy, Debug, Eq, PartialEq)]
            pub struct ReadyFamily;
            worth_query_host::facade::worth_query_conditional_node!(
                pub ReadyNode in ReadyDomain, ReadyOp, ReadyFamily => operation $ready_op
            );
            pub struct Readiness;
            type Contract = WorthQueryOutputReadinessContract<ReadyDomain, ReadyOp, ReadyFamily>;
            type ReadyBind = domain::WorthQueryApplicationConditionalOperationBinding<
                Catalog, Operation, GenInput, ReadyDomain, ReadyOp, ReadyFamily,
            >;
            fn contract(
                declaration: &ErasedApplicationSchemaDeclaration,
            ) -> Result<Contract, WorthQueryOutputReadinessContractDenial> {
                WorthQueryOutputReadinessContractBuilder::for_field(
                    declaration,
                    $payload_field::at("Payload"),
                    domain::WorthQueryDomainOperationIdentity::new(concat!("coupled-flush-", $tag), 1),
                    concat!("coupled-flush-", $tag, "-ready"),
                    canonical_query(),
                    domain::WorthQueryOperationProjectionRole::new(concat!("coupled-", $tag)).unwrap(),
                    domain::WorthQueryExecutionStrategyName::new(concat!("coupled-flush-", $tag)).unwrap(),
                    concat!("coupled-flush-", $tag, "-v1"),
                )?
                .field_dependency(
                    declaration,
                    $payload_field::at("Payload"),
                    WorthQueryReadinessFieldUse::DeliveryAndEligibility,
                )?
                .build_typed()
            }
            fn canonical_query() -> CanonicalQueryBundle {
                let query = DetailQueryBuilder::new(RootEntityKey::new($entity_name).unwrap())
                    .project(AspectFieldSelector::new($facts_name, "Payload").unwrap())
                    .build().unwrap().into_raw();
                let shape = DetailResultShapeBuilder::new()
                    .field(AuthoredResultShapeField::new($facts_name, "Payload", "payload").unwrap())
                    .build().unwrap().into_raw();
                canonicalize_request(
                    AuthoredQueryBundleRequest::for_ordinary_read(query, shape, QueryBindingDescriptor::new()).unwrap(),
                ).unwrap()
            }
            impl WorthQueryApplicationConditionalBinding<Catalog> for Readiness {
                type Configuration = ();
                type Installed = ();
                type Operation = Operation;
                const IDENTITY: &'static str = concat!("worth.query.coupled-flush.", $tag, "-readiness.v1");
                const REQUIRED_PRODUCERS: &'static [&'static str] =
                    &[<Producer as WorthQueryApplicationProducerBinding<Catalog>>::IDENTITY];
                fn package_contract(
                    declaration: &ErasedApplicationSchemaDeclaration,
                ) -> Result<WorthQueryApplicationConditionalPackageContract, WorthQueryOutputReadinessContractDenial> {
                    let c = contract(declaration)?;
                    let b = ReadyBind::declare(Operation::reference(), c.definition().reference());
                    Ok(c.into_package_contract(b.portable().clone()))
                }
                fn install(
                    _: (),
                    _: &WorthQueryApplicationConditionalProducerAccess<'_, Catalog>,
                    installation: &mut WorthQueryConditionalApplicationRuntimeInstallation<Catalog>,
                ) -> Result<(), WorthQueryConditionalRuntimeInstallationDenial> {
                    let c = contract(installation.installed_schema().installed_declaration()).unwrap();
                    let op = installation.installed_schema().installed_operation(Operation::reference()).unwrap();
                    let b = ReadyBind::declare(Operation::reference(), c.definition().reference());
                    let node = installation
                        .installed_packages()
                        .bind_conditional_application_operation(op, &b).unwrap()
                        .bind_node(ReadyNode::reference()).unwrap();
                    let ord = c.delivery_ordinal(installation.installed_schema(), node.declaration(), $payload_field::at("Payload")).unwrap();
                    installation.bind_output_readiness::<Producer, _, _, _, _, _, _>(node, ord)
                }
            }
            pub fn register(
                builder: ApplicationSchemaDeclarationBuilder<Catalog>,
            ) -> ApplicationSchemaDeclarationBuilder<Catalog> {
                let op = Operation::reference();
                builder
                    .operation(op.definition().no_external_effect().no_aftermath().finish())
                    .operation_projection_work_budget(op, 64)
                    .operation_read_entity(op, Model::reference())
                    .operation_read_field(op, ModelKey::at("Key"))
                    .operation_create(op, $entity::reference())
                    .operation_write(op, $payload_field::at("Payload"))
                    .application_mutation_binding::<Binding>()
            }
        }
    };
}

coupled_member!(first, FirstOp, ArtifactA, "ArtifactA", "ArtifactAFacts", PayloadA, "first", "coupled-flush-first-ready", 10);
coupled_member!(second, SecondOp, ArtifactB, "ArtifactB", "ArtifactBFacts", PayloadB, "second", "coupled-flush-second-ready", 20);
