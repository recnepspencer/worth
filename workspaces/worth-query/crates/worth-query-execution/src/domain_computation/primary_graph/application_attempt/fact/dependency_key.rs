use worth_foundational::facade::{
    prepare_aspect_value_identity_basis, CanonicalAspectValueIdentityBasis,
};

use super::WorthQueryApplicationObservedFact;

/// Storage identity for retained observations. Descriptive legacy locators
/// alone do not distinguish separate predicates against the same index.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(in crate::domain_computation::primary_graph) struct WorthQueryApplicationFactStorageKey {
    locator: String,
    predicate: Option<(CanonicalAspectValueIdentityBasis, usize)>,
}

impl WorthQueryApplicationObservedFact {
    /// Exact predicate-aware key for provider ports that carry string locators.
    /// The descriptive locator and canonical dependency encoding stay unchanged.
    pub(in crate::domain_computation) fn dependency_locator_identity(&self) -> String {
        match self {
            Self::IndexedEntitySelection {
                value,
                candidate_limit,
                ..
            } => {
                let predicate = prepare_aspect_value_identity_basis(value);
                format!(
                    "{}:predicate#{}:{}:limit#{}",
                    self.locator_identity(),
                    predicate.as_str().len(),
                    predicate.as_str(),
                    candidate_limit
                )
            }
            _ => self.locator_identity(),
        }
    }

    pub(in crate::domain_computation::primary_graph) fn dependency_key(
        &self,
    ) -> WorthQueryApplicationFactStorageKey {
        let predicate = match self {
            Self::IndexedEntitySelection {
                value,
                candidate_limit,
                ..
            } => Some((prepare_aspect_value_identity_basis(value), *candidate_limit)),
            _ => None,
        };
        WorthQueryApplicationFactStorageKey {
            locator: self.locator_identity(),
            predicate,
        }
    }
}
