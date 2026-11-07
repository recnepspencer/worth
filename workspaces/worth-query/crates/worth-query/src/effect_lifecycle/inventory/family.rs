use crate::basis_lifecycle::BasisFamily;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::{WorthQueryEvidenceIdentity, WorthQueryEvidenceTag};

use super::super::planning::EffectAuthorityOwner;
use super::super::support_matrix::EffectSupportPosture;
use super::kinds::{
    EffectLifecycleFamilyKey, EffectLoweredArtifactKind, EffectReceiptArtifactKind,
};
use super::EFFECT_LIFECYCLE_IDENTITY_SCOPE;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EffectLifecycleFamilyInventoryRow {
    family_key: EffectLifecycleFamilyKey,
    authority_owner: EffectAuthorityOwner,
    admitted_basis_families: Vec<BasisFamily>,
    lowered_artifact_kind: EffectLoweredArtifactKind,
    receipt_artifact_kind: EffectReceiptArtifactKind,
    denial_posture: EffectSupportPosture,
    deferred_posture: EffectSupportPosture,
    row_identity: WorthQueryEvidenceIdentity,
}

impl EffectLifecycleFamilyInventoryRow {
    pub(in crate::effect_lifecycle) fn new(
        family_key: EffectLifecycleFamilyKey,
        authority_owner: EffectAuthorityOwner,
        admitted_basis_families: Vec<BasisFamily>,
        lowered_artifact_kind: EffectLoweredArtifactKind,
        receipt_artifact_kind: EffectReceiptArtifactKind,
        denial_posture: EffectSupportPosture,
        deferred_posture: EffectSupportPosture,
    ) -> Result<Self, IdentityDerivationDenial> {
        let basis_identities = admitted_basis_families
            .iter()
            .map(|basis| {
                WorthQueryEvidenceIdentity::compose(EFFECT_LIFECYCLE_IDENTITY_SCOPE)
                    .field_shape(
                        WorthQueryEvidenceTag::new("identity_family"),
                        "effect_lifecycle_family_inventory_basis_v1",
                    )
                    .field_shape(WorthQueryEvidenceTag::new("basis"), basis.as_str())
                    .seal()
            })
            .collect::<Result<Vec<_>, _>>()?;
        let row_identity = WorthQueryEvidenceIdentity::compose(EFFECT_LIFECYCLE_IDENTITY_SCOPE)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "effect_lifecycle_family_inventory_row_v1",
            )
            .field_shape(WorthQueryEvidenceTag::new("family"), family_key.as_str())
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
            .field_shape(
                WorthQueryEvidenceTag::new("denial_posture"),
                denial_posture.as_str(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("deferred_posture"),
                deferred_posture.as_str(),
            )
            .field_evidence_identity_sequence(
                WorthQueryEvidenceTag::new("admitted_basis"),
                &basis_identities,
            )
            .seal()?;
        Ok(Self {
            family_key,
            authority_owner,
            admitted_basis_families,
            lowered_artifact_kind,
            receipt_artifact_kind,
            denial_posture,
            deferred_posture,
            row_identity,
        })
    }

    pub fn family_key(&self) -> EffectLifecycleFamilyKey {
        self.family_key
    }
    pub fn authority_owner(&self) -> EffectAuthorityOwner {
        self.authority_owner
    }
    pub fn admitted_basis_families(&self) -> &[BasisFamily] {
        &self.admitted_basis_families
    }
    pub fn lowered_artifact_kind(&self) -> EffectLoweredArtifactKind {
        self.lowered_artifact_kind
    }
    pub fn receipt_artifact_kind(&self) -> EffectReceiptArtifactKind {
        self.receipt_artifact_kind
    }
    pub fn denial_posture(&self) -> EffectSupportPosture {
        self.denial_posture
    }
    pub fn deferred_posture(&self) -> EffectSupportPosture {
        self.deferred_posture
    }
    pub fn row_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.row_identity
    }
    pub fn row_for_reporting(&self) -> &str {
        self.row_identity.as_str()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EffectLifecycleFamilyInventory {
    pub(super) rows: Vec<EffectLifecycleFamilyInventoryRow>,
    pub(super) inventory_identity: WorthQueryEvidenceIdentity,
}

impl EffectLifecycleFamilyInventory {
    pub fn rows(&self) -> &[EffectLifecycleFamilyInventoryRow] {
        &self.rows
    }
    pub fn inventory_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.inventory_identity
    }
    pub fn inventory_for_reporting(&self) -> &str {
        self.inventory_identity.as_str()
    }
}
