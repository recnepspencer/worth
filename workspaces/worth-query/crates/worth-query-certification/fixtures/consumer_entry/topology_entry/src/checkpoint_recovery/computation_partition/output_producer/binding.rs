// The producer of a region output: a demand for one scope's output runs the
// region output operation over the set the scope's ordinate names, first to
// create the output and then to keep it.

use std::marker::PhantomData;

use super::fixture_values::PositiveLength;
use super::runtime_api::application_contribution::{
    self, WorthQueryApplicationProducerBinding,
    WorthQueryApplicationProducerProvider, WorthQueryProducerApplicability,
    WorthQueryProducerInputReuseContract, WorthQueryProducerInvariantRequirement,
    WorthQueryProducerLifecyclePosture, WorthQueryProducerOutputFamily,
};

use super::region_output::{RegionOutputBinding, RegionOutputInput, TotalRegionOutput};
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
}
