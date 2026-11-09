use super::*;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;

pub(in crate::runtime) fn lower_runtime_support_row_identity(
    row: &WorthQueryLowerRuntimeSupportRow,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    let mut identity =
        worth_query_evidence_identity(WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "lower_runtime_support_row_v1",
            )
            .field_shape(WorthQueryEvidenceTag::new("seam"), row.seam_key().as_str())
            .field_shape(
                WorthQueryEvidenceTag::new("capability"),
                row.capability_label(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("owner"),
                row.authority_owner().as_str(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("route_kind"),
                row.route_kind().as_str(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("artifact"),
                row.artifact_strength().as_str(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("posture"),
                row.posture().as_str(),
            );
    match row.detail() {
        crate::lower_runtime_routing::WorthQueryLowerRuntimeSupportDetail::Crossing => {
            identity = identity.field_shape(WorthQueryEvidenceTag::new("detail"), "crossing");
        }
        crate::lower_runtime_routing::WorthQueryLowerRuntimeSupportDetail::Closeout {
            closeout_target,
            required_closeout,
            certification_row,
        } => {
            identity = identity
                .field_shape(WorthQueryEvidenceTag::new("detail"), "closeout")
                .field_shape(
                    WorthQueryEvidenceTag::new("closeout_target"),
                    closeout_target,
                )
                .field_shape(
                    WorthQueryEvidenceTag::new("required_closeout"),
                    required_closeout,
                )
                .field_shape(
                    WorthQueryEvidenceTag::new("certification_row"),
                    certification_row,
                );
        }
    }
    Ok(identity.seal()?)
}

pub(in crate::runtime) fn lower_runtime_support_rows_aggregate_identity<'a>(
    rows: impl IntoIterator<Item = &'a WorthQueryLowerRuntimeSupportRow>,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    let row_identities = rows
        .into_iter()
        .map(lower_runtime_support_row_identity)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(
        worth_query_evidence_identity(WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "runtime_downstream_durable_resume_support_v1",
            )
            .field_evidence_identity_sequence(WorthQueryEvidenceTag::new("rows"), &row_identities)
            .seal()?,
    )
}

pub(in crate::runtime) fn runtime_downstream_delivery_contract_identity(
    backend_posture: WorthQueryRuntimeBackendPosture,
    runtime_resume_support_status: WorthQueryLowerRuntimeSupportPosture,
    runtime_resume_support_identity: &WorthQueryEvidenceIdentity,
    durable_resume_support_status: WorthQueryLowerRuntimeSupportPosture,
    durable_resume_support_identity: &WorthQueryEvidenceIdentity,
) -> WorthQueryEvidenceIdentity {
    worth_query_evidence_identity(WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "worth_query_runtime_downstream_delivery_contract_v1",
            )
            .field_shape(
                WorthQueryEvidenceTag::new("posture"),
                backend_posture.as_str(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("runtime_resume"),
                runtime_resume_support_status.as_str(),
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("runtime_resume_support"),
                runtime_resume_support_identity,
            )
            .field_shape(
                WorthQueryEvidenceTag::new("durable_resume"),
                durable_resume_support_status.as_str(),
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("durable_resume_support"),
                durable_resume_support_identity,
            )
            .seal().expect("fixed digest grammar: literal tags, primitive values, closed enum labels, or sealed digest tokens")
}
