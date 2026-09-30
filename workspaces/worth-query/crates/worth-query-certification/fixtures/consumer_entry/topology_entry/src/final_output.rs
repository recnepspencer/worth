use std::marker::PhantomData;

use worth_query_decl::facade::{
    application_operation::*, application_schema::*, worth_query_operation,
    worth_query_operation_creates, worth_query_operation_links, worth_query_operation_reads,
    worth_query_operation_writes, worth_query_structured_value_binding,
};
use worth_query_host::facade::application_contribution::{
    WorthQueryApplicationOutputDemand, WorthQueryApplicationProducerBinding,
    WorthQueryApplicationProducerProvider, WorthQueryProducerApplicability,
    WorthQueryProducerDemandResources, WorthQueryProducerInvariantRequirement,
    WorthQueryProducerLifecyclePosture, WorthQueryProducerOutputFamily,
};
use worth_query_host::facade::primary_graph::{
    CandidateWriter, DecisionReader, HandlerExecutionDenial, HandlerResult, OperationHandler,
};
use worth_query_host::facade::{application_contribution, domain};

use super::{
    Body, BodyKey, ConsumerPrincipalBinding, ExternalPrincipalMapping, Length,
    PlanarMutationDenialBinding, PlanarMutationResultBinding, PlanarMutationScope, PlanarPosition,
    PlanarQuery, PlanarRead, PlanarReadBinding, PlanarReadResult, PlanarSuccessor, PositionX,
    PositionY, Principal, TopologySchemaBinding,
};

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct FinalPlanarMutation {
    pub scope_key: String,
    pub output_key: String,
    pub value: worth_query_consumer_values::PositiveLength,
}

worth_query_structured_value_binding!(pub FinalPlanarMutationInputBinding for FinalPlanarMutation {
    identity: "worth.query.certification.final-planar-mutation-input.v1"
});
worth_query_operation!(pub PublishFinalPlanarOutput for Schema: TopologySchemaBinding, input FinalPlanarMutationInputBinding);
worth_query_operation_reads!(PublishFinalPlanarOutput => [Body, BodyKey, Length]);
worth_query_operation_writes!(PublishFinalPlanarOutput => [BodyKey, PositionX, PositionY, Length]);
worth_query_operation_creates!(PublishFinalPlanarOutput => [Body]);
worth_query_operation_links!(PublishFinalPlanarOutput => [PlanarSuccessor]);

pub struct FinalPlanarMutationBinding<Schema>(PhantomData<fn() -> Schema>);

pub struct FinalPlanarOutputs;

/// The vertex that anchors the final ring.
pub const fn final_anchor_output<Schema: TopologySchemaBinding>(
) -> WorthQueryApplicationOutputRole<FinalPlanarMutationBinding<Schema>, Body, WorthQueryCreateOutput>
{
    WorthQueryApplicationOutputRole::for_entity::<Schema>("anchor")
}

/// The vertex that closes the final ring. Declared at-most-one; every final
/// output this fixture publishes binds it.
pub const fn final_closing_output<Schema: TopologySchemaBinding>(
) -> WorthQueryApplicationOptionalOutputRole<
    FinalPlanarMutationBinding<Schema>,
    Body,
    WorthQueryCreateOutput,
> {
    WorthQueryApplicationOptionalOutputRole::for_entity::<Schema>("closing")
}

/// A declared at-most-one role no final output binds.
pub const fn final_auxiliary_output<Schema: TopologySchemaBinding>(
) -> WorthQueryApplicationOptionalOutputRole<
    FinalPlanarMutationBinding<Schema>,
    Body,
    WorthQueryCreateOutput,
> {
    WorthQueryApplicationOptionalOutputRole::for_entity::<Schema>("auxiliary")
}

/// The other vertices the final ring creates, one member per vertex.
pub const fn final_created_outputs<Schema: TopologySchemaBinding>(
) -> WorthQueryApplicationOutputRoleFamily<FinalPlanarMutationBinding<Schema>, Body> {
    WorthQueryApplicationOutputRoleFamily::for_entity::<Schema>(
        "created.",
        ApplicationMutationOutputPostureSet::CREATE,
        0,
    )
}

impl<Schema: TopologySchemaBinding> ApplicationMutationOutputContract<Schema>
    for FinalPlanarOutputs
{
    const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] = &[
        final_anchor_output::<Schema>().descriptor(),
        final_closing_output::<Schema>().descriptor(),
        final_auxiliary_output::<Schema>().descriptor(),
    ];
    const ROLE_FAMILIES: &'static [ApplicationMutationOutputRoleFamilyDescriptor] =
        &[final_created_outputs::<Schema>().descriptor()];
}

