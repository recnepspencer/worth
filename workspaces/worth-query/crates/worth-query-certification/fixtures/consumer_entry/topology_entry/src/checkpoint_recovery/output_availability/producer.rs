//! Exactly-one native Quantity output, original PlanarRead source and lifecycle.
use super::*;
pub(super) struct Family;
impl WorthQueryProducerOutputFamily<Schema> for Family {
    type Entity = members::Quantity;
    type Source = PlanarReadBinding<Schema>;
    const IDENTITY: &'static str = "availability-quantity-family-v1";
    const SUPPORTED: &'static [WorthQueryProducerApplicability] = &[
        WorthQueryProducerApplicability::new(
            "quantity",
            WorthQueryProducerLifecyclePosture::Initial,
        ),
        WorthQueryProducerApplicability::new(
            "quantity",
            WorthQueryProducerLifecyclePosture::Preserve,
        ),
    ];
    fn profile_kind(_: &PlanarReadResult) -> &'static str {
        "quantity"
    }
}
#[derive(Clone)]
pub(super) struct Demand(pub String);
impl WorthQueryApplicationOutputDemand<Schema> for Demand {
    type OutputFamily = Family;
    fn source_intent(&self) -> PlanarRead {
        PlanarRead {
            body_key: self.0.clone(),
        }
    }
}
pub(super) struct Provider;
macro_rules! producer {
    ($producer:ident,$binding:ident,$role:ident,$posture:ident,$id:literal) => {
        pub(super) struct $producer;
        impl WorthQueryApplicationProducerBinding<Schema> for $producer {
            type Operation = binding::$binding;
            type OutputFamily = Family;
            type OutputRole = binding::$role;
            type Provider = Provider;
            const IDENTITY: &'static str = concat!("availability-producer-", $id);
            const APPLICABILITY: &'static [WorthQueryProducerApplicability] =
                &[WorthQueryProducerApplicability::new(
                    "quantity",
                    WorthQueryProducerLifecyclePosture::$posture,
                )];
            const REQUIRED_INVARIANTS: &'static [WorthQueryProducerInvariantRequirement] = &[];

            const REUSE_POLICY: &'static str = "tracked-source-and-publication-facts";
        }
        impl WorthQueryApplicationProducerProvider<Schema, $producer> for Provider {
            type DenialValue = WorthQueryUndeclaredProducerDenial;
            const SEMANTIC_IDENTITY: &'static str = concat!("availability-provider-", $id);
            fn operation_input(&self, source: &PlanarReadResult) -> binding::Input {
                binding::Input {
                    scope_key: source.body_key.clone(),
                }
            }
            fn idempotency_key(&self, _: &PlanarReadResult, epoch: &[u8; 32]) -> u64 {
                planar_source_key(epoch) ^ 0x91742700
            }
        }
    };
}
producer!(Initial, Initial, Created, Initial, "initial");
producer!(Preserve, Preserve, Preserved, Preserve, "preserve");
