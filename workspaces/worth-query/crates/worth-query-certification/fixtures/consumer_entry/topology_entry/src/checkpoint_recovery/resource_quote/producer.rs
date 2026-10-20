//! Produce the counted output from its original authored source.
use super::*;
use std::marker::PhantomData;
const APPLICABILITY: &[WorthQueryProducerApplicability] = &[
    WorthQueryProducerApplicability::new("counted", WorthQueryProducerLifecyclePosture::Initial),
    WorthQueryProducerApplicability::new("counted", WorthQueryProducerLifecyclePosture::Preserve),
];
pub struct Family;
impl<S: TopologySchemaBinding> WorthQueryProducerOutputFamily<S> for Family {
    type Source = ReadBinding<S>;
    type Entity = Body;
    const IDENTITY: &'static str = "quote-count.family";
    const SUPPORTED: &'static [WorthQueryProducerApplicability] = APPLICABILITY;
    fn profile_kind(_: &Source) -> &'static str {
        "counted"
    }
}
#[derive(Clone)]
pub struct Demand(pub String);
impl<S: TopologySchemaBinding> WorthQueryApplicationOutputDemand<S> for Demand {
    type OutputFamily = Family;
    fn source_intent(&self) -> Read {
        Read {
            key: self.0.clone(),
        }
    }
}
pub struct Producer<S>(PhantomData<fn() -> S>);
impl<S: TopologySchemaBinding> WorthQueryApplicationProducerBinding<S> for Producer<S> {
    type Operation = ProduceBinding<S>;
    type OutputFamily = Family;
    type Provider = Provider;
    type OutputRole = PlanarAnchorOutput<S>;
    const IDENTITY: &'static str = "quote-count.producer";
    const APPLICABILITY: &'static [WorthQueryProducerApplicability] = APPLICABILITY;
    const REQUIRED_INVARIANTS: &'static [WorthQueryProducerInvariantRequirement] = &[];
    const REUSE_POLICY: &'static str = "exact-source";
}
pub struct Provider;
impl<S: TopologySchemaBinding> WorthQueryApplicationProducerProvider<S, Producer<S>> for Provider {
    const SEMANTIC_IDENTITY: &'static str = "quote-count.provider";
    fn operation_input(&self, source: &Source) -> Input {
        Input {
            scope_key: source.key.clone(),
            count: source.count,
        }
    }
    fn idempotency_key(&self, _: &Source, identity: &[u8; 32]) -> u64 {
        super::super::super::planar_source_key(identity) ^ 0x51554f5445
    }
}