impl<Schema: TopologySchemaBinding> ApplicationMutationBinding<Schema>
    for FinalPlanarMutationBinding<Schema>
{
    type Input = FinalPlanarMutation;
    type InputBinding = FinalPlanarMutationInputBinding;
    type Result = worth_query_consumer_values::PlanarAdjustmentResult;
    type ResultBinding = PlanarMutationResultBinding;
    type IdempotencyKey = u64;
    type Operation = PublishFinalPlanarOutput;
    type Decision = ();
    type Denial = worth_query_consumer_values::PlanarMutationDenial;
    type DenialBinding = PlanarMutationDenialBinding;
    type Output = FinalPlanarOutputs;
    type ScopeBinding = PlanarMutationScope<Schema>;
    type PrincipalBinding = ConsumerPrincipalBinding;
    type Mapping = ExternalPrincipalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type SourceExpectation = ApplicationQueryMutationSource<PlanarQuery>;

    const IDENTITY: &'static str = "worth.query.certification.final-planar-mutation.v1";
    const HANDLER_IDENTITY: &'static str = "worth.query.certification.final-planar-handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str = "worth.query.certification.final-planar-command.v1";
    const CANDIDATES: ApplicationCandidateRequirements =
        super::requirements(3, 3, 0, 12, 4096, 4096);

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

impl<Schema: TopologySchemaBinding> ApplicationMutationIntent<Schema> for FinalPlanarMutation {
    type Binding = FinalPlanarMutationBinding<Schema>;

    fn input(&self) -> &Self {
        self
    }

    fn scope_binding(&self) -> PlanarMutationScope<Schema> {
        PlanarMutationScope::new(BodyKey::reference(), self.scope_key.clone())
    }
}

pub struct FinalPlanarMutationHandler;

impl<Schema: TopologySchemaBinding> OperationHandler<Schema, FinalPlanarMutationBinding<Schema>>
    for FinalPlanarMutationHandler
{
    fn decide(
        &self,
        input: &FinalPlanarMutation,
        reader: &mut DecisionReader<'_, '_, '_, Schema, FinalPlanarMutationBinding<Schema>>,
    ) -> HandlerResult<(), worth_query_consumer_values::PlanarMutationDenial> {
        match reader.resolve_entity(BodyKey::reference(), input.scope_key.clone()) {
            Ok(_) => HandlerResult::Completed(()),
            Err(error) => HandlerResult::ExecutionDenied(error),
        }
    }

    fn candidate_requirements(
        &self,
        _: &FinalPlanarMutation,
        _: &(),
    ) -> ApplicationCandidateRequirements {
        FinalPlanarMutationBinding::<Schema>::CANDIDATES
    }

    fn build_candidate(
        &self,
        input: &FinalPlanarMutation,
        _: (),
        writer: &mut CandidateWriter<'_, Schema, FinalPlanarMutationBinding<Schema>>,
    ) -> HandlerResult<
        worth_query_consumer_values::PlanarAdjustmentResult,
        worth_query_consumer_values::PlanarMutationDenial,
    > {
        let keys = [
            input.output_key.clone(),
            format!("{}:b", input.output_key),
            format!("{}:c", input.output_key),
        ];
        let base_y = worth_query_consumer_values::PositiveLength::get(&input.value);
        let Some(next_y) = base_y.checked_add(1) else {
            return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(
                std::io::Error::other("final output ordinate exceeds its value binding"),
            ));
        };
        let coordinates = [(1, base_y), (2, base_y), (1, next_y)];
        let mut entities = Vec::with_capacity(3);
        for (key, (x, y)) in keys.iter().zip(coordinates) {
            let entity_key =
                match worth_query_host::facade::primary_graph::WorthQueryApplicationEntityKey::new(
                    key,
                ) {
                    Ok(key) => key,
                    Err(error) => {
                        return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error))
                    }
                };
            let entity = match writer.create_entity(Body::reference(), entity_key) {
                Ok(entity) => entity,
                Err(error) => {
                    return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error))
                }
            };
            for result in [
                writer.initialize_field(&entity, BodyKey::reference(), key.clone()),
                writer.initialize_field(
                    &entity,
                    PositionX::reference(),
                    worth_query_consumer_values::PositiveLength::new(x).unwrap(),
                ),
                writer.initialize_field(
                    &entity,
                    PositionY::reference(),
                    worth_query_consumer_values::PositiveLength::new(y).unwrap(),
                ),
                writer.initialize_field(&entity, Length::reference(), input.value),
            ] {
                if let Err(error) = result {
                    return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error));
                }
            }
            entities.push(entity);
        }
        for index in 0..entities.len() {
            if let Err(error) = writer.link(
                PlanarSuccessor::reference(),
                format!("final-successor:{}", keys[index]),
                &entities[index],
                &entities[(index + 1) % entities.len()],
            ) {
                return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error));
            }
        }
        let entity = &entities[0];
        if let Err(error) = writer.create_output(final_anchor_output::<Schema>(), entity) {
            return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error));
        }
        let role = final_created_outputs::<Schema>()
            .member::<WorthQueryCreateOutput>(&keys[1])
            .expect("fixture keys name valid created members");
        if let Err(error) = writer.create_output(role, &entities[1]) {
            return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error));
        }
        if let Err(error) = writer.create_output(final_closing_output::<Schema>(), &entities[2]) {
            return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error));
        }
        HandlerResult::Completed(worth_query_consumer_values::PlanarAdjustmentResult {
            changed_vertices: 1,
        })
    }
}

