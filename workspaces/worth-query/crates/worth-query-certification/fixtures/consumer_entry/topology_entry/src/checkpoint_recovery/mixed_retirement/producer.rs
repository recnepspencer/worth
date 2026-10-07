//! A real installed producer for the existing mixed vertex replacement.
use super::*;
use std::{
    marker::PhantomData,
    sync::atomic::{AtomicUsize, Ordering},
};
use worth_query_consumer_values::{PlanarVertex, PlanarVertexReplacement};
use worth_query_host::facade::application_contribution::*;

pub(super) struct ReplacementFamily;
const APPLICABILITY: &[WorthQueryProducerApplicability] = &[
    WorthQueryProducerApplicability::new(
        "replacement",
        WorthQueryProducerLifecyclePosture::Initial,
    ),
    WorthQueryProducerApplicability::new(
        "replacement",
        WorthQueryProducerLifecyclePosture::Preserve,
    ),
];

impl<Schema: TopologySchemaBinding> WorthQueryProducerOutputFamily<Schema> for ReplacementFamily {
    type Source = PlanarReadBinding<Schema>;
    type Entity = Body;
    const IDENTITY: &'static str = "worth.query.certification.mixed-replacement-family.v1";
    const SUPPORTED: &'static [WorthQueryProducerApplicability] = APPLICABILITY;
    fn profile_kind(_: &PlanarReadResult) -> &'static str {
        "replacement"
    }
}

#[derive(Clone)]
pub(super) struct ReplacementDemand(pub String);
impl<Schema: TopologySchemaBinding> WorthQueryApplicationOutputDemand<Schema>
    for ReplacementDemand
{
    type OutputFamily = ReplacementFamily;
    fn source_intent(&self) -> PlanarRead {
        PlanarRead {
            body_key: self.0.clone(),
        }
    }
}

pub(super) struct ReplacementProducer<Schema>(PhantomData<fn() -> Schema>);
pub(super) struct ReplacementProvider;
static CONTACTS: AtomicUsize = AtomicUsize::new(0);
pub(super) fn reset_contacts() {
    CONTACTS.store(0, Ordering::SeqCst);
}
pub(super) fn contacts() -> usize {
    CONTACTS.load(Ordering::SeqCst)
}

impl<Schema: TopologySchemaBinding> WorthQueryApplicationProducerBinding<Schema>
    for ReplacementProducer<Schema>
{
    type Operation = VertexReplacementBinding<Schema>;
    type OutputFamily = ReplacementFamily;
    type Provider = ReplacementProvider;
    type OutputRole = VertexReplacementCreatedOutput<Schema>;
    const IDENTITY: &'static str = "worth.query.certification.mixed-replacement-producer.v1";
    const APPLICABILITY: &'static [WorthQueryProducerApplicability] = APPLICABILITY;
    const REQUIRED_INVARIANTS: &'static [WorthQueryProducerInvariantRequirement] = &[];
    const RESOURCE_POLICY: &'static str = "bounded-synchronous";
    const REUSE_POLICY: &'static str = "exact-source";
    const INPUT_REUSE: Option<WorthQueryProducerInputReuseContract> =
        Some(WorthQueryProducerInputReuseContract::canonical_bitwise(
            WorthQueryDecisionContextDependencies::NONE,
        ));
}

impl<Schema: TopologySchemaBinding>
    WorthQueryApplicationProducerProvider<Schema, ReplacementProducer<Schema>>
    for ReplacementProvider
{
    const SEMANTIC_IDENTITY: &'static str =
        "worth.query.certification.mixed-replacement-provider.v1";
    fn operation_input(&self, source: &PlanarReadResult) -> VertexReplacement {
        CONTACTS.fetch_add(1, Ordering::SeqCst);
        VertexReplacement {
            scope_key: source.body_key.clone(),
            replacement: PlanarVertexReplacement {
                retired_key: "anchor-b".into(),
                next_key: "anchor-c".into(),
                replacement: PlanarVertex {
                    body_key: "replacement-b".into(),
                    x: length(11),
                    y: length(1),
                },
            },
        }
    }
    fn idempotency_key(&self, _: &PlanarReadResult, identity: &[u8; 32]) -> u64 {
        CONTACTS.fetch_add(1, Ordering::SeqCst);
        planar_source_key(identity) ^ 0x9176_7301
    }
    fn demand_resources(&self, _: &PlanarReadResult) -> WorthQueryProducerDemandResources {
        CONTACTS.fetch_add(1, Ordering::SeqCst);
        planar_producer_resources()
    }
}
