use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};

use super::WorthQueryConsumerInvalidationSemanticProjection;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryInvalidationCompatibilityOutcome {
    SingletonContinuity,
    SharedEquivalentContinuity,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryAdmittedInvalidationSemanticProjection {
    identity: WorthQueryEvidenceIdentity,
    semantic: WorthQueryConsumerInvalidationSemanticProjection,
    compatibility: WorthQueryInvalidationCompatibilityOutcome,
}

impl WorthQueryAdmittedInvalidationSemanticProjection {
    pub const fn semantic(&self) -> &WorthQueryConsumerInvalidationSemanticProjection {
        &self.semantic
    }

    pub const fn compatibility(&self) -> WorthQueryInvalidationCompatibilityOutcome {
        self.compatibility
    }

    pub fn canonical_bytes(&self) -> &[u8; 32] {
        self.identity.canonical_digest().value().bytes()
    }
}

impl super::super::WorthQueryAdmittedConsumerInvalidation<'_> {
    pub fn admitted_semantic_projection(
        &self,
        workspace: &crate::runtime::WorthQueryWorkspace,
    ) -> Result<Option<WorthQueryAdmittedInvalidationSemanticProjection>, IdentityDerivationDenial>
    {
        if !self.remains_current(workspace) {
            return Ok(None);
        }
        let semantic = self.delta().semantic_projection()?;
        let compatibility = match self.delta().compatibility_continuity() {
            crate::domain_installation::operation_execution::WorthQueryProjectionSharingContinuity::Singleton => {
                WorthQueryInvalidationCompatibilityOutcome::SingletonContinuity
            }
            crate::domain_installation::operation_execution::WorthQueryProjectionSharingContinuity::Equivalent => {
                WorthQueryInvalidationCompatibilityOutcome::SharedEquivalentContinuity
            }
        };
        let identity =
            WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::ConsumerInvalidationDelta)
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("semantic-delta"),
                    semantic.identity(),
                )
                .field_value(
                    WorthQueryEvidenceTag::new("compatibility"),
                    compatibility_name(compatibility),
                )
                .seal()?;
        Ok(Some(WorthQueryAdmittedInvalidationSemanticProjection {
            identity,
            semantic,
            compatibility,
        }))
    }
}

fn compatibility_name(value: WorthQueryInvalidationCompatibilityOutcome) -> &'static str {
    match value {
        WorthQueryInvalidationCompatibilityOutcome::SingletonContinuity => "singleton",
        WorthQueryInvalidationCompatibilityOutcome::SharedEquivalentContinuity => {
            "shared-equivalent"
        }
    }
}
