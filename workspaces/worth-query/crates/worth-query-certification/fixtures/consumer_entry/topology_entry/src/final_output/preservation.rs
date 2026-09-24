use super::*;
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationOutputRoleFamily, WorthQueryCreateOutput,
    WorthQueryInvariantMutationTarget, WorthQueryPreserveOutput,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FinalPlanarPreserve {
    pub scope_key: String,
    pub output_key: String,
    pub value: worth_query_consumer_values::PositiveLength,
}

worth_query_structured_value_binding!(pub FinalPlanarPreserveInputBinding for FinalPlanarPreserve {
    identity: "worth.query.certification.final-planar-preserve-input.v1"
});
worth_query_operation!(pub PreserveFinalPlanarOutput for Schema: TopologySchemaBinding, input FinalPlanarPreserveInputBinding);
worth_query_operation_reads!(PreserveFinalPlanarOutput => [Body, BodyKey, PositionY, Length, PlanarSuccessor]);
worth_query_operation_writes!(PreserveFinalPlanarOutput => [PositionY, Length]);

pub struct FinalPlanarPreserveOutputs;

impl<Schema: TopologySchemaBinding> ApplicationMutationOutputContract<Schema>
    for FinalPlanarPreserveOutputs
{
    const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] =
        &[ApplicationMutationOutputRoleDescriptor::for_entity::<
            Schema,
            Body,
        >(
            "anchor", ApplicationMutationOutputPosture::Preserve
        )];
    const ROLE_FAMILIES: &'static [ApplicationMutationOutputRoleFamilyDescriptor] =
        &[ApplicationMutationOutputRoleFamilyDescriptor::for_entity::<
            Schema,
            Body,
        >(
            "preserved.",
            ApplicationMutationOutputPostureSet::PRESERVE,
            0,
        )];
}

pub struct FinalPlanarPreserveBinding<Schema>(PhantomData<fn() -> Schema>);

impl<Schema: TopologySchemaBinding> ApplicationMutationBinding<Schema>
    for FinalPlanarPreserveBinding<Schema>
{
    type Input = FinalPlanarPreserve;
    type InputBinding = FinalPlanarPreserveInputBinding;
    type Result = worth_query_consumer_values::PlanarAdjustmentResult;
    type ResultBinding = PlanarMutationResultBinding;
    type IdempotencyKey = u64;
    type Operation = PreserveFinalPlanarOutput;
    type Decision = [WorthQueryInvariantMutationTarget<Schema, Body>; 3];
    type Denial = worth_query_consumer_values::PlanarMutationDenial;
    type DenialBinding = PlanarMutationDenialBinding;
    type Output = FinalPlanarPreserveOutputs;
    type ScopeBinding = PlanarMutationScope<Schema>;
    type PrincipalBinding = ConsumerPrincipalBinding;
    type Mapping = ExternalPrincipalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type SourceExpectation = ApplicationQueryMutationSource<PlanarQuery>;

    const IDENTITY: &'static str = "worth.query.certification.final-planar-preserve.v1";
    const HANDLER_IDENTITY: &'static str =
        "worth.query.certification.final-planar-preserve-handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str =
        "worth.query.certification.final-planar-preserve-command.v1";
    const CANDIDATES: ApplicationCandidateRequirements =
        super::super::requirements(0, 0, 0, 6, 4096, 4096);

    fn idempotency_key_identity(key: &u64) -> [u8; 32] {
        super::super::mutation_identity::key_identity(*key)
    }

    fn input_identity(input: &FinalPlanarPreserve) -> [u8; 32] {
        super::super::mutation_identity::input_identity(&PlanarMutation {
            scope_key: input.scope_key.clone(),
            operation: PlanarOperation::PublishDerivedOutput(PlanarDerivedOutput {
                body_key: input.output_key.clone(),
                value: input.value,
            }),
            validator_work: 4_096,
        })
    }

    fn scope_field() -> ApplicationFieldRef<
        Schema,
        Body,
        PlanarPosition,
        BodyKey,
        String,
        ReadOnly,
        EqualityPredicate,
        NoApplicationUnit,
    > {
        BodyKey::reference()
    }

    fn principal_binding() -> ApplicationPrincipalBindingRef<
        Schema,
        ConsumerPrincipalBinding,
        ExternalPrincipalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        ConsumerPrincipalBinding::reference()
    }
}

impl<Schema: TopologySchemaBinding> ApplicationMutationIntent<Schema> for FinalPlanarPreserve {
    type Binding = FinalPlanarPreserveBinding<Schema>;

