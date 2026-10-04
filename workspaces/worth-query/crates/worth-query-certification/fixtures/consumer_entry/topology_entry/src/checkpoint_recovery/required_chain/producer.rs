use super::*;
use std::marker::PhantomData;
use worth_query_host::facade::application_contribution::*;

pub(super) struct ChainFamily;
// Every chain node reads its upstream values: once one changes, its retained
// output's read facts are stale and the refresh selects Preserve over it.
const APPLICABILITY: &[WorthQueryProducerApplicability] = &[
    WorthQueryProducerApplicability::new(
        "consumed-chain",
        WorthQueryProducerLifecyclePosture::Initial,
    ),
    WorthQueryProducerApplicability::new(
        "consumed-chain",
        WorthQueryProducerLifecyclePosture::Preserve,
    ),
];

impl<Schema: TopologySchemaBinding> WorthQueryProducerOutputFamily<Schema> for ChainFamily {
    type Source = PlanarOutputReadBinding<Schema>;
    const IDENTITY: &'static str = "worth.query.certification.consumed-chain-family.v1";
    const SUPPORTED: &'static [WorthQueryProducerApplicability] = APPLICABILITY;
    fn profile_kind(_: &PlanarOutputReadResult) -> &'static str {
        "consumed-chain"
    }
}

#[derive(Clone)]
pub(super) struct ChainDemand(pub String);
impl<Schema: TopologySchemaBinding> WorthQueryApplicationOutputDemand<Schema> for ChainDemand {
    type OutputFamily = ChainFamily;
    fn source_intent(&self) -> PlanarOutputRead {
        PlanarOutputRead {
            body_key: self.0.clone(),
        }
    }
}

pub(super) struct ChainProducer<Schema>(PhantomData<fn() -> Schema>);
pub(super) struct ChainProvider;

impl<Schema: TopologySchemaBinding> WorthQueryApplicationProducerBinding<Schema>
    for ChainProducer<Schema>
{
    type Operation = ChainBinding<Schema>;
    type OutputFamily = ChainFamily;
    type Provider = ChainProvider;
    const IDENTITY: &'static str = "worth.query.certification.consumed-chain-producer.v1";
    const OUTPUT_ROLE: &'static str = "anchor";
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
    WorthQueryApplicationProducerProvider<Schema, ChainProducer<Schema>> for ChainProvider
{
    const SEMANTIC_IDENTITY: &'static str = "worth.query.certification.consumed-chain-provider.v1";
    fn operation_input(&self, source: &PlanarOutputReadResult) -> ChainInput {
        let upstream = |key: &str, root| ChainUpstream {
            key: key.to_owned(),
            root,
        };
        let upstreams = match source.body_key.as_str() {
            "anchor-b" => vec![upstream("anchor-a", true)],
            "anchor-c" => vec![upstream("anchor-b", false)],
            // The diamond's shared dependent consumes both root outputs.
            "diamond-join" => vec![
                upstream("diamond-left", true),
                upstream("diamond-right", true),
            ],
            ring => vec![super::ring_world::upstream_of(ring)
                .expect("the courtroom declares no other downstream node")],
        };
        ChainInput {
            scope_key: source.body_key.clone(),
            upstreams,
            value: source.value,
        }
    }
    fn idempotency_key(&self, _: &PlanarOutputReadResult, identity: &[u8; 32]) -> u64 {
        planar_source_key(identity) ^ 0x9176_3000
    }
    fn demand_resources(&self, _: &PlanarOutputReadResult) -> WorthQueryProducerDemandResources {
        planar_producer_resources()
    }
}
