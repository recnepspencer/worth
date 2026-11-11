use crate::evidence_identity::WorthQueryEvidenceIdentity;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryImpactSemanticProjection {
    identity: WorthQueryEvidenceIdentity,
    class: crate::domain_installation::WorthQueryImpactClass,
    affected_roles: Vec<crate::domain_installation::WorthQuerySemanticDependencyRole>,
    affected_dependency_count: usize,
}

impl WorthQueryImpactSemanticProjection {
    pub const fn class(&self) -> crate::domain_installation::WorthQueryImpactClass {
        self.class
    }

    pub fn affected_roles(
        &self,
    ) -> &[crate::domain_installation::WorthQuerySemanticDependencyRole] {
        &self.affected_roles
    }

    pub const fn affected_dependency_count(&self) -> usize {
        self.affected_dependency_count
    }

    pub fn canonical_bytes(&self) -> &[u8; 32] {
        self.identity.canonical_digest().value().bytes()
    }

    pub(super) const fn identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.identity
    }
}

impl crate::domain_installation::WorthQueryImpactDecision {
    pub fn semantic_projection(
        &self,
    ) -> Result<WorthQueryImpactSemanticProjection, IdentityDerivationDenial> {
        let roles = self.affected_roles().to_vec();
        let identity = super::encoding::impact_identity(
            self.class(),
            &roles,
            self.affected_dependency_count(),
        )?;
        Ok(WorthQueryImpactSemanticProjection {
            identity,
            class: self.class(),
            affected_roles: roles,
            affected_dependency_count: self.affected_dependency_count(),
        })
    }
}
