//! Producer families with one whole-entity realization owner each.
use super::realization::*;
use super::*;
macro_rules! split_phase {
    ($module:ident, $operation:ident, $preserve:expr, $posture:ident, $action:ty, $writer:ident, $identity:literal) => {
        pub mod $module {
            use super::*;
            #[derive(Clone, serde::Serialize)] pub struct Input(pub Row);
            worth_query_structured_value_binding!(pub InputBinding for Input { identity: $identity });
            worth_query_operation!(pub $operation for Schema, input InputBinding; access {
                read_entities: [Node, Realization]; read_fields: [Key::at("Key"), ParentName::at("ParentName"), Value::at("Value"), Width::at("Width"), RealizationKey::at("RealizationKey"), RealizedValue::at("RealizedValue")];
                read_relations: [GeneratedBy, Realizes]; writes: [Key::at("Key"), ParentName::at("ParentName"), Value::at("Value"), Width::at("Width"), RealizationKey::at("RealizationKey"), RealizedValue::at("RealizedValue")];
                creates: [Node, Realization]; links: [GeneratedBy, Realizes]; deletes: [Node, Realization]; unlinks: [GeneratedBy, Realizes];
            });
            pub type Operation = $operation;
            impl OperationReads<Operation> for OutputNext {}
            impl OperationLinks<Operation> for OutputNext {}
            impl OperationUnlinks<Operation> for OutputNext {}
            pub struct Outputs;
            pub struct Output;
            impl WorthQueryApplicationOutputRole for Output {
                type Schema = Schema; type Contract = Outputs; type Entity = Realization; type Action = $action;
                type Cardinality = WorthQueryExactlyOneOutput; const NAME: &'static str = "realization";
            }
            pub struct Children;
            impl WorthQueryApplicationOutputRoleFamily for Children {
                type Schema = Schema; type Contract = Outputs; type Entity = Node;
                const PREFIX: &'static str = "child.";
                const POSTURES: ApplicationMutationOutputPostureSet = ApplicationMutationOutputPostureSet::ALL;
                const MINIMUM: usize = 0;
            }
            pub struct RetiredRealizations;
            impl WorthQueryApplicationOutputRoleFamily for RetiredRealizations {
                type Schema = Schema; type Contract = Outputs; type Entity = Realization;
                const PREFIX: &'static str = "retired-realization.";
                const POSTURES: ApplicationMutationOutputPostureSet = ApplicationMutationOutputPostureSet::RETIRE;
                const MINIMUM: usize = 0;
            }
            impl ApplicationMutationOutputContract<Schema> for Outputs {
                const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] = &[<Output as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR];
                const ROLE_FAMILIES: &'static [ApplicationMutationOutputRoleFamilyDescriptor] = &[<Children as WorthQueryApplicationDeclaredOutputRoleFamily>::DESCRIPTOR, <RetiredRealizations as WorthQueryApplicationDeclaredOutputRoleFamily>::DESCRIPTOR];
            }
            pub struct Binding;
            impl ApplicationMutationBinding<Schema> for Binding {
                type Input = Input; type InputBinding = InputBinding; type Result = (); type ResultBinding = ResultBinding;
                type IdempotencyKey = u64; type Operation = Operation; type Decision = PublicationDecision;
                type Denial = Denial; type DenialBinding = DenialBinding; type Output = Outputs; type ScopeBinding = Scope;
                type PrincipalBinding = PrincipalBinding; type Mapping = Mapping; type Principal = Principal;
                type PrincipalIdentity = u64; type PrincipalIdentityBinding = U64ApplicationValueBinding;
                type SourceExpectation = ApplicationQueryMutationSource<SourceQuery>;
                const IDENTITY: &'static str = $identity; const HANDLER_IDENTITY: &'static str = $identity; const IDEMPOTENCY_IDENTITY: &'static str = $identity;
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
            pub struct Provider;
            impl WorthQueryApplicationProducerBinding<Schema> for Producer {
                type Operation = Binding; type OutputFamily = Family; type Provider = Provider; type OutputRole = Output;
                const IDENTITY: &'static str = $identity;
                const APPLICABILITY: &'static [WorthQueryProducerApplicability] = &[WorthQueryProducerApplicability::new(PROFILE, WorthQueryProducerLifecyclePosture::$posture)];
                const REQUIRED_INVARIANTS: &'static [WorthQueryProducerInvariantRequirement] = &[];
                const OWNERSHIP: ProducerPublicationOwnership = super::OWNERSHIP;
                const REUSE_POLICY: &'static str = "exact-source";

            }
            impl WorthQueryApplicationProducerProvider<Schema, Producer> for Provider {
                type DenialValue = WorthQueryUndeclaredProducerDenial;
                const SEMANTIC_IDENTITY: &'static str = $identity;
                fn operation_input(&self, row: &Row) -> Input { Input(row.clone()) }
                fn idempotency_key(&self, _: &Row, identity: &[u8; 32]) -> u64 { u64::from_le_bytes(identity[..8].try_into().unwrap()) }
                fn domain_denial_reason(&self, denial: &Denial) -> Option<&'static str> { Some(match denial {
                    Denial::InitialHasPriorOutput => "InitialHasPriorOutput", Denial::PreserveMissingOutput => "PreserveMissingOutput",
                    Denial::WrongPriorOutput => "WrongPriorOutput", Denial::WrongParent => "WrongParent",
                }) }
            }
            readiness!($identity, RealizedValue, realization::canonical_query, "realization");
            pub struct Handler(pub Selections, pub Fault, pub u64, pub bool);
            impl OperationHandler<Schema, Binding> for Handler {
                fn decide(&self, input: &Input, reader: &mut DecisionReader<'_, '_, '_, Schema, Binding>) -> HandlerResult<PublicationDecision, Denial> {
                    let result = (|| {
                        let source = reader.resolve_entity(Key::at("Key"), input.0.key.clone())?;
                        let prior = if $preserve { Some(reader.prior_output_preserved_or_created::<preserve::Binding, preserve::Output, initial::Binding, initial::Output>()?) }
                            else { // Initial requires authoritative live absence. Historical
                            // correspondence deliberately refuses a retired generation;
                            // it cannot answer whether a current realization exists.
                            reader.resolve_optional_entity(RealizationKey::at("RealizationKey"), super::output_key(&input.0))? };
                        self.0.lock().unwrap().push(Selection { key: input.0.key.clone(), tier: TIER, preserve: $preserve, prior: prior.is_some() });
                        if !$preserve && prior.is_some() { return Ok(Err(Denial::InitialHasPriorOutput)); }
                        if $preserve && prior.is_none() { return Ok(Err(Denial::PreserveMissingOutput)); }
                        let realization = if let Some(prior) = prior {
                            let links = reader.relations_from(Realizes::reference(), &prior).map_err(HandlerExecutionDenial::new)?;
                            if links.len() != 1 || links[0].to().entity_id() != source.entity_id() { return Ok(Err(Denial::WrongPriorOutput)); }
                            if self.3 {
                                let orbit = reader.orbit(output_next(), &prior).map_err(HandlerExecutionDenial::new)?;
                                assert_eq!(orbit.len(), 1);
                                let links = reader.relations_from(output_next(), &prior).map_err(HandlerExecutionDenial::new)?;
                                assert_eq!(links.len(), 1);
                            }
                            let _old_value = reader.field(&prior, RealizedValue::at("RealizedValue"))?;
                            Some(reader.mutation_target(&prior)?)
                        } else { None };
                        if super::super::PRIVATE_AGGREGATES && input.0.key == "observer" {
                            if self.3 && let Some(foreign) = reader.resolve_optional_entity(RealizationKey::at("RealizationKey"), "root".to_owned())? {
                                let orbit = reader.orbit(output_next(), &foreign).map_err(HandlerExecutionDenial::new)?;
                                assert_eq!(orbit.len(), 1);
                            }
                            if input.0.width == 1 {
                                let walk = reader.walk(GeneratedBy::reference(), RelationDirection::Reverse, &source).map_err(HandlerExecutionDenial::new)?;
                                assert!(!walk.is_empty());
                            }
                        }
                        let parents = reader.relations_from(GeneratedBy::reference(), &source).map_err(HandlerExecutionDenial::new)?;
                        if input.0.generated_by.is_empty() {
                            if !parents.is_empty() { return Ok(Err(Denial::WrongParent)); }
                        } else {
                            let parent = reader.resolve_entity(Key::at("Key"), input.0.generated_by.clone())?;
                            if parents.len() != 1 || parents[0].to().entity_id() != parent.entity_id() { return Ok(Err(Denial::WrongParent)); }
                        }
                        let mut children = Vec::new();
                        if TIER < 2 {
                        for index in 0..input.0.width {
                            let child = reader.resolve_optional_entity(Key::at("Key"), format!("{}.{}", input.0.key, index))?;
                            if let Some(child) = &child { reader.field(child, Value::at("Value"))?; }
                            children.push(child.as_ref().map(|child| reader.mutation_target(child)).transpose()?);
                        }
                        }
                        let retire = select_retirements!(reader, source, input.0.width);
                        Ok(Ok(PublicationDecision { source: reader.mutation_target(&source)?, realization, children, retire,
                            foreign: if matches!(self.1, Fault::Foreign | Fault::Extra | Fault::HiddenDeletion) && TIER == 0 && $preserve {
                                let output = reader.resolve_entity(RealizationKey::at("RealizationKey"), if self.1 == Fault::Foreign { "root.0" } else { "unrelated" }.to_owned())?;
                                reader.entity(&output, Realization::reference())?;
                                reader.relations_from(Realizes::reference(), &output).map_err(HandlerExecutionDenial::new)?;
                                Some(reader.mutation_target(&output)?)
                            } else { None },
                        }))
                    })();
                    match result { Ok(Ok(decision)) => HandlerResult::Completed(decision), Ok(Err(denial)) => HandlerResult::DomainDenied(denial), Err(error) => HandlerResult::ExecutionDenied(error) }
                }
                fn build_candidate(&self, input: &Input, decision: PublicationDecision, writer: &mut CandidateWriter<'_, Schema, Binding>) -> HandlerResult<(), Denial> {
                    let result = (|| {
                        let source = writer.projected_entity(&decision.source)?;
                        let output = match &decision.realization {
                            Some(prior) => { let output = writer.projected_entity(prior)?; writer.write_field(&output, RealizedValue::at("RealizedValue"), input.0.value + 10)?; output },
                            None => {
                                let output = writer.create_entity_in_context(&source, Realization::reference(), WorthQueryApplicationEntityKey::new(format!("output:{}", super::output_key(&input.0))).unwrap())?;
                                writer.initialize_field(&output, RealizationKey::at("RealizationKey"), super::output_key(&input.0))?;
                                writer.initialize_field(&output, RealizedValue::at("RealizedValue"), input.0.value + 10)?;
                                writer.link(Realizes::reference(), format!("realizes:{}", super::output_key(&input.0)), &output, &source)?;
                                output
                            }
                        };
                        if super::super::PRIVATE_AGGREGATES {
                            if self.3 || decision.realization.is_none() {
                                if decision.realization.is_some() { writer.unlink(output_next(), &output, &output)?; }
                                writer.link(output_next(), format!("next:{}", input.0.key), &output, &output)?;
                            }
                        }
                        writer.$writer::<Output>(&output)?;
                        for (index, prior) in decision.children.iter().enumerate() {
                            let key = format!("{}.{}", input.0.key, index);
                            if let Some(prior) = prior { let child = writer.projected_entity(prior)?;
                                writer.write_field(&child, Value::at("Value"), input.0.value + 1)?;
                                writer.preserve_member::<Children>(&key, &child)?; }
                            else {
                                let child = writer.create_entity_in_context(&source, Node::reference(), WorthQueryApplicationEntityKey::new(&key).unwrap())?;
                                writer.initialize_field(&child, Key::at("Key"), key.clone())?;
                                writer.initialize_field(&child, ParentName::at("ParentName"), input.0.key.clone())?;
                                writer.initialize_field(&child, Value::at("Value"), input.0.value + 1)?;
                                writer.initialize_field(&child, Width::at("Width"), self.2)?;
                                writer.link(GeneratedBy::reference(), format!("parent:{key}"), &child, &source)?;
                                writer.create_member::<Children>(&key, &child)?;
                            }
                        }
                        let fault = if TIER == 0 { self.1 } else { Fault::None };
                        write_retirements!(writer, decision.retire, fault);
                        if let Some(foreign) = decision.foreign {
                            let foreign = writer.projected_entity(&foreign)?;
                            let sibling = writer.projected_entity(decision.children[0].as_ref().unwrap())?;
                            if self.1 == Fault::Foreign { writer.unlink(Realizes::reference(), &foreign, &sibling)?; }
                            writer.delete_entity(Realization::reference(), &foreign)?;
                            if self.1 != Fault::HiddenDeletion { writer.retire_member::<RetiredRealizations>("foreign", &foreign)?; }
                        }
                        Ok::<_, WorthQueryApplicationAttemptDenial>(())
                    })();
                    match result { Ok(()) => HandlerResult::Completed(()), Err(error) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error)) }
                }
            }
            pub fn declare(schema: ApplicationSchemaDeclarationBuilder<Schema>) -> ApplicationSchemaDeclarationBuilder<Schema> {
                let schema = schema.declared_operation::<Operation>(Operation::reference().definition().no_external_effect().no_aftermath().finish()).application_mutation_binding::<Binding>();
                if super::super::PRIVATE_AGGREGATES {
                    schema.operation_read_relation(Operation::reference(), output_next())
                        .operation_link(Operation::reference(), output_next()).operation_unlink(Operation::reference(), output_next())
                } else { schema }
            }
        }
    };
}
macro_rules! split_family {
    ($module:ident, $tier:expr, $profile:literal, $initial:literal, $preserve:literal, $initial_op:ident, $preserve_op:ident, $ownership:ident $(, $prefix:literal)?) => {
        pub mod $module {
            use super::*;
            type SourceQuery = Query;
            fn output_key(row: &Row) -> String {
                let prefix = concat!("" $(, $prefix)?);
                format!("{prefix}{}", row.key)
            }
            const TIER: u8 = $tier;
            const PROFILE: &str = $profile;
            const OWNERSHIP: ProducerPublicationOwnership =
                ProducerPublicationOwnership::$ownership;
            pub struct Family;
            impl WorthQueryProducerOutputFamily<Schema> for Family {
                type Source = ReadBinding;
                type Entity = Realization;
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
            split_phase!(
                initial,
                $initial_op,
                false,
                Initial,
                WorthQueryCreateOutput,
                create_output,
                $initial
            );
            split_phase!(
                preserve,
                $preserve_op,
                true,
                Preserve,
                WorthQueryPreserveOutput,
                preserve_output,
                $preserve
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
                selections: Selections,
                fault: Fault,
                child_width: u64,
                read_predecessor_orbits: bool,
            ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
                setup.handler::<initial::Binding, _>(initial::Handler(
                    selections.clone(),
                    fault,
                    child_width,
                    read_predecessor_orbits,
                ))?;
                setup.handler::<preserve::Binding, _>(preserve::Handler(
                    selections,
                    fault,
                    child_width,
                    read_predecessor_orbits,
                ))?;
                setup.producer::<initial::Producer>(initial::Provider)?;
                setup.producer::<preserve::Producer>(preserve::Provider)?;
                setup.conditional::<initial::Readiness>(())?;
                setup.conditional::<preserve::Readiness>(())
            }
        }
    };
}
split_family!(
    parent,
    0,
    "split-parent",
    "dispatch.split.parent.initial",
    "dispatch.split.parent.preserve",
    SplitParentInitial,
    SplitParentPreserve,
    SourceScopeCascade
);
split_family!(
    child,
    1,
    "split-child",
    "dispatch.split.child.initial",
    "dispatch.split.child.preserve",
    SplitChildInitial,
    SplitChildPreserve,
    SourceScopeCascade
);
split_family!(
    leaf,
    2,
    "split-leaf",
    "dispatch.split.leaf.initial",
    "dispatch.split.leaf.preserve",
    SplitLeafInitial,
    SplitLeafPreserve,
    SourceScopeCascade
);
split_family!(
    independent_parent,
    0,
    "independent-parent",
    "dispatch.independent.parent.initial",
    "dispatch.independent.parent.preserve",
    IndependentParentInitial,
    IndependentParentPreserve,
    Independent
);

split_family!(
    twin,
    2,
    "split-twin",
    "dispatch.split.twin.initial",
    "dispatch.split.twin.preserve",
    SplitTwinInitial,
    SplitTwinPreserve,
    SourceScopeCascade,
    "twin:"
);
#[path = "route_degree/family.rs"]
pub mod parameter_routes;
