//! An installed Source audience is distinct from public principal admission.
use std::sync::Arc;

/// Kept by the Native installation that validates and admits Source loans.
#[derive(Debug)]
pub struct ManagedIndexSourceAudienceIssuer(Arc<()>);

/// Bound into the compiled Source receiver. Names and numeric IDs cannot
/// reconstruct this occurrence, including after an independent installation.
#[derive(Clone, Debug)]
pub struct ManagedIndexSourceAudienceAuthority(Arc<()>);

/// Admission of the exact installed Source audience, not a transferable
/// execution request, public READ grant, or permission to select a World.
#[derive(Debug)]
pub struct AdmittedManagedIndexSourceAudience(Arc<()>);

impl ManagedIndexSourceAudienceIssuer {
    pub fn fresh() -> Self {
        Self(Arc::new(()))
    }

    pub fn authority(&self) -> ManagedIndexSourceAudienceAuthority {
        ManagedIndexSourceAudienceAuthority(Arc::clone(&self.0))
    }

    pub fn admit(&self) -> AdmittedManagedIndexSourceAudience {
        AdmittedManagedIndexSourceAudience(Arc::clone(&self.0))
    }
}

impl ManagedIndexSourceAudienceAuthority {
    pub fn admits(&self, evidence: &AdmittedManagedIndexSourceAudience) -> bool {
        Arc::ptr_eq(&self.0, &evidence.0)
    }
}
