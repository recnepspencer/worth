//! Distinct fixed exactly-one roles share one operation, canonical input and family.
use super::super::super::super::output_producer::RegionOutputFamily;
use super::binding::{CreatedTotal, PreservedTotal};
use super::*;
use worth_query_consumer_values::PositiveLength;
use worth_query_host::facade::application_contribution::*;

macro_rules! producer {
    ($producer:ident, $binding:ident, $role:ident, $posture:ident, $id:literal) => {
        pub(in crate::checkpoint_recovery::computation_partition::oracle::collection) struct $producer;
        impl WorthQueryApplicationProducerBinding<CheckpointSchema> for $producer {
            type Operation = $binding<CheckpointSchema>;
            type OutputFamily = RegionOutputFamily;
            type OutputRole = $role<CheckpointSchema>;
            type Provider = Provider;
            const IDENTITY: &'static str = concat!("checked-collection-", $id);
            const APPLICABILITY: &'static [WorthQueryProducerApplicability] =
                &[WorthQueryProducerApplicability::new(
                    "region-output",
                    WorthQueryProducerLifecyclePosture::$posture,
                )];
            const REQUIRED_INVARIANTS: &'static [WorthQueryProducerInvariantRequirement] = &[];
            const RESOURCE_POLICY: &'static str = "bounded-synchronous";
            const REUSE_POLICY: &'static str = "exact-source";
        }
        impl WorthQueryApplicationProducerProvider<CheckpointSchema, $producer> for Provider {
            const SEMANTIC_IDENTITY: &'static str = concat!("checked-collection-provider-", $id);
            fn operation_input(&self, source: &PlanarReadResult) -> RegionOutputInput {
                RegionOutputInput {
                    scope_key: source.body_key.clone(),
                    entries: if PositiveLength::get(&source.y).is_multiple_of(2) {
                        "even"
                    } else {
                        "odd"
                    }
                    .into(),
                }
            }
            fn idempotency_key(&self, _: &PlanarReadResult, epoch: &[u8; 32]) -> u64 {
                planar_source_key(epoch) ^ 0x71cb_6619
            }
            fn demand_resources(&self, _: &PlanarReadResult) -> Result<WorthQueryProducerDemandResources, worth_query_host::facade::application_contribution::WorthQueryProducerResourceQuoteDenial> { Ok({
                planar_producer_resources()
            }) }
        }
    };
}
pub(in crate::checkpoint_recovery::computation_partition::oracle::collection) struct Provider;
producer!(
    InitialProducer,
    InitialBinding,
    CreatedTotal,
    Initial,
    "initial-v1"
);
producer!(
    PreserveProducer,
    PreserveBinding,
    PreservedTotal,
    Preserve,
    "preserve-v1"
);
