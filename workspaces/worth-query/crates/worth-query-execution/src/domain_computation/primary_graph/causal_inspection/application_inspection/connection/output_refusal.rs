//! Descriptive output join failures, carrying no field identities or values.

/// The first failed condition after the longest satisfied output-join prefix.
/// Only diagnostically disclosed observed fields participate. With several
/// candidates, this describes the furthest candidate, not every failed pair;
/// ties retain original role/footprint order. It grants no authority.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryProducerOutputConnectionDenial {
    /// No non-retired role / disclosed observed-field pair was available.
    NoDisclosedOutputCandidate,
    OutputKindMismatch,
    OutputEntityMismatch,
    IncompleteObservation,
    NativeRevisionUnavailable,
    NativeFieldAbsent,
    NativeRevisionNotCreatedByProducer,
    InstalledFieldUnavailable,
    ContractRevisionMismatch,
    CanonicalFieldCleared,
    CanonicalAspectCleared,
    CanonicalFieldUnchanged,
    CanonicalEffectUnsupported,
    CanonicalEffectAmbiguous,
}

impl WorthQueryProducerOutputConnectionDenial {
    fn prefix(self) -> u8 {
        use WorthQueryProducerOutputConnectionDenial::*;
        match self {
            NoDisclosedOutputCandidate => 0,
            OutputKindMismatch => 1,
            OutputEntityMismatch => 2,
            IncompleteObservation => 3,
            NativeRevisionUnavailable => 4,
            NativeFieldAbsent => 5,
            NativeRevisionNotCreatedByProducer => 6,
            InstalledFieldUnavailable => 7,
            ContractRevisionMismatch => 8,
            CanonicalFieldCleared
            | CanonicalAspectCleared
            | CanonicalFieldUnchanged
            | CanonicalEffectUnsupported
            | CanonicalEffectAmbiguous => 9,
        }
    }

    pub(super) fn include(&mut self, candidate: Self) {
        if candidate.prefix() > self.prefix() {
            *self = candidate;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::WorthQueryProducerOutputConnectionDenial::*;

    #[test]
    fn output_refusal_keeps_the_furthest_join_and_first_equally_deep_candidate() {
        let mut cause = NoDisclosedOutputCandidate;
        cause.include(ContractRevisionMismatch);
        cause.include(NativeRevisionNotCreatedByProducer);
        assert_eq!(cause, ContractRevisionMismatch);
        cause.include(CanonicalEffectAmbiguous);
        cause.include(CanonicalFieldUnchanged);
        assert_eq!(cause, CanonicalEffectAmbiguous);
        let mut reversed = NativeRevisionNotCreatedByProducer;
        reversed.include(ContractRevisionMismatch);
        assert_eq!(reversed, ContractRevisionMismatch);
    }
}
