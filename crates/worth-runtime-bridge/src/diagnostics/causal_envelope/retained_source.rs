//! Exact Native custody, separate from descriptive envelope identities.
use crate::conditional_execution::BridgeConditionalCausalObservation;

#[derive(Clone, Debug, Default)]
pub(super) struct RetainedNativeObservation(pub(super) Option<BridgeConditionalCausalObservation>);

impl PartialEq for RetainedNativeObservation {
    fn eq(&self, other: &Self) -> bool {
        match (&self.0, &other.0) {
            (None, None) => true,
            (Some(left), Some(right)) => left.same_observation(right),
            _ => false,
        }
    }
}

impl Eq for RetainedNativeObservation {}
