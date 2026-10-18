use worth_proof::{AuthorityMarker, AuthorityProves};

use super::super::CanonicalProductionReadinessCertified;

#[derive(Debug, PartialEq, Eq)]
pub struct CanonicalProductionReadinessAuthority(());

impl CanonicalProductionReadinessAuthority {
    pub(super) const fn certification_boundary() -> Self {
        Self(())
    }
}

impl AuthorityMarker for CanonicalProductionReadinessAuthority {}
impl AuthorityProves<CanonicalProductionReadinessCertified>
    for CanonicalProductionReadinessAuthority
{
}