    fn input(&self) -> &Self {
        self
    }

    fn scope_binding(&self) -> PlanarMutationScope<Schema> {
        PlanarMutationScope::new(BodyKey::reference(), self.scope_key.clone())
    }
}

pub struct FinalPlanarPreserveHandler;

impl<Schema: TopologySchemaBinding> OperationHandler<Schema, FinalPlanarPreserveBinding<Schema>>
    for FinalPlanarPreserveHandler
{
    fn decide(
        &self,
        input: &FinalPlanarPreserve,
        reader: &mut DecisionReader<'_, '_, '_, Schema, FinalPlanarPreserveBinding<Schema>>,
    ) -> HandlerResult<
        [WorthQueryInvariantMutationTarget<Schema, Body>; 3],
        worth_query_consumer_values::PlanarMutationDenial,
    > {
        let preserve_family = reader
            .prior_output_family_if_present::<FinalPlanarPreserveBinding<Schema>, Body>(
                WorthQueryApplicationOutputRoleFamily::from_static("preserved."),
            );
        let anchor = match preserve_family {
            Ok(Some(_)) => reader
                .prior_output::<FinalPlanarPreserveBinding<Schema>, Body, WorthQueryPreserveOutput>(
                    WorthQueryApplicationOutputRole::from_static("anchor"),
                ),
            Ok(None) => {
                match reader.prior_output_family_if_present::<FinalPlanarMutationBinding<Schema>, Body>(
                    WorthQueryApplicationOutputRoleFamily::from_static("created."),
                ) {
                    Ok(Some(_)) => reader.prior_output::<FinalPlanarMutationBinding<Schema>, Body, WorthQueryCreateOutput>(
                        WorthQueryApplicationOutputRole::from_static("anchor"),
                    ),
                    Ok(None) => return HandlerResult::DomainDenied(
                        worth_query_consumer_values::PlanarMutationDenial::CurrentOutputMissing,
                    ),
                    Err(error) => return HandlerResult::ExecutionDenied(error),
                }
            }
            Err(error) => return HandlerResult::ExecutionDenied(error),
        };
        let anchor = match anchor {
            Ok(anchor) => anchor,
            Err(error) => return HandlerResult::ExecutionDenied(error),
        };
        let second = match reader.related_one(PlanarSuccessor::reference(), &anchor) {
            Ok(second) => second,
            Err(error) => {
                return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error))
            }
        };
        let third = match reader.related_one(PlanarSuccessor::reference(), &second) {
            Ok(third) => third,
            Err(error) => {
                return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error))
            }
        };
        let closure = match reader.related_one(PlanarSuccessor::reference(), &third) {
            Ok(closure) => closure,
            Err(error) => {
                return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error))
            }
        };
        if closure != anchor {
            return HandlerResult::DomainDenied(
                worth_query_consumer_values::PlanarMutationDenial::UnexpectedSuccessor,
            );
        }
        let entities = [anchor, second, third];
        let keys = [
            input.output_key.clone(),
            format!("{}:b", input.output_key),
            format!("{}:c", input.output_key),
        ];
        for (entity, key) in entities.iter().zip(&keys) {
            match reader.field(entity, BodyKey::reference()) {
                Ok(Some(actual)) if actual == *key => {}
                Ok(_) => {
                    return HandlerResult::DomainDenied(
                        worth_query_consumer_values::PlanarMutationDenial::UnexpectedCurrentOutput,
                    )
                }
                Err(error) => return HandlerResult::ExecutionDenied(error),
            }
            for result in [
                reader.field(entity, PositionY::reference()).map(|_| ()),
                reader.field(entity, Length::reference()).map(|_| ()),
            ] {
                if let Err(error) = result {
                    return HandlerResult::ExecutionDenied(error);
                }
            }
        }
        let targets = entities.map(|entity| reader.mutation_target(&entity));
        match targets {
            [Ok(first), Ok(second), Ok(third)] => HandlerResult::Completed([first, second, third]),
            [Err(error), _, _] | [_, Err(error), _] | [_, _, Err(error)] => {
                HandlerResult::ExecutionDenied(error)
            }
        }
    }

    fn candidate_requirements(
        &self,
        _: &FinalPlanarPreserve,
        _: &[WorthQueryInvariantMutationTarget<Schema, Body>; 3],
    ) -> ApplicationCandidateRequirements {
        FinalPlanarPreserveBinding::<Schema>::CANDIDATES
    }

    fn build_candidate(
        &self,
        input: &FinalPlanarPreserve,
        decision: [WorthQueryInvariantMutationTarget<Schema, Body>; 3],
        writer: &mut CandidateWriter<'_, Schema, FinalPlanarPreserveBinding<Schema>>,
    ) -> HandlerResult<
        worth_query_consumer_values::PlanarAdjustmentResult,
        worth_query_consumer_values::PlanarMutationDenial,
    > {
        let base_y = worth_query_consumer_values::PositiveLength::get(&input.value);
        let Some(next_y) = base_y.checked_add(1) else {
            return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(
                std::io::Error::other("final output ordinate exceeds its value binding"),
            ));
        };
        let keys = [
            input.output_key.clone(),
            format!("{}:b", input.output_key),
            format!("{}:c", input.output_key),
        ];
        for (index, target) in decision.iter().enumerate() {
            let entity = match writer.projected_entity(target) {
                Ok(entity) => entity,
                Err(error) => {
                    return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error))
                }
            };
            let y = if index == 2 { next_y } else { base_y };
            for result in [
                writer.write_field(
                    &entity,
                    PositionY::reference(),
                    worth_query_consumer_values::PositiveLength::new(y).unwrap(),
                ),
                writer.write_field(&entity, Length::reference(), input.value),
            ] {
                if let Err(error) = result {
                    return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error));
                }
            }
            let role = if index == 0 {
                WorthQueryApplicationOutputRole::<
                    FinalPlanarPreserveBinding<Schema>,
                    Body,
                    worth_query_host::facade::primary_graph::WorthQueryPreserveOutput,
                >::try_new("anchor")
            } else {
                WorthQueryApplicationOutputRole::<
                    FinalPlanarPreserveBinding<Schema>,
                    Body,
                    worth_query_host::facade::primary_graph::WorthQueryPreserveOutput,
                >::try_new(format!("preserved.{}", keys[index]))
            }
            .expect("the preserved role is declared");
            if let Err(error) = writer.preserve_output(role, &entity) {
                return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error));
            }
        }
        HandlerResult::Completed(worth_query_consumer_values::PlanarAdjustmentResult {
            changed_vertices: 1,
        })
    }
}

