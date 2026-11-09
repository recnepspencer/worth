use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceIdentity, WorthQueryEvidenceScope,
    WorthQueryEvidenceTag,
};
use crate::runtime::{WorthQueryAuthorityLane, WorthQueryEffectPolicy};
use crate::session_label::WorthQuerySessionLabel;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryBasisAdmissionEvidenceRow {
    kind: &'static str,
    value: String,
    row_digest: WorthQueryEvidenceIdentity,
}

impl WorthQueryBasisAdmissionEvidenceRow {
    pub fn tagged(
        kind: &'static str,
        value: impl Into<String>,
    ) -> Result<Self, IdentityDerivationDenial> {
        let value = value.into();
        let row_digest =
            worth_query_evidence_identity(WorthQueryEvidenceScope::BasisAdmissionEvidenceRow)
                .field_shape(WorthQueryEvidenceTag::new("kind"), kind)
                .field_value(WorthQueryEvidenceTag::new("value"), value.as_str())
                .seal()?;
        Ok(Self {
            kind,
            value,
            row_digest,
        })
    }

    pub fn support_profile_token(
        token: impl Into<String>,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self::tagged("support-profile-evidence", token)?)
    }

    pub fn rows_from_values(
        values: impl IntoIterator<Item = impl Into<String>>,
    ) -> Result<Vec<Self>, IdentityDerivationDenial> {
        Ok(values
            .into_iter()
            .map(|value| Self::tagged("basis-admission-evidence", value))
            .collect::<Result<Vec<_>, _>>()?)
    }

    pub fn kind(&self) -> &'static str {
        self.kind
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn row_digest(&self) -> &WorthQueryEvidenceIdentity {
        &self.row_digest
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryPreviewBasisAdmission {
    label: WorthQuerySessionLabel,
    effect_policy: WorthQueryEffectPolicy,
    authority_lane: WorthQueryAuthorityLane,
    evidence_rows: Vec<WorthQueryBasisAdmissionEvidenceRow>,
    admission_identity: WorthQueryEvidenceIdentity,
}

impl WorthQueryPreviewBasisAdmission {
    pub fn new(
        _authority: &super::WorthQueryRuntimeEvidenceAuthority,
        label: WorthQuerySessionLabel,
        effect_policy: WorthQueryEffectPolicy,
        evidence_rows: impl IntoIterator<Item = WorthQueryBasisAdmissionEvidenceRow>,
    ) -> Result<Self, IdentityDerivationDenial> {
        let evidence_rows = evidence_rows.into_iter().collect::<Vec<_>>();
        let authority_lane = WorthQueryAuthorityLane::PreviewTruth;
        let admission_identity = basis_admission_identity(
            WorthQueryEvidenceScope::PreviewBasisAdmission,
            &label,
            effect_policy,
            authority_lane,
            &evidence_rows,
        )?;
        Ok(Self {
            label,
            effect_policy,
            authority_lane,
            evidence_rows,
            admission_identity,
        })
    }

    pub fn label(&self) -> &str {
        self.label.display()
    }

    pub fn session_label(&self) -> &WorthQuerySessionLabel {
        &self.label
    }

    pub fn label_identity(&self) -> &WorthQueryEvidenceIdentity {
        self.label.identity_digest()
    }

    pub fn effect_policy(&self) -> WorthQueryEffectPolicy {
        self.effect_policy
    }

    pub fn authority_lane(&self) -> WorthQueryAuthorityLane {
        self.authority_lane
    }

    pub fn evidence_rows(&self) -> &[WorthQueryBasisAdmissionEvidenceRow] {
        &self.evidence_rows
    }

    pub fn evidence(&self) -> Vec<String> {
        self.evidence_rows
            .iter()
            .map(|row| row.value().to_string())
            .collect()
    }

    pub fn admission_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.admission_identity
    }

    pub fn admission_digest(&self) -> &WorthQueryEvidenceIdentity {
        self.admission_identity()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryBranchBasisAdmission {
    label: WorthQuerySessionLabel,
    effect_policy: WorthQueryEffectPolicy,
    authority_lane: WorthQueryAuthorityLane,
    evidence_rows: Vec<WorthQueryBasisAdmissionEvidenceRow>,
    admission_identity: WorthQueryEvidenceIdentity,
}

impl WorthQueryBranchBasisAdmission {
    pub fn new(
        _authority: &super::WorthQueryRuntimeEvidenceAuthority,
        label: WorthQuerySessionLabel,
        effect_policy: WorthQueryEffectPolicy,
        evidence_rows: impl IntoIterator<Item = WorthQueryBasisAdmissionEvidenceRow>,
    ) -> Result<Self, IdentityDerivationDenial> {
        let evidence_rows = evidence_rows.into_iter().collect::<Vec<_>>();
        let authority_lane = WorthQueryAuthorityLane::BranchLocalTruth;
        let admission_identity = basis_admission_identity(
            WorthQueryEvidenceScope::BranchBasisAdmission,
            &label,
            effect_policy,
            authority_lane,
            &evidence_rows,
        )?;
        Ok(Self {
            label,
            effect_policy,
            authority_lane,
            evidence_rows,
            admission_identity,
        })
    }

    pub fn label(&self) -> &str {
        self.label.display()
    }

    pub fn session_label(&self) -> &WorthQuerySessionLabel {
        &self.label
    }

    pub fn label_identity(&self) -> &WorthQueryEvidenceIdentity {
        self.label.identity_digest()
    }

    pub fn effect_policy(&self) -> WorthQueryEffectPolicy {
        self.effect_policy
    }

    pub fn authority_lane(&self) -> WorthQueryAuthorityLane {
        self.authority_lane
    }

    pub fn evidence_rows(&self) -> &[WorthQueryBasisAdmissionEvidenceRow] {
        &self.evidence_rows
    }

    pub fn evidence(&self) -> Vec<String> {
        self.evidence_rows
            .iter()
            .map(|row| row.value().to_string())
            .collect()
    }

    pub fn admission_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.admission_identity
    }

    pub fn admission_digest(&self) -> &WorthQueryEvidenceIdentity {
        self.admission_identity()
    }
}

fn basis_admission_identity(
    scope: WorthQueryEvidenceScope,
    label: &WorthQuerySessionLabel,
    effect_policy: WorthQueryEffectPolicy,
    authority_lane: WorthQueryAuthorityLane,
    evidence_rows: &[WorthQueryBasisAdmissionEvidenceRow],
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(worth_query_evidence_identity(scope)
        .field_value(
            WorthQueryEvidenceTag::new("session_label_identity"),
            label.identity_digest().reporting_projection(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("effect_policy"),
            effect_policy.as_str(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("authority_lane"),
            authority_lane.as_str(),
        )
        .field_value_sequence(
            WorthQueryEvidenceTag::new("basis_evidence"),
            evidence_rows
                .iter()
                .map(|row| row.row_digest().reporting_projection()),
        )
        .seal()?)
}
