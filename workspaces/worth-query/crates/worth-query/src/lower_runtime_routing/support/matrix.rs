use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};

use super::super::{
    worth_query_lower_runtime_closeout_registry, worth_query_lower_runtime_crossing_inventory,
    WorthQueryLowerRuntimeArtifactStrength, WorthQueryLowerRuntimeSeamKey,
};
use super::posture::{
    support_posture_for_classification, support_posture_for_closeout,
    WorthQueryLowerRuntimeSupportDetail,
};
use super::row::WorthQueryLowerRuntimeSupportRow;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryLowerRuntimeSupportMatrix {
    rows: Vec<WorthQueryLowerRuntimeSupportRow>,
}

impl WorthQueryLowerRuntimeSupportMatrix {
    pub(crate) fn new(rows: Vec<WorthQueryLowerRuntimeSupportRow>) -> Self {
        Self { rows }
    }

    pub fn rows(&self) -> &[WorthQueryLowerRuntimeSupportRow] {
        &self.rows
    }

    pub fn support_for(
        &self,
        seam_key: WorthQueryLowerRuntimeSeamKey,
    ) -> Option<&WorthQueryLowerRuntimeSupportRow> {
        self.rows.iter().find(|row| row.seam_key() == seam_key)
    }

    pub fn matrix_digest(&self) -> Result<String, IdentityDerivationDenial> {
        let row_identities = self
            .rows
            .iter()
            .map(|row| {
                WorthQueryEvidenceIdentity::compose(
                    WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence,
                )
                .field_value(WorthQueryEvidenceTag::new("support_row"), row.row_digest())
                .seal()
            })
            .collect::<Vec<_>>();
        Ok(WorthQueryEvidenceIdentity::compose(
            WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence,
        )
        .field_shape(
            WorthQueryEvidenceTag::new("identity_family"),
            "lower_runtime_support_matrix_v1",
        )
        .field_evidence_identity_sequence(WorthQueryEvidenceTag::new("rows"), &row_identities)
        .seal()?
        .as_str()
        .to_string())
    }
}

pub fn worth_query_lower_runtime_support_matrix() -> WorthQueryLowerRuntimeSupportMatrix {
    let mut rows: Vec<_> = worth_query_lower_runtime_crossing_inventory()
        .rows()
        .iter()
        .map(|row| {
            WorthQueryLowerRuntimeSupportRow::new(
                row.seam_key(),
                row.capability_label(),
                row.lower_runtime_owner(),
                row.route_kind(),
                row.current_artifact_strength(),
                support_posture_for_classification(row.classification()),
                WorthQueryLowerRuntimeSupportDetail::Crossing,
            )
        })
        .collect();
    rows.extend(
        worth_query_lower_runtime_closeout_registry()
            .rows()
            .iter()
            .map(|row| {
                WorthQueryLowerRuntimeSupportRow::new(
                    row.seam_key(),
                    row.capability_label(),
                    row.owner(),
                    row.route_kind(),
                    WorthQueryLowerRuntimeArtifactStrength::DerivedAggregateArtifact,
                    support_posture_for_closeout(row.posture()),
                    WorthQueryLowerRuntimeSupportDetail::Closeout {
                        closeout_target: row.closeout_target(),
                        required_closeout: row.required_closeout(),
                        certification_row: row.certification_row(),
                    },
                )
            }),
    );
    WorthQueryLowerRuntimeSupportMatrix::new(rows)
}
