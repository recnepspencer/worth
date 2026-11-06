//! Sealed test doors over this module's private members.
use super::*;
use crate::domain_computation::primary_graph::vocabulary::application_attempt::fact::WorthQueryApplicationFactKey;

impl SealedComputationFacts {
    #[doc(hidden)]
    pub fn replace_fact_for_test(
        &mut self,
        key: &WorthQueryApplicationFactKey,
        fact: WorthQueryApplicationObservedFact,
    ) {
        Self::replace_fact(self, key, fact)
    }
}

impl SealedComputationFacts {
    #[doc(hidden)]
    pub fn at_seal_for_test(
        attribution: ComputationFactAttribution,
        observed: &BTreeMap<WorthQueryApplicationFactKey, WorthQueryApplicationObservedFact>,
    ) -> Self {
        Self::at_seal(attribution, observed)
    }
}

impl SealedComputationFacts {
    #[doc(hidden)]
    pub fn copy_keys_for_test(
        &mut self,
        key: &WorthQueryApplicationFactKey,
        destinations: impl IntoIterator<Item = WorthQueryApplicationFactKey>,
    ) {
        Self::copy_keys(self, key, destinations)
    }
}

impl SealedComputationFacts {
    #[doc(hidden)]
    pub fn rename_key_for_test(
        &mut self,
        key: &WorthQueryApplicationFactKey,
        to: WorthQueryApplicationFactKey,
    ) {
        Self::rename_key(self, key, to)
    }
}
