use crate::domain_capabilities::certification::worth_query_domain_capability_public_surface_inventory;
use crate::domain_capabilities::identity::compose_certification_surface_digest;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryDomainCapabilityCertificationSurface {
    public_surface_digest: String,
    certification_surface_digest: String,
    category_count: usize,
}

impl WorthQueryDomainCapabilityCertificationSurface {
    pub(crate) fn new(
        public_surface_digest: String,
        category_count: usize,
    ) -> Result<Self, IdentityDerivationDenial> {
        let certification_surface_digest =
            compose_certification_surface_digest(&public_surface_digest, category_count)?;
        Ok(Self {
            public_surface_digest,
            certification_surface_digest,
            category_count,
        })
    }

    pub fn public_surface_digest(&self) -> &str {
        &self.public_surface_digest
    }

    pub fn certification_surface_digest(&self) -> &str {
        &self.certification_surface_digest
    }

    pub fn category_count(&self) -> usize {
        self.category_count
    }
}

pub fn worth_query_domain_capability_certification_surface(
) -> Result<WorthQueryDomainCapabilityCertificationSurface, IdentityDerivationDenial> {
    let inventory = worth_query_domain_capability_public_surface_inventory();
    Ok(WorthQueryDomainCapabilityCertificationSurface::new(
        inventory.public_surface_digest(),
        inventory.rows().len(),
    )?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn certification_surface_reuses_public_inventory() -> Result<(), IdentityDerivationDenial> {
        let inventory = worth_query_domain_capability_public_surface_inventory();
        let surface = worth_query_domain_capability_certification_surface()?;

        assert_eq!(
            surface.public_surface_digest(),
            inventory.public_surface_digest()
        );
        assert_eq!(surface.category_count(), inventory.rows().len());
        assert!(!surface.certification_surface_digest().is_empty());

        Ok(())
    }
}
