// The producer of a region output: a demand for one scope's output runs the
// region output operation over the set the scope's ordinate names, first to
// create the output and then to keep it.

const DECISION_FACTS_PER_ENTRY: usize = 8;
/// The scope, the set, its weight and the entry an armed decision writes.
const DECISION_FACTS_BESIDE_ENTRIES: usize = 32;
/// The facts the largest set's decision may read.
pub(super) const DECISION_FACT_BUDGET: usize =
    DECISION_FACTS_PER_ENTRY * LARGEST_SET + DECISION_FACTS_BESIDE_ENTRIES;

use std::marker::PhantomData;

use super::fixture_values::PositiveLength;
use super::runtime_api::application_contribution::{
    self, WorthQueryApplicationOutputDemand, WorthQueryApplicationProducerBinding,
    WorthQueryApplicationProducerProvider, WorthQueryProducerApplicability,
    WorthQueryProducerDemandResources, WorthQueryProducerInputReuseContract,
    WorthQueryProducerInvariantRequirement, WorthQueryProducerLifecyclePosture,
    WorthQueryProducerOutputFamily,
};

use super::region_output::{
    RegionOutputBinding, RegionOutputInput, TotalRegionOutput, LARGEST_SET,
};
use super::*;

const APPLICABILITY: &[WorthQueryProducerApplicability] = &[
    WorthQueryProducerApplicability::new(
        "region-output",
        WorthQueryProducerLifecyclePosture::Initial,
    ),
    WorthQueryProducerApplicability::new(
        "region-output",
        WorthQueryProducerLifecyclePosture::Preserve,
    ),
];

pub(super) struct RegionOutputFamily;
impl<Schema: TopologySchemaBinding> WorthQueryProducerOutputFamily<Schema> for RegionOutputFamily {
    type Source = PlanarReadBinding<Schema>;
    type Entity = Body;
    const IDENTITY: &'static str = "worth.query.certification.region-output-family.v1";
    const SUPPORTED: &'static [WorthQueryProducerApplicability] = APPLICABILITY;
    fn profile_kind(_: &PlanarReadResult) -> &'static str {
        "region-output"
    }
}

/// A demand for the region output of the scope it names.
#[derive(Clone)]
pub(super) struct RegionOutputDemand(pub(super) String);
impl<Schema: TopologySchemaBinding> WorthQueryApplicationOutputDemand<Schema>
    for RegionOutputDemand
{
    type OutputFamily = RegionOutputFamily;
    fn source_intent(&self) -> PlanarRead {
        PlanarRead {
            body_key: self.0.clone(),
        }
    }
}

const PRODUCER: &str = "worth.query.certification.region-output-producer.v1";

pub(super) struct RegionOutputProducer<Schema, const REUSE: bool = false, const MODE: u8 = 0>(
    PhantomData<fn() -> Schema>,
);
impl<Schema: TopologySchemaBinding, const REUSE: bool, const MODE: u8>
    WorthQueryApplicationProducerBinding<Schema> for RegionOutputProducer<Schema, REUSE, MODE>
{
    type Operation = RegionOutputBinding<Schema>;
    type OutputFamily = RegionOutputFamily;
    type Provider = RegionOutputProvider<MODE>;
    const IDENTITY: &'static str = PRODUCER;
    type OutputRole = PlanarAnchorOutput<Schema>;
    const APPLICABILITY: &'static [WorthQueryProducerApplicability] = APPLICABILITY;
    const REQUIRED_INVARIANTS: &'static [WorthQueryProducerInvariantRequirement] = &[];
    const RESOURCE_POLICY: &'static str = "bounded-synchronous";
    const REUSE_POLICY: &'static str = "exact-source";
    const INPUT_REUSE: Option<WorthQueryProducerInputReuseContract> = if REUSE {
        Some(WorthQueryProducerInputReuseContract::canonical_bitwise(
            application_contribution::WorthQueryDecisionContextDependencies::NONE,
        ))
    } else {
        None
    };
}

/// The scope's ordinate names the set its output totals: an even ordinate
/// the even set, an odd one the odd set.
pub(super) struct RegionOutputProvider<const MODE: u8 = 0>;
impl<Schema: TopologySchemaBinding, const REUSE: bool, const MODE: u8>
    WorthQueryApplicationProducerProvider<Schema, RegionOutputProducer<Schema, REUSE, MODE>>
    for RegionOutputProvider<MODE>
{
    const SEMANTIC_IDENTITY: &'static str = "worth.query.certification.region-output-provider.v1";
    fn operation_input(&self, source: &PlanarReadResult) -> RegionOutputInput {
        let entries = if PositiveLength::get(&source.y).is_multiple_of(2) {
            "even"
        } else {
            "odd"
        };
        RegionOutputInput {
            scope_key: source.body_key.clone(),
            entries: entries.to_owned(),
        }
    }
    fn idempotency_key(&self, _: &PlanarReadResult, identity: &[u8; 32]) -> u64 {
        let key = planar_source_key(identity) ^ 0x9176_3c0b;
        if MODE == 4 {
            key | (3 << 62)
        } else {
            key
        }
    }
    fn demand_resources(
        &self,
        _: &PlanarReadResult,
    ) -> Result<
        WorthQueryProducerDemandResources,
        super::runtime_api::application_contribution::WorthQueryProducerResourceQuoteDenial,
    > {
        Ok({
            if MODE == 1 {
                // The wide-key decision may disclose every declared Native fact.
                // Reserve its finite projection bound, including own-write rebasing.
                WorthQueryProducerDemandResources::new(Some(4096), DECISION_FACT_BUDGET * 512)
            } else {
                planar_producer_resources()
            }
        })
    }
}
