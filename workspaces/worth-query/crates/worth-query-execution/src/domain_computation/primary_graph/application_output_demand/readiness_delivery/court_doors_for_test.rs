//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryOutputReadinessDeliveryEvidence {
    #[doc(hidden)]
    pub fn causal_observation_for_test(
        &self) -> Option<&worth_runtime_bridge::facade::BridgeConditionalCausalObservation> {
        Self::causal_observation(self)
    }
}

impl WorthQueryOutputReadinessDeliveryEvidence {
    #[doc(hidden)]
    pub fn for_test_for_test() -> Self {
        Self::for_test()
    }
}
