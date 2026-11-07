use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};
use crate::lower_runtime_routing::{
    WorthQueryLowerRuntimeBoundaryEnvelope, WorthQueryLowerRuntimeBoundaryExecutionReceipt,
    WorthQueryLowerRuntimeCapabilityRequest, WorthQueryLowerRuntimeCrossingRow,
    WorthQueryLowerRuntimeReadmissionReceipt, WorthQueryLowerRuntimeRouteKind,
    WorthQueryLowerRuntimeRoutePlan,
};

use super::{
    admitted_fixture_eligibility, fixture_retained_evidence_identity,
    fixture_route_subject_identity, fixture_subject_identity, RepresentativeArtifacts,
    WorthQueryLowerRuntimeRepresentativeEvidenceSource,
};

pub(crate) fn synthetic_inventory_row(
    row: &WorthQueryLowerRuntimeCrossingRow,
) -> Result<RepresentativeArtifacts, IdentityDerivationDenial> {
    let request = WorthQueryLowerRuntimeCapabilityRequest::new(
        row.seam_key(),
        row.route_kind(),
        row.lower_runtime_owner(),
        row.capability_label(),
        fixture_subject_identity(
            "synthetic-inventory-route-subject",
            format!("{}-subject", row.seam_key().as_str()),
        )?,
    )?;
    let eligibility = admitted_fixture_eligibility(
        request.clone(),
        "synthetic-inventory-eligibility",
        format!("{}-eligibility-detail", row.seam_key().as_str()),
    )?;
    Ok(match row.route_kind() {
        WorthQueryLowerRuntimeRouteKind::RoutePlanning => {
            let plan = WorthQueryLowerRuntimeRoutePlan::new(
                eligibility.clone(),
                fixture_route_subject_identity(
                    "synthetic-inventory-route",
                    format!("{}-route", row.seam_key().as_str()),
                )?,
            );
            let retained_evidence =
                crate::lower_runtime_routing::worth_query_lower_runtime_retained_evidence_identity(
                    "synthetic-inventory-route-plan",
                    &fixture_retained_evidence_identity(
                        "synthetic-inventory-route-plan",
                        format!("{}-evidence", row.seam_key().as_str()),
                    )?,
                )?;
            let boundary_receipt = WorthQueryLowerRuntimeBoundaryExecutionReceipt::from_route_plan(
                &plan,
                &retained_evidence,
            )?;
            let envelope = WorthQueryLowerRuntimeBoundaryEnvelope::from_route_plan(
                row.seam_key(),
                &plan,
                &boundary_receipt,
                &retained_evidence,
            )?;
            RepresentativeArtifacts {
                seam_key: row.seam_key(),
                request,
                eligibility,
                route_plan: Some(plan),
                boundary_receipt,
                envelope,
                evidence_source:
                    WorthQueryLowerRuntimeRepresentativeEvidenceSource::InventorySynthesized,
            }
        }
        WorthQueryLowerRuntimeRouteKind::ReadmissionHandoff => {
            let retained_evidence =
                crate::lower_runtime_routing::worth_query_lower_runtime_retained_evidence_identity(
                    "synthetic-inventory-readmission",
                    &fixture_retained_evidence_identity(
                        "synthetic-inventory-readmission",
                        format!("{}-evidence", row.seam_key().as_str()),
                    )?,
                )?;
            let handoff = WorthQueryLowerRuntimeReadmissionReceipt::new(
                eligibility.clone(),
                &retained_evidence,
            );
            let boundary_receipt =
                WorthQueryLowerRuntimeBoundaryExecutionReceipt::from_readmission_receipt(&handoff)?;
            let envelope = WorthQueryLowerRuntimeBoundaryEnvelope::from_readmission_receipt(
                row.seam_key(),
                &handoff,
                &boundary_receipt,
            )?;
            RepresentativeArtifacts {
                seam_key: row.seam_key(),
                request,
                eligibility,
                route_plan: None,
                boundary_receipt,
                envelope,
                evidence_source:
                    WorthQueryLowerRuntimeRepresentativeEvidenceSource::InventorySynthesized,
            }
        }
    })
}

pub(crate) fn normalized_parity_digest(
    label: &str,
    envelopes: &[WorthQueryLowerRuntimeBoundaryEnvelope],
) -> Result<String, IdentityDerivationDenial> {
    let selected = match label {
        "compose-read" => envelopes
            .iter()
            .filter(|row| {
                matches!(
                    row.seam_key().as_str(),
                    "compose-read" | "execute-read-family"
                )
            })
            .collect::<Vec<_>>(),
        _ => envelopes
            .iter()
            .filter(|row| {
                matches!(
                    row.seam_key().as_str(),
                    "basis-readmission-from-truth-view-evidence"
                        | "basis-readmission-from-subscription-evidence"
                )
            })
            .collect::<Vec<_>>(),
    };
    let row_identities = selected
        .iter()
        .map(|envelope| {
            WorthQueryEvidenceIdentity::compose(
                WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence,
            )
            .field_shape(
                WorthQueryEvidenceTag::new("owner"),
                envelope.authority_owner().as_str(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("route_kind"),
                envelope.route_kind().as_str(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("support"),
                envelope.support_posture().as_str(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("cost"),
                envelope.route_cost_posture().as_str(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("failure"),
                envelope.route_failure_topology().as_str(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("strength"),
                envelope.artifact_strength().as_str(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("classification"),
                envelope.crossing_classification().as_str(),
            )
            .seal()
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence)
            .field_shape(WorthQueryEvidenceTag::new("parity_label"), label)
            .field_evidence_identity_sequence(WorthQueryEvidenceTag::new("rows"), &row_identities)
            .seal()?
            .as_str()
            .to_string(),
    )
}

pub(crate) fn hostile_parity_divergence_digest(
    envelopes: &[WorthQueryLowerRuntimeBoundaryEnvelope],
) -> Result<String, IdentityDerivationDenial> {
    let readmission = envelopes
        .iter()
        .find(|row| row.seam_key().as_str() == "live-view-schema-admission")
        .expect("live-view schema admission seam should be present");
    let routing = envelopes
        .iter()
        .find(|row| row.seam_key().as_str() == "signal-invalidation-routing")
        .expect("signal invalidation routing seam should be present");

    Ok(
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence)
            .field_shape(
                WorthQueryEvidenceTag::new("parity_label"),
                "hostile-route-divergence",
            )
            .field_shape(
                WorthQueryEvidenceTag::new("readmission_owner"),
                readmission.authority_owner().as_str(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("readmission_route_kind"),
                readmission.route_kind().as_str(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("readmission_strength"),
                readmission.artifact_strength().as_str(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("readmission_failure"),
                readmission.route_failure_topology().as_str(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("routing_owner"),
                routing.authority_owner().as_str(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("routing_route_kind"),
                routing.route_kind().as_str(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("routing_strength"),
                routing.artifact_strength().as_str(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("routing_failure"),
                routing.route_failure_topology().as_str(),
            )
            .seal()?
            .as_str()
            .to_string(),
    )
}
