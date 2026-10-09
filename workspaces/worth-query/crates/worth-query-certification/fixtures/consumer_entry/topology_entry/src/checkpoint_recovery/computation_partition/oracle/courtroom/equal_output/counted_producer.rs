//! The ordinary dependent uses the counted source with the existing operation.
use super::*;
pub(super) struct Family;
pub(super) struct Producer;
pub(super) struct Provider;
pub(super) struct Demand;
impl WorthQueryProducerOutputFamily<Schema> for Family {
    type Source = counted_source::CountedReadBinding<Schema>;
    type Entity = Body;
    const IDENTITY: &'static str = "courtroom-counted-dependent-family";
    const SUPPORTED: &'static [WorthQueryProducerApplicability] = APPLICABILITY;
    fn profile_kind(_: &counted_source::CountedReadResult) -> &'static str {
        "region-output"
    }
}
impl WorthQueryApplicationOutputDemand<Schema> for Demand {
    type OutputFamily = Family;
    fn source_intent(&self) -> counted_source::CountedRead {
        counted_source::CountedRead {
            body_key: DEPENDENT.to_owned(),
        }
    }
}
impl WorthQueryApplicationProducerBinding<Schema> for Producer {
    type Operation = dependent_publication::Binding;
    type OutputFamily = Family;
    type Provider = Provider;
    type OutputRole = PlanarAnchorOutput<Schema>;
    const IDENTITY: &'static str = "courtroom-counted-dependent-producer";
    const APPLICABILITY: &'static [WorthQueryProducerApplicability] = APPLICABILITY;
    const REQUIRED_INVARIANTS: &'static [WorthQueryProducerInvariantRequirement] = &[];
    const RESOURCE_POLICY: &'static str = "bounded-synchronous";
    const REUSE_POLICY: &'static str = "exact-source";
    const INPUT_REUSE: Option<WorthQueryProducerInputReuseContract> =
        Some(WorthQueryProducerInputReuseContract::canonical_bitwise(
            WorthQueryDecisionContextDependencies::NONE,
        ));
}
impl WorthQueryApplicationProducerProvider<Schema, Producer> for Provider {
    const SEMANTIC_IDENTITY: &'static str = "courtroom-counted-dependent-provider";
    fn operation_input(
        &self,
        source: &counted_source::CountedReadResult,
    ) -> dependent_publication::Input {
        dependent_publication::Input {
            scope_key: source.body_key.clone(),
            entries: "the-upstream-output".to_owned(),
        }
    }
    fn idempotency_key(&self, _: &counted_source::CountedReadResult, identity: &[u8; 32]) -> u64 {
        planar_source_key(identity) ^ 0x0006_12c0
    }
    fn demand_resources(
        &self,
        _: &counted_source::CountedReadResult,
    ) -> WorthQueryProducerDemandResources {
        planar_producer_resources()
    }
}
