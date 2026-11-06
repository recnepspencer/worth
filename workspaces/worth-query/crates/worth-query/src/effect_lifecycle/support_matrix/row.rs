use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::{WorthQueryEvidenceIdentity, WorthQueryEvidenceTag};

use super::super::inventory::{EffectLoweredArtifactKind, EffectReceiptArtifactKind};
use super::super::planning::EffectAuthorityOwner;
use super::super::support_contract::{
    deferred_support_contract, support_deferred_neighbors, support_denial_kinds,
    EffectDeferredNeighborFamily, EffectDeferredSupportContract,
};
use super::super::taxonomy::{DeniedEffectEligibilityKind, EffectFamily};
use super::{EffectSupportCause, EffectSupportPosture, EFFECT_LIFECYCLE_IDENTITY_SCOPE};
use crate::basis_lifecycle::BasisFamily;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EffectLifecycleSupportRow {
    basis_family: BasisFamily,
    effect_family: EffectFamily,
    authority_owner: EffectAuthorityOwner,
    lowered_artifact_kind: EffectLoweredArtifactKind,
    receipt_artifact_kind: EffectReceiptArtifactKind,
    posture: EffectSupportPosture,
    cause: EffectSupportCause,
    row_identity: WorthQueryEvidenceIdentity,
}

impl EffectLifecycleSupportRow {
    pub(crate) fn new(
        basis_family: BasisFamily,
        effect_family: EffectFamily,
        authority_owner: EffectAuthorityOwner,
        lowered_artifact_kind: EffectLoweredArtifactKind,
        receipt_artifact_kind: EffectReceiptArtifactKind,
        posture: EffectSupportPosture,
        cause: EffectSupportCause,
    ) -> Result<Self, IdentityDerivationDenial> {
        let row_identity = WorthQueryEvidenceIdentity::compose(EFFECT_LIFECYCLE_IDENTITY_SCOPE)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "effect_lifecycle_support_row_v1",
            )
            .field_shape(
                WorthQueryEvidenceTag::new("basis_family"),
                basis_family.as_str(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("effect_family"),
                effect_family.as_str(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("authority_owner"),
                authority_owner.as_str(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("lowered_artifact"),
                lowered_artifact_kind.as_str(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("receipt_artifact"),
                receipt_artifact_kind.as_str(),
            )
            .field_shape(WorthQueryEvidenceTag::new("posture"), posture.as_str())
            .field_shape(WorthQueryEvidenceTag::new("cause"), cause.as_str())
            .seal()?;
        Ok(Self {
            basis_family,
            effect_family,
            authority_owner,
            lowered_artifact_kind,
            receipt_artifact_kind,
            posture,
            cause,
            row_identity,
        })
    }

    pub fn basis_family(&self) -> BasisFamily {
        self.basis_family
    }

    pub fn effect_family(&self) -> EffectFamily {
        self.effect_family
    }

    pub fn authority_owner(&self) -> EffectAuthorityOwner {
        self.authority_owner
    }

    pub fn lowered_artifact_kind(&self) -> EffectLoweredArtifactKind {
        self.lowered_artifact_kind
    }

    pub fn receipt_artifact_kind(&self) -> EffectReceiptArtifactKind {
        self.receipt_artifact_kind
    }

    pub fn posture(&self) -> EffectSupportPosture {
        self.posture
    }

    pub fn cause(&self) -> EffectSupportCause {
        self.cause
    }

    pub fn requires_rebind(&self) -> bool {
        self.posture == EffectSupportPosture::RebindRequired
    }

    pub fn denial_kinds(&self) -> &'static [DeniedEffectEligibilityKind] {
        support_denial_kinds(self.posture, self.cause)
    }

    pub fn deferred_neighbors(&self) -> &'static [EffectDeferredNeighborFamily] {
        support_deferred_neighbors(self.effect_family)
    }

    pub fn deferred_contract(
        &self,
    ) -> Result<Option<EffectDeferredSupportContract>, IdentityDerivationDenial> {
        Ok(deferred_support_contract(self.cause)?)
    }

    pub fn row_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.row_identity
    }

    pub fn row_for_reporting(&self) -> &str {
        self.row_identity.as_str()
    }
}
