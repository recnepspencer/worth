//! Explicit installation construction is distinct from ordinary Source READ.
use std::sync::Arc;
/// Kept only by the unpublished owner of one compiled Source installation.
pub struct ManagedIndexSourceInitializationIssuer(Arc<()>);
/// Matching authority retained by that installation's actual Source services.
#[derive(Clone)]
pub struct ManagedIndexSourceInitializationAuthority(Arc<()>);
/// A matching grant still requires the runtime owner's live one-batch session.
/// It supplies no principal READ, logical/physical posture or recovery authority.
pub struct AdmittedManagedIndexSourceInitialization(Arc<()>);
impl ManagedIndexSourceInitializationIssuer {
    pub fn fresh() -> Self {
        Self(Arc::new(()))
    }
    pub fn authority(&self) -> ManagedIndexSourceInitializationAuthority {
        ManagedIndexSourceInitializationAuthority(Arc::clone(&self.0))
    }
    pub fn admit(&self) -> AdmittedManagedIndexSourceInitialization {
        AdmittedManagedIndexSourceInitialization(Arc::clone(&self.0))
    }
}
impl ManagedIndexSourceInitializationAuthority {
    pub fn admits(&self, proof: &AdmittedManagedIndexSourceInitialization) -> bool {
        Arc::ptr_eq(&self.0, &proof.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn initialization_matches_only_its_installed_owner_and_issuer_drop_is_not_revocation() {
        let issuers = [
            ManagedIndexSourceInitializationIssuer::fresh(),
            ManagedIndexSourceInitializationIssuer::fresh(),
            ManagedIndexSourceInitializationIssuer::fresh(),
        ];
        let authorities = issuers.each_ref().map(|issuer| issuer.authority());
        let grants = issuers.each_ref().map(|issuer| issuer.admit());
        drop(issuers);
        for (owner, authority) in authorities.iter().enumerate() {
            let cloned = authority.clone();
            for (source, grant) in grants.iter().enumerate() {
                assert_eq!(
                    cloned.admits(grant),
                    owner == source,
                    "matching authority stays valid; a foreign fresh issuer never opens it"
                );
            }
        }
    }
}
