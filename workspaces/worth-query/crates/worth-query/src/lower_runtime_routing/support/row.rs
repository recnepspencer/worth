use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};

use super::super::{
    WorthQueryLowerRuntimeArtifactStrength, WorthQueryLowerRuntimeAuthorityOwner,
    WorthQueryLowerRuntimeRouteKind, WorthQueryLowerRuntimeSeamKey,
};
use super::posture::{WorthQueryLowerRuntimeSupportDetail, WorthQueryLowerRuntimeSupportPosture};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryLowerRuntimeSupportRow {
    seam_key: WorthQueryLowerRuntimeSeamKey,
    capability_label: &'static str,
    authority_owner: WorthQueryLowerRuntimeAuthorityOwner,
    route_kind: WorthQueryLowerRuntimeRouteKind,
    artifact_strength: WorthQueryLowerRuntimeArtifactStrength,
    posture: WorthQueryLowerRuntimeSupportPosture,
    detail: WorthQueryLowerRuntimeSupportDetail,
}

impl WorthQueryLowerRuntimeSupportRow {
    pub(crate) const fn new(
        seam_key: WorthQueryLowerRuntimeSeamKey,
        capability_label: &'static str,
        authority_owner: WorthQueryLowerRuntimeAuthorityOwner,
        route_kind: WorthQueryLowerRuntimeRouteKind,
        artifact_strength: WorthQueryLowerRuntimeArtifactStrength,
        posture: WorthQueryLowerRuntimeSupportPosture,
        detail: WorthQueryLowerRuntimeSupportDetail,
    ) -> Self {
        Self {
            seam_key,
            capability_label,
            authority_owner,
            route_kind,
            artifact_strength,
            posture,
            detail,
        }
    }

    pub fn seam_key(&self) -> WorthQueryLowerRuntimeSeamKey {
        self.seam_key
    }

    pub fn capability_label(&self) -> &'static str {
        self.capability_label
    }

    pub fn authority_owner(&self) -> WorthQueryLowerRuntimeAuthorityOwner {
        self.authority_owner
    }

    pub fn route_kind(&self) -> WorthQueryLowerRuntimeRouteKind {
        self.route_kind
    }

    pub fn artifact_strength(&self) -> WorthQueryLowerRuntimeArtifactStrength {
        self.artifact_strength
    }

    pub fn posture(&self) -> WorthQueryLowerRuntimeSupportPosture {
        self.posture
    }

    pub fn detail(&self) -> WorthQueryLowerRuntimeSupportDetail {
        self.detail
    }

    pub fn closeout_target(&self) -> Option<&'static str> {
        self.detail.closeout_target()
    }

    pub fn required_closeout(&self) -> Option<&'static str> {
        self.detail.required_closeout()
    }

    pub fn certification_row(&self) -> Option<&'static str> {
        self.detail.certification_row()
    }

    pub fn row_digest(&self) -> Result<String, IdentityDerivationDenial> {
        let mut identity = WorthQueryEvidenceIdentity::compose(
            WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence,
        )
        .field_shape(
            WorthQueryEvidenceTag::new("identity_family"),
            "lower_runtime_support_row_v1",
        )
        .field_shape(WorthQueryEvidenceTag::new("seam"), self.seam_key.as_str())
        .field_shape(
            WorthQueryEvidenceTag::new("capability"),
            self.capability_label,
        )
        .field_shape(
            WorthQueryEvidenceTag::new("owner"),
            self.authority_owner.as_str(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("route_kind"),
            self.route_kind.as_str(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("artifact"),
            self.artifact_strength.as_str(),
        )
        .field_shape(WorthQueryEvidenceTag::new("posture"), self.posture.as_str());
        match self.detail {
            WorthQueryLowerRuntimeSupportDetail::Crossing => {
                identity = identity.field_shape(WorthQueryEvidenceTag::new("detail"), "crossing");
            }
            WorthQueryLowerRuntimeSupportDetail::Closeout {
                closeout_target,
                required_closeout,
                certification_row,
            } => {
                identity = identity
                    .field_shape(WorthQueryEvidenceTag::new("detail"), "closeout")
                    .field_value(
                        WorthQueryEvidenceTag::new("closeout_target"),
                        closeout_target,
                    )
                    .field_value(
                        WorthQueryEvidenceTag::new("required_closeout"),
                        required_closeout,
                    )
                    .field_value(
                        WorthQueryEvidenceTag::new("certification_row"),
                        certification_row,
                    );
            }
        }
        Ok(identity.seal()?.as_str().to_string())
    }
}
