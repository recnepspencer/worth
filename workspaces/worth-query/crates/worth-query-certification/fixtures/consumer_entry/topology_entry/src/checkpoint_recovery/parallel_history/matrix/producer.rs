//! Source inputs derive from the declared partition set and real revision field.
use super::{application::*, control::Control, operation, source};
use std::sync::Arc;
use worth_query_host::facade::application_contribution::WorthQueryProducerResourceQuoteDenial;
use worth_query_host::facade::application_contribution::*;
use worth_query_host::facade::primary_graph::*;
const APPLICABILITY: &[WorthQueryProducerApplicability] = &[
    WorthQueryProducerApplicability::new("matrix", WorthQueryProducerLifecyclePosture::Initial),
    WorthQueryProducerApplicability::new("matrix", WorthQueryProducerLifecyclePosture::Preserve),
];
pub struct Family;
impl WorthQueryProducerOutputFamily<Schema> for Family {
    type Source = source::Binding<Schema>;
    type Entity = Node;
    const IDENTITY: &'static str = "matrix-producer-family";
    const SUPPORTED: &'static [WorthQueryProducerApplicability] = APPLICABILITY;
    fn profile_kind(_: &source::Result) -> &'static str {
        "matrix"
    }
}
#[derive(Clone)]
pub struct Demand(pub u64);
impl WorthQueryApplicationOutputDemand<Schema> for Demand {
    type OutputFamily = Family;
    fn source_intent(&self) -> source::Read {
        source::Read { key: self.0 }
    }
}
pub struct Producer;
pub struct Provider(pub Arc<Control>);
impl WorthQueryApplicationProducerBinding<Schema> for Producer {
    type Operation = operation::Binding<Schema>;
    type OutputFamily = Family;
    type Provider = Provider;
    type OutputRole = operation::NodeOutput<Schema>;
    const IDENTITY: &'static str = "matrix-producer";
    const APPLICABILITY: &'static [WorthQueryProducerApplicability] = APPLICABILITY;
    const REQUIRED_INVARIANTS: &'static [WorthQueryProducerInvariantRequirement] = &[];
    const RESOURCE_POLICY: &'static str = "bounded-synchronous";
    const REUSE_POLICY: &'static str = "matrix-exact";
    const INPUT_REUSE: Option<WorthQueryProducerInputReuseContract> =
        Some(WorthQueryProducerInputReuseContract::canonical_bitwise(
            WorthQueryDecisionContextDependencies::NONE,
        ));
}
impl WorthQueryApplicationProducerProvider<Schema, Producer> for Provider {
    const SEMANTIC_IDENTITY: &'static str = "matrix-provider";
    fn operation_input(&self, source: &source::Result) -> operation::Input {
        let (upstream, mut partitions) = self
            .0
            .input(source.key.strip_prefix("member-").unwrap().parse().unwrap());
        for part in &mut partitions {
            part.value *= source.revision + 1;
        }
        operation::Input {
            key: source.key.strip_prefix("member-").unwrap().parse().unwrap(),
            revision: source.revision,
            upstream,
            partitions,
        }
    }
    fn domain_denial_reason(&self, denial: &operation::Denied) -> Option<&'static str> {
        Some(match denial.phase {
            super::control::Phase::Prepare => "matrix prepare refusal",
            super::control::Phase::Compute => "matrix compute refusal",
            super::control::Phase::Complete => "matrix completion refusal",
        })
    }
    fn idempotency_key(&self, _: &source::Result, identity: &[u8; 32]) -> u64 {
        (identity
            .chunks_exact(8)
            .map(|chunk| u64::from_le_bytes(chunk.try_into().unwrap()))
            .fold(0, u64::wrapping_add))
            | (3 << 62)
    }
    fn demand_resources(
        &self,
        _: &source::Result,
    ) -> Result<WorthQueryProducerDemandResources, WorthQueryProducerResourceQuoteDenial> {
        Ok({ WorthQueryProducerDemandResources::new(Some(4096), 16384) })
    }
}
impl WorthQueryApplicationRequiredOutputConnection<Schema> for Connection {
    type Source = super::edit::Binding<Schema>;
    type Demand = Demand;
    const IDENTITY: &'static str = "matrix-source-output";
    fn demand_from_source(
        source: &super::edit::Input,
    ) -> Result<Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(Demand(source.key))
    }
}
impl WorthQueryApplicationRequiredOutputSource<Schema, Connection>
    for super::edit::Binding<Schema>
{
    fn demand_from_source(
        source: &super::edit::Input,
    ) -> Result<Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(Demand(source.key))
    }
}
