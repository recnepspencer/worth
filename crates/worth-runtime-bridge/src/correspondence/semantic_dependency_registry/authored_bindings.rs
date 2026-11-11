//! Borrowed declaration inventory and stable claim origin resolution.
use super::{
    AdmittedSemanticDependencyRegistry, BridgeSemanticCorrespondenceRegistration,
    BridgeSemanticDependencyCandidate,
};
impl AdmittedSemanticDependencyRegistry {
    pub(crate) fn admitted_entry(
        &self,
        key: &str,
    ) -> Option<(usize, &BridgeSemanticCorrespondenceRegistration)> {
        self.index
            .get(key)
            .map(|position| (*position, &self.admitted[*position]))
    }
    pub(crate) fn admitted_entries(
        &self,
    ) -> impl Iterator<Item = (&str, &BridgeSemanticCorrespondenceRegistration)> {
        self.index
            .iter()
            .map(|(key, position)| (key.as_str(), &self.admitted[*position]))
    }
    pub(crate) fn claim_origin(
        &self,
        candidate: &BridgeSemanticDependencyCandidate,
    ) -> crate::correspondence::BridgeSemanticClaimOrigin {
        let key = candidate.canonical_registration_key();
        match self
            .index
            .get(&key)
            .map(|position| &self.admitted[*position])
            .and_then(|row| row.recorded_claim)
        {
            Some(reference) => {
                crate::correspondence::BridgeSemanticClaimOrigin::Recorded(reference)
            }
            None => crate::correspondence::BridgeSemanticClaimOrigin::Memory(key),
        }
    }
}
