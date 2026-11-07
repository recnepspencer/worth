use crate::basis_lifecycle::BasisFamily;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;

use super::super::support_matrix_rows::support_rows;
use super::super::taxonomy::EffectFamily;
use super::row::EffectLifecycleSupportRow;
use super::{EffectSupportCause, EffectSupportDecision, EffectSupportPosture};

pub(crate) fn support_decision_for(
    basis_family: BasisFamily,
    effect_family: EffectFamily,
) -> Result<EffectSupportDecision, IdentityDerivationDenial> {
    let mut rows_consulted = 0;
    for row in support_rows() {
        rows_consulted += 1;
        if row.basis_family == basis_family && row.effect_family == effect_family {
            return Ok(EffectSupportDecision {
                posture: row.posture,
                matched_row: Some(EffectLifecycleSupportRow::new(
                    row.basis_family,
                    row.effect_family,
                    row.authority_owner,
                    row.lowered_artifact_kind,
                    row.receipt_artifact_kind,
                    row.posture,
                    row.cause,
                )?),
                cause: row.cause,
                rows_consulted,
            });
        }
    }

    Ok(EffectSupportDecision {
        posture: EffectSupportPosture::Unsupported,
        cause: EffectSupportCause::UnsupportedForBasisFamily,
        matched_row: None,
        rows_consulted,
    })
}
