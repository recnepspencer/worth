//! Sealed test doors over this module's private members.
use super::*;

impl PostingGroup {
    #[doc(hidden)]
    pub fn key_for_test(&self) -> &Arc<FactPostingKey> {
        &self.key
    }
}

impl PostingGroup {
    #[doc(hidden)]
    pub fn ordinals_for_test(&self) -> &OrdSet<usize> {
        &self.ordinals
    }
}

impl PreparedPostingOrdinals {
    #[doc(hidden)]
    pub fn ordinals_for_test(&self) -> &OrdMap<Arc<FactPostingKey>, OrdSet<usize>> {
        &self.ordinals
    }
}

impl PreparedPostingOrdinals {
    #[doc(hidden)]
    pub fn into_projection_for_test(
        self,
        id: u64,
        source_len: usize,
        output_len: usize,
        requirement: Option<FullVerificationReason>,
    ) -> Arc<super::super::PostingProjection> {
        Self::into_projection(self, id, source_len, output_len, requirement)
    }
}

impl PreparedPostingOrdinals {
    #[doc(hidden)]
    pub fn prepare_for_test(
        facts: &(impl FactSequence + ?Sized),
        output_facts: Option<&[WorthQueryApplicationObservedFact]>,
        verification_requirement: &mut Option<FullVerificationReason>,
    ) -> Result<Self, CompanionPreflightStop> {
        Self::prepare(facts, output_facts, verification_requirement)
    }
}

impl ProjectedKey {
    #[doc(hidden)]
    pub fn new_for_test(hash: u64, key: Arc<FactPostingKey>, ordinal: usize) -> Self {
        Self { hash, key, ordinal }
    }
}
