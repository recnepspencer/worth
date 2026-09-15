use std::marker::PhantomData;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use worth_query_consumer_values::{PlanarDerivedOutput, PlanarOperation};
use worth_query_decl::facade::application_schema::ApplicationInvariantExecutionPoint;
use worth_query_host::facade::application_contribution::{
    WorthQueryApplicationProducerBinding, WorthQueryApplicationProducerProvider,
    WorthQueryProducerApplicability, WorthQueryProducerDemandResources,
    WorthQueryProducerInvariantRequirement, WorthQueryProducerLifecyclePosture,
    WorthQueryProducerOutputFamily,
};

use super::{PlanarMutationBinding, TopologySchemaBinding};

const INITIAL: WorthQueryProducerApplicability =
    WorthQueryProducerApplicability::new("planar", WorthQueryProducerLifecyclePosture::Initial);
const PRESERVE: WorthQueryProducerApplicability =
    WorthQueryProducerApplicability::new("planar", WorthQueryProducerLifecyclePosture::Preserve);
const PRIMARY: &[WorthQueryProducerApplicability] = &[INITIAL, PRESERVE];
const SUPPORTED: &[WorthQueryProducerApplicability] = &[
    INITIAL,
    PRESERVE,
    super::alternate_output::ALTERNATE_OUTPUT_APPLICABILITY,
];

pub struct PlanarOutputFamily;

impl<Schema: TopologySchemaBinding> WorthQueryProducerOutputFamily<Schema> for PlanarOutputFamily {
    type Source = super::PlanarReadBinding<Schema>;

    const IDENTITY: &'static str = "worth.query.certification.planar-output.v1";
    const SUPPORTED: &'static [WorthQueryProducerApplicability] = SUPPORTED;

    fn profile_kind(_: &super::PlanarReadResult) -> &'static str {
        "planar"
    }
}

pub struct InitialPlanarProvider {
    authorization_denials: Arc<AtomicUsize>,
}

impl InitialPlanarProvider {
    pub fn new(authorization_denials: Arc<AtomicUsize>) -> Self {
        Self {
            authorization_denials,
        }
    }
}

impl<Schema: TopologySchemaBinding>
    WorthQueryApplicationProducerProvider<Schema, InitialPlanarProducer<Schema>>
    for InitialPlanarProvider
{
    const SEMANTIC_IDENTITY: &'static str = "worth.query.certification.planar-initial-provider.v1";

    fn operation_input(&self, source: &super::PlanarReadResult) -> super::PlanarMutation {
        let mut input = planar_producer_input(source);
        if self
            .authorization_denials
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |remaining| {
                remaining.checked_sub(1)
            })
            .is_ok()
        {
            input.scope_key.push_str(":authorization-denied");
        }
        input
    }

    fn idempotency_key(&self, _: &super::PlanarReadResult, source_identity: &[u8; 32]) -> u64 {
        planar_source_key(source_identity)
    }

    fn demand_resources(&self, _: &super::PlanarReadResult) -> WorthQueryProducerDemandResources {
        planar_producer_resources()
    }
}

pub struct InitialPlanarProducer<Schema>(PhantomData<fn() -> Schema>);

impl<Schema: TopologySchemaBinding> WorthQueryApplicationProducerBinding<Schema>
    for InitialPlanarProducer<Schema>
{
    type Operation = PlanarMutationBinding<Schema>;
    type OutputFamily = PlanarOutputFamily;
    type Provider = InitialPlanarProvider;

    const IDENTITY: &'static str = "worth.query.certification.planar-initial.v1";
    const OUTPUT_ROLE: &'static str = "anchor";
    const APPLICABILITY: &'static [WorthQueryProducerApplicability] = PRIMARY;
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

pub fn planar_producer_input(source: &super::PlanarReadResult) -> super::PlanarMutation {
    super::PlanarMutation {
        scope_key: source.body_key.clone(),
        operation: PlanarOperation::PublishDerivedOutput(PlanarDerivedOutput {
            body_key: source.body_key.clone(),
            value: worth_query_consumer_values::PositiveLength::new(
                worth_query_consumer_values::PositiveLength::get(&source.y) + 1,
            )
            .expect("a positive planar source has a positive successor"),
        }),
        validator_work: 4_096,
    }
}

pub fn planar_source_key(source_identity: &[u8; 32]) -> u64 {
    source_identity
        .chunks_exact(8)
        .map(|chunk| u64::from_le_bytes(chunk.try_into().expect("eight-byte identity chunk")))
        .fold(0, u64::wrapping_add)
}

pub const fn planar_producer_resources() -> WorthQueryProducerDemandResources {
    WorthQueryProducerDemandResources::new(4_096, 8_192)
}
