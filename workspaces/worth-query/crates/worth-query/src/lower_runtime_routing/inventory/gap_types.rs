use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};

use super::WorthQueryLowerRuntimeAuthorityOwner;
use super::WorthQueryLowerRuntimeSeamKey;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryLowerRuntimeGapRegistryRow {
    seam_key: WorthQueryLowerRuntimeSeamKey,
    concrete_seam: &'static str,
    current_shape: &'static str,
    missing_contract_owner: WorthQueryLowerRuntimeAuthorityOwner,
    missing_contract: &'static str,
    required_closeout: &'static str,
}

impl WorthQueryLowerRuntimeGapRegistryRow {
    pub fn seam_key(&self) -> WorthQueryLowerRuntimeSeamKey {
        self.seam_key
    }

    pub fn row_digest(&self) -> Result<String, IdentityDerivationDenial> {
        Ok(self.row_identity()?.as_str().to_string())
    }

    fn row_identity(&self) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
        Ok(WorthQueryEvidenceIdentity::compose(
            WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence,
        )
        .field_shape(
            WorthQueryEvidenceTag::new("identity_family"),
            "lower_runtime_gap_registry_row_v1",
        )
        .field_shape(WorthQueryEvidenceTag::new("seam"), self.seam_key.as_str())
        .field_value(
            WorthQueryEvidenceTag::new("concrete_seam"),
            self.concrete_seam,
        )
        .field_shape(
            WorthQueryEvidenceTag::new("current_shape"),
            self.current_shape,
        )
        .field_shape(
            WorthQueryEvidenceTag::new("missing_contract_owner"),
            self.missing_contract_owner.as_str(),
        )
        .field_value(
            WorthQueryEvidenceTag::new("missing_contract"),
            self.missing_contract,
        )
        .field_value(
            WorthQueryEvidenceTag::new("required_closeout"),
            self.required_closeout,
        )
        .seal()?)
    }

    pub fn required_closeout(&self) -> &'static str {
        self.required_closeout
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryLowerRuntimeGapRegistry {
    rows: &'static [WorthQueryLowerRuntimeGapRegistryRow],
}

impl WorthQueryLowerRuntimeGapRegistry {
    pub(crate) const fn new(rows: &'static [WorthQueryLowerRuntimeGapRegistryRow]) -> Self {
        Self { rows }
    }

    pub fn rows(&self) -> &'static [WorthQueryLowerRuntimeGapRegistryRow] {
        self.rows
    }

    pub fn registry_digest(&self) -> Result<String, IdentityDerivationDenial> {
        let row_identities = self
            .rows
            .iter()
            .map(WorthQueryLowerRuntimeGapRegistryRow::row_identity)
            .collect::<Vec<_>>();
        Ok(WorthQueryEvidenceIdentity::compose(
            WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence,
        )
        .field_shape(
            WorthQueryEvidenceTag::new("identity_family"),
            "lower_runtime_gap_registry_v1",
        )
        .field_evidence_identity_sequence(WorthQueryEvidenceTag::new("rows"), &row_identities)
        .seal()?
        .as_str()
        .to_string())
    }

    pub fn debt_exit_criteria_digest(&self) -> Result<String, IdentityDerivationDenial> {
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
            .collect::<Vec<_>>();
        Ok(WorthQueryEvidenceIdentity::compose(
            WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence,
        )
        .field_shape(
            WorthQueryEvidenceTag::new("identity_family"),
            "lower_runtime_gap_debt_exit_criteria_v1",
        )
        .field_evidence_identity_sequence(WorthQueryEvidenceTag::new("rows"), &closeout_identities)
        .seal()?
        .as_str()
        .to_string())
    }
}
