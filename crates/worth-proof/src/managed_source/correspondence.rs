//! A candidate-specific correspondence namespace; Native owns its derivation.
use std::sync::Arc;

pub struct ManagedIndexSourceCorrespondenceIssuer(Arc<()>);
pub struct ManagedIndexSourceCorrespondenceAuthority(Arc<()>);
pub struct AdmittedManagedIndexSourceCorrespondence(Arc<()>);

impl ManagedIndexSourceCorrespondenceIssuer {
    pub fn fresh() -> Self {
        Self(Arc::new(()))
    }
    pub fn authority(&self) -> ManagedIndexSourceCorrespondenceAuthority {
        ManagedIndexSourceCorrespondenceAuthority(Arc::clone(&self.0))
    }
    pub fn admit(&self) -> AdmittedManagedIndexSourceCorrespondence {
        AdmittedManagedIndexSourceCorrespondence(Arc::clone(&self.0))
    }
}
impl ManagedIndexSourceCorrespondenceAuthority {
    pub fn admits(&self, proof: &AdmittedManagedIndexSourceCorrespondence) -> bool {
        Arc::ptr_eq(&self.0, &proof.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn correspondence_is_bound_to_one_candidate_namespace() {
        let actual = ManagedIndexSourceCorrespondenceIssuer::fresh();
        let foreign = ManagedIndexSourceCorrespondenceIssuer::fresh();
        let authority = actual.authority();
        let proof = actual.admit();
        drop(actual);
        assert!(authority.admits(&proof));
        assert!(!authority.admits(&foreign.admit()));
        assert!(!foreign.authority().admits(&proof));
    }
}
