//! Quote the actual retained input vector from its authored symbolic row count.
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
    const RESOURCE_POLICY: &'static str = "actual-counted-input";
    const REUSE_POLICY: &'static str = "exact-source";
}
pub struct Provider;
impl<S: TopologySchemaBinding> WorthQueryApplicationProducerProvider<S, Producer<S>> for Provider {
    const SEMANTIC_IDENTITY: &'static str = "quote-count.provider";
    fn operation_input(&self, source: &Source) -> Input {
        // Execution has admitted this same immutable source's finite quote.
        Input {
            scope_key: source.key.clone(),
            rows: vec![1; source.count as usize],
        }
    }
    fn idempotency_key(&self, _: &Source, identity: &[u8; 32]) -> u64 {
        super::super::super::planar_source_key(identity) ^ 0x51554f5445
    }
    fn demand_resources(
        &self,
        source: &Source,
    ) -> Result<WorthQueryProducerDemandResources, WorthQueryProducerResourceQuoteDenial> {
        let count = usize::try_from(source.count)
            .map_err(|_| WorthQueryProducerResourceQuoteDenial::SizeOverflow)?;
        let bytes = count
            .checked_mul(size_of::<u64>())
            .and_then(|n| n.checked_add(size_of::<Input>()))
            .and_then(|n| n.checked_add(source.key.len()))
            .and_then(|n| n.checked_add(size_of::<Source>()))
            .and_then(|n| n.checked_add(source.key.capacity()))
            .ok_or(WorthQueryProducerResourceQuoteDenial::SizeOverflow)?;
        Ok(WorthQueryProducerDemandResources::new(None, bytes))
    }
}