const INITIAL: WorthQueryProducerApplicability = WorthQueryProducerApplicability::new(
    "planar-final",
    WorthQueryProducerLifecyclePosture::Initial,
);
const PRESERVE: WorthQueryProducerApplicability = WorthQueryProducerApplicability::new(
    "planar-final",
    WorthQueryProducerLifecyclePosture::Preserve,
);
const SUPPORTED: &[WorthQueryProducerApplicability] = &[INITIAL, PRESERVE];

pub struct PlanarFinalOutputFamily;

impl<Schema: TopologySchemaBinding> WorthQueryProducerOutputFamily<Schema>
    for PlanarFinalOutputFamily
{
    type Source = PlanarReadBinding<Schema>;

    const IDENTITY: &'static str = "worth.query.certification.planar-final-output.v1";
    const SUPPORTED: &'static [WorthQueryProducerApplicability] = SUPPORTED;

    fn profile_kind(_: &PlanarReadResult) -> &'static str {
        "planar-final"
    }
}

#[derive(Clone)]
pub struct PlanarFinalOutputDemand {
    body_key: String,
}

impl PlanarFinalOutputDemand {
    pub fn new(body_key: impl Into<String>) -> Self {
        Self {
            body_key: body_key.into(),
        }
    }

    pub fn body_key(&self) -> &str {
        &self.body_key
    }
}

impl<Schema: TopologySchemaBinding> WorthQueryApplicationOutputDemand<Schema>
    for PlanarFinalOutputDemand
{
    type OutputFamily = PlanarFinalOutputFamily;

    fn source_intent(&self) -> PlanarRead {
        PlanarRead {
            body_key: self.body_key.clone(),
        }
    }
}

pub struct PlanarFinalOutputProvider;

impl<Schema: TopologySchemaBinding>
    WorthQueryApplicationProducerProvider<Schema, PlanarFinalOutputProducer<Schema>>
    for PlanarFinalOutputProvider
{
    const SEMANTIC_IDENTITY: &'static str =
        "worth.query.certification.planar-final-output-provider.v1";

    fn operation_input(&self, source: &PlanarReadResult) -> FinalPlanarMutation {
        FinalPlanarMutation {
            scope_key: source.body_key.clone(),
            output_key: format!("final:{}", source.body_key),
            value: worth_query_consumer_values::PositiveLength::new(
                worth_query_consumer_values::PositiveLength::get(&source.y) + 2,
            )
            .expect("a positive derived output has a positive successor"),
        }
    }

    fn idempotency_key(&self, _: &PlanarReadResult, source_identity: &[u8; 32]) -> u64 {
        super::planar_source_key(source_identity) ^ 0x9174_f1a1_0000_0001
    }

    fn demand_resources(&self, _: &PlanarReadResult) -> WorthQueryProducerDemandResources {
        super::planar_producer_resources()
    }
}

pub struct PlanarFinalOutputProducer<Schema>(PhantomData<fn() -> Schema>);

impl<Schema: TopologySchemaBinding> WorthQueryApplicationProducerBinding<Schema>
    for PlanarFinalOutputProducer<Schema>
{
    type Operation = FinalPlanarMutationBinding<Schema>;
    type OutputFamily = PlanarFinalOutputFamily;
    type Provider = PlanarFinalOutputProvider;

    const IDENTITY: &'static str = "worth.query.certification.planar-final-output-producer.v1";
    const OUTPUT_ROLE: &'static str = "anchor";
    const APPLICABILITY: &'static [WorthQueryProducerApplicability] = &[INITIAL];
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

mod preservation;
mod preserve_readiness;
mod readiness;
pub use preservation::*;
pub use preserve_readiness::*;
pub use readiness::*;
