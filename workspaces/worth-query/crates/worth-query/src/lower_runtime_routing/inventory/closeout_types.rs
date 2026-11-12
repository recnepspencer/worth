use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};

use super::{
    WorthQueryLowerRuntimeAuthorityOwner, WorthQueryLowerRuntimeRouteKind,
    WorthQueryLowerRuntimeSeamKey,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryLowerRuntimeCloseoutPosture {
    SeamEliminated,
    DeferredNeighbor,
}

impl WorthQueryLowerRuntimeCloseoutPosture {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::SeamEliminated => "seam-eliminated",
            Self::DeferredNeighbor => "deferred-neighbor",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryLowerRuntimeCloseoutRow {
    seam_key: WorthQueryLowerRuntimeSeamKey,
    capability_label: &'static str,
    posture: WorthQueryLowerRuntimeCloseoutPosture,
    owner: WorthQueryLowerRuntimeAuthorityOwner,
    route_kind: WorthQueryLowerRuntimeRouteKind,
    closeout_target: &'static str,
    required_closeout: &'static str,
    certification_row: &'static str,
}

impl WorthQueryLowerRuntimeCloseoutRow {
    pub(crate) const fn new(
        seam_key: WorthQueryLowerRuntimeSeamKey,
        capability_label: &'static str,
        posture: WorthQueryLowerRuntimeCloseoutPosture,
        owner: WorthQueryLowerRuntimeAuthorityOwner,
        route_kind: WorthQueryLowerRuntimeRouteKind,
        closeout_target: &'static str,
        required_closeout: &'static str,
        certification_row: &'static str,
    ) -> Self {
        Self {
            seam_key,
            capability_label,
            posture,
            owner,
            route_kind,
            closeout_target,
            required_closeout,
            certification_row,
        }
    }

    pub fn seam_key(&self) -> WorthQueryLowerRuntimeSeamKey {
        self.seam_key
    }

    pub fn capability_label(&self) -> &'static str {
        self.capability_label
    }

    pub fn posture(&self) -> WorthQueryLowerRuntimeCloseoutPosture {
        self.posture
    }

    pub fn owner(&self) -> WorthQueryLowerRuntimeAuthorityOwner {
        self.owner
    }

    pub fn route_kind(&self) -> WorthQueryLowerRuntimeRouteKind {
        self.route_kind
    }

    pub fn closeout_target(&self) -> &'static str {
        self.closeout_target
    }

    pub fn required_closeout(&self) -> &'static str {
        self.required_closeout
    }

    pub fn certification_row(&self) -> &'static str {
        self.certification_row
    }

    pub fn row_digest(&self) -> Result<String, IdentityDerivationDenial> {
        Ok(self.row_identity()?.as_str().to_string())
    }

    fn row_identity(&self) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "lower_runtime_closeout_row_v1",
            )
            .field_shape(WorthQueryEvidenceTag::new("seam"), self.seam_key.as_str())
            .field_shape(
                WorthQueryEvidenceTag::new("capability"),
                self.capability_label,
            )
            .field_shape(WorthQueryEvidenceTag::new("posture"), self.posture.as_str())
            .field_shape(WorthQueryEvidenceTag::new("owner"), self.owner.as_str())
            .field_shape(
                WorthQueryEvidenceTag::new("route_kind"),
                self.route_kind.as_str(),
            )
            .field_value(
                WorthQueryEvidenceTag::new("closeout_target"),
                self.closeout_target,
            )
            .field_value(
                WorthQueryEvidenceTag::new("required_closeout"),
                self.required_closeout,
            )
            .field_value(
                WorthQueryEvidenceTag::new("certification_row"),
                self.certification_row,
            )
            .seal()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryLowerRuntimeCloseoutRegistry {
    rows: &'static [WorthQueryLowerRuntimeCloseoutRow],
}

impl WorthQueryLowerRuntimeCloseoutRegistry {
    pub(crate) const fn new(rows: &'static [WorthQueryLowerRuntimeCloseoutRow]) -> Self {
        Self { rows }
    }

    pub fn rows(&self) -> &'static [WorthQueryLowerRuntimeCloseoutRow] {
        self.rows
    }

    pub fn registry_digest(&self) -> Result<String, IdentityDerivationDenial> {
        let row_identities = self
            .rows
            .iter()
            .map(WorthQueryLowerRuntimeCloseoutRow::row_identity)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(WorthQueryEvidenceIdentity::compose(
            WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence,
        )
        .field_shape(
            WorthQueryEvidenceTag::new("identity_family"),
            "lower_runtime_closeout_registry_v1",
        )
        .field_evidence_identity_sequence(WorthQueryEvidenceTag::new("rows"), &row_identities)
        .seal()?
        .as_str()
        .to_string())
    }

    pub fn required_closeout_digest(&self) -> Result<String, IdentityDerivationDenial> {
        let closeout_identities = self
            .rows
            .iter()
            .map(|row| {
                WorthQueryEvidenceIdentity::compose(
                    WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence,
                )
                .field_value(
                    WorthQueryEvidenceTag::new("required_closeout"),
                    row.required_closeout(),
                )
                .seal()
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(WorthQueryEvidenceIdentity::compose(
            WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence,
        )
        .field_shape(
            WorthQueryEvidenceTag::new("identity_family"),
            "lower_runtime_required_closeout_v1",
        )
        .field_evidence_identity_sequence(WorthQueryEvidenceTag::new("rows"), &closeout_identities)
        .seal()?
        .as_str()
        .to_string())
    }
}
