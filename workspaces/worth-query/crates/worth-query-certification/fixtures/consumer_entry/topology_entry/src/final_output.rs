use std::marker::PhantomData;

use worth_query_decl::facade::{
    application_operation::*, application_schema::*, worth_query_operation,
    worth_query_structured_value_binding,
};
use worth_query_host::facade::application_contribution::{
    WorthQueryApplicationOutputDemand, WorthQueryApplicationProducerBinding,
    WorthQueryApplicationProducerProvider, WorthQueryDecisionContextDependencies,
    WorthQueryProducerApplicability, WorthQueryProducerInputReuseContract,
    WorthQueryProducerInvariantRequirement, WorthQueryProducerLifecyclePosture,
    WorthQueryProducerOutputFamily,
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
worth_query_operation!(pub PublishFinalPlanarOutput for Schema: TopologySchemaBinding, input FinalPlanarMutationInputBinding;
    access {
        read_entities: [Body]; read_fields: [BodyKey, Length];
        writes: [BodyKey, PositionX, PositionY, Length];
        creates: [Body]; links: [PlanarSuccessor];
    }
);

pub struct FinalPlanarMutationBinding<Schema>(PhantomData<fn() -> Schema>);

mod output_roles;
pub use output_roles::*;

impl<Schema: TopologySchemaBinding> ApplicationMutationBinding<Schema>
    for FinalPlanarMutationBinding<Schema>
{
    fn workflow_execution_requirements(
    ) -> worth_query_decl::facade::application_operation::ApplicationWorkflowExecutionRequirements
    {
        worth_query_decl::facade::application_operation::ApplicationWorkflowExecutionRequirements::not_applicable()
    }

    type Input = FinalPlanarMutation;
    type InputBinding = FinalPlanarMutationInputBinding;
    type Result = worth_query_consumer_values::PlanarAdjustmentResult;
    type ResultBinding = PlanarMutationResultBinding;
    type IdempotencyKey = u64;
    type Operation = PublishFinalPlanarOutput;
    type Decision =
        worth_query_host::facade::primary_graph::WorthQueryInvariantMutationTarget<Schema, Body>;
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

    const IDENTITY: &'static str = "worth.query.certification.final-planar-mutation.v2";
    const HANDLER_IDENTITY: &'static str = "worth.query.certification.final-planar-handler.v2";
    const IDEMPOTENCY_IDENTITY: &'static str = "worth.query.certification.final-planar-command.v1";

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

mod handler;
pub use handler::FinalPlanarMutationHandler;

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
    type Entity = Body;

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

#[derive(Clone, Copy, Default)]
pub struct PlanarFinalOutputProvider {
    key_mask: u64,
}
impl PlanarFinalOutputProvider {
    #[cfg(test)]
    pub(crate) fn with_uniform_decimal_key_width(mut self) -> Self {
        self.key_mask = 3 << 62;
        self
    }
}

#[cfg(test)]
static PROVIDER_CONTACTS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

#[cfg(test)]
pub(super) fn reset_provider_contacts() {
    PROVIDER_CONTACTS.store(0, std::sync::atomic::Ordering::SeqCst);
}

#[cfg(test)]
pub(super) fn provider_contacts() -> usize {
    PROVIDER_CONTACTS.load(std::sync::atomic::Ordering::SeqCst)
}

impl<Schema: TopologySchemaBinding>
    WorthQueryApplicationProducerProvider<Schema, PlanarFinalOutputProducer<Schema>>
    for PlanarFinalOutputProvider
{
    const SEMANTIC_IDENTITY: &'static str =
        "worth.query.certification.planar-final-output-provider.v2";

    fn operation_input(&self, source: &PlanarReadResult) -> FinalPlanarMutation {
        #[cfg(test)]
        PROVIDER_CONTACTS.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
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
        #[cfg(test)]
        PROVIDER_CONTACTS.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        (super::planar_source_key(source_identity) ^ 0x9174_f1a1_0000_0001) | self.key_mask
    }
}

pub struct PlanarFinalOutputProducer<Schema>(PhantomData<fn() -> Schema>);

impl<Schema: TopologySchemaBinding> WorthQueryApplicationProducerBinding<Schema>
    for PlanarFinalOutputProducer<Schema>
{
    type Operation = FinalPlanarMutationBinding<Schema>;
    type OutputFamily = PlanarFinalOutputFamily;
    type Provider = PlanarFinalOutputProvider;

    type OutputRole = FinalAnchorOutput<Schema>;

    const IDENTITY: &'static str = "worth.query.certification.planar-final-output-producer.v2";
    const APPLICABILITY: &'static [WorthQueryProducerApplicability] = &[INITIAL];
    const REQUIRED_INVARIANTS: &'static [WorthQueryProducerInvariantRequirement] =
        &[WorthQueryProducerInvariantRequirement::new(
            "PositivePlanarTurn",
            1,
            0,
            ApplicationInvariantExecutionPoint::CommitBoundary,
        )];
    const REUSE_POLICY: &'static str = "exact-source";
    const INPUT_REUSE: Option<WorthQueryProducerInputReuseContract> = Schema::FINAL_INPUT_REUSE;
}

mod definition;
mod feature;
pub use definition::{CompletePlanarFinalRuntime, PlanarFinalFeatureDefinition};
mod preservation;
pub use feature::{
    planar_final_feature_runtime, PlanarFinalFeatureRoles, PlanarFinalFeatureRuntime,
};
mod preserve_readiness;
mod readiness;
pub use preservation::*;
pub use preserve_readiness::*;
pub use readiness::*;