pub struct PlanarFinalPreserveProducer<Schema>(PhantomData<fn() -> Schema>);

impl<Schema: TopologySchemaBinding>
    WorthQueryApplicationProducerProvider<Schema, PlanarFinalPreserveProducer<Schema>>
    for PlanarFinalOutputProvider
{
    const SEMANTIC_IDENTITY: &'static str =
        "worth.query.certification.planar-final-preserve-provider.v1";

    fn operation_input(&self, source: &PlanarReadResult) -> FinalPlanarPreserve {
        let initial = <Self as WorthQueryApplicationProducerProvider<
            Schema,
            PlanarFinalOutputProducer<Schema>,
        >>::operation_input(self, source);
        FinalPlanarPreserve {
            scope_key: initial.scope_key,
            output_key: initial.output_key,
            value: initial.value,
        }
    }

    fn idempotency_key(&self, source: &PlanarReadResult, identity: &[u8; 32]) -> u64 {
        <Self as WorthQueryApplicationProducerProvider<Schema, PlanarFinalOutputProducer<Schema>>>::idempotency_key(self, source, identity)
    }

    fn demand_resources(&self, source: &PlanarReadResult) -> WorthQueryProducerDemandResources {
        <Self as WorthQueryApplicationProducerProvider<Schema, PlanarFinalOutputProducer<Schema>>>::demand_resources(self, source)
    }
}

impl<Schema: TopologySchemaBinding> WorthQueryApplicationProducerBinding<Schema>
    for PlanarFinalPreserveProducer<Schema>
{
    type Operation = FinalPlanarPreserveBinding<Schema>;
    type OutputFamily = PlanarFinalOutputFamily;
    type Provider = PlanarFinalOutputProvider;

    const IDENTITY: &'static str = "worth.query.certification.planar-final-preserve-producer.v1";
    const OUTPUT_ROLE: &'static str = "anchor";
    const APPLICABILITY: &'static [WorthQueryProducerApplicability] = &[PRESERVE];
    const REQUIRED_INVARIANTS: &'static [WorthQueryProducerInvariantRequirement] =
        &[WorthQueryProducerInvariantRequirement::new(
            "PositivePlanarTurn",
            1,
            0,
            ApplicationInvariantExecutionPoint::CommitBoundary,
        )];
    const RESOURCE_POLICY: &'static str = "bounded-synchronous";
    const REUSE_POLICY: &'static str = "exact-source";
}
