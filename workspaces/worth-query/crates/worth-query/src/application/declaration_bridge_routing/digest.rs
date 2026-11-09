use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use worth_foundational::facade::CanonicalDerivedDigest;

use crate::application::{
    WorthQueryDeclarationAspectContract, WorthQueryDeclarationAspectCoverage,
    WorthQueryDeclarationAspectCoverageBasis, WorthQueryDeclarationAspectFit,
    WorthQueryDeclarationEnvelope, WorthQueryDeclarationFutureProjection,
    WorthQueryDeclarationReceiptDenialCause, WorthQueryDeclarationRoutePlanDenialCause,
};
use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};

use super::{
    artifact::WorthQueryDeclarationBridgeRoutingClass,
    contract::WorthQueryDeclarationBridgeContinuationFamily,
    request::WorthQueryDeclarationBridgeContinuationRequest,
};
use crate::application::declaration_aspect::terminal_declaration_aspect_projection;

pub(crate) fn derive_bridge_routing_digest<
    D: crate::application::WorthQueryDomainEntryMarker,
    I: crate::application::WorthQueryDeclarationInput<D>,
>(
    envelope: &WorthQueryDeclarationEnvelope<D, I>,
    class: WorthQueryDeclarationBridgeRoutingClass,
    continuation_request: WorthQueryDeclarationBridgeContinuationRequest,
    continuation_family: WorthQueryDeclarationBridgeContinuationFamily,
    binding_surface: &'static str,
    aspect_contract: &WorthQueryDeclarationAspectContract,
    aspect_coverage: &WorthQueryDeclarationAspectCoverage,
    aspect_coverage_basis: WorthQueryDeclarationAspectCoverageBasis,
    aspect_fit: WorthQueryDeclarationAspectFit,
    mapped_aspects: &WorthQueryDeclarationAspectCoverage,
    mapping_fit: WorthQueryDeclarationAspectFit,
    route_cause: Option<WorthQueryDeclarationRoutePlanDenialCause>,
    receipt_cause: Option<WorthQueryDeclarationReceiptDenialCause>,
    future_projection: &WorthQueryDeclarationFutureProjection,
    basis_lifecycle_support_digest: &str,
) -> Result<String, IdentityDerivationDenial> {
    let receipt_digest = canonical_digest_token(envelope.receipt_digest());
    let envelope_digest = canonical_digest_token(envelope.envelope_digest());

    Ok(
        worth_query_evidence_identity(WorthQueryEvidenceScope::DeclarationBridgeRoutingDigest)
            .field_value(
                WorthQueryEvidenceTag::new("handle"),
                envelope.handle_identity_digest(),
            )
            .field_value(
                WorthQueryEvidenceTag::new("operating_context"),
                envelope.operating_context_identity_digest(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("family"),
                envelope.declaration_family_key(),
            )
            .field_value(
                WorthQueryEvidenceTag::new("declaration"),
                envelope.declaration_digest(),
            )
            .optional_value(
                WorthQueryEvidenceTag::new("progression"),
                envelope.progression_digest(),
            )
            .optional_value(
                WorthQueryEvidenceTag::new("route_plan"),
                envelope.route_plan_digest(),
            )
            .field_value(WorthQueryEvidenceTag::new("receipt"), &receipt_digest)
            .field_value(WorthQueryEvidenceTag::new("envelope"), &envelope_digest)
            .field_shape(WorthQueryEvidenceTag::new("class"), class.as_str())
            .field_shape(
                WorthQueryEvidenceTag::new("mode"),
                continuation_request.mode().as_str(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("truth_context"),
                continuation_request.truth_context().as_str(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("continuation_family"),
                continuation_family.as_str(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("binding_surface"),
                binding_surface,
            )
            .field_value_sequence(
                WorthQueryEvidenceTag::new("aspect_contract_required"),
                aspect_contract
                    .required()
                    .iter()
                    .map(terminal_declaration_aspect_projection),
            )
            .field_value_sequence(
                WorthQueryEvidenceTag::new("aspect_contract_preserved"),
                aspect_contract
                    .preserved()
                    .iter()
                    .map(terminal_declaration_aspect_projection),
            )
            .field_value_sequence(
                WorthQueryEvidenceTag::new("aspect_contract_published"),
                aspect_contract
                    .published()
                    .iter()
                    .map(terminal_declaration_aspect_projection),
            )
            .field_value_sequence(
                WorthQueryEvidenceTag::new("aspect_contract_masked"),
                aspect_contract
                    .masked()
                    .iter()
                    .map(terminal_declaration_aspect_projection),
            )
            .field_value_sequence(
                WorthQueryEvidenceTag::new("aspect_contract_incompatible"),
                aspect_contract
                    .incompatible()
                    .iter()
                    .map(terminal_declaration_aspect_projection),
            )
            .field_value_sequence(
                WorthQueryEvidenceTag::new("aspect_coverage_present"),
                aspect_coverage
                    .present()
                    .iter()
                    .map(terminal_declaration_aspect_projection),
            )
            .field_value_sequence(
                WorthQueryEvidenceTag::new("aspect_coverage_masked"),
                aspect_coverage
                    .masked()
                    .iter()
                    .map(terminal_declaration_aspect_projection),
            )
            .field_value_sequence(
                WorthQueryEvidenceTag::new("aspect_coverage_conflicting"),
                aspect_coverage
                    .conflicting()
                    .iter()
                    .map(terminal_declaration_aspect_projection),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("aspect_coverage_basis"),
                aspect_coverage_basis.as_str(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("aspect_fit"),
                aspect_fit.as_str(),
            )
            .field_value_sequence(
                WorthQueryEvidenceTag::new("mapped_aspect_present"),
                mapped_aspects
                    .present()
                    .iter()
                    .map(terminal_declaration_aspect_projection),
            )
            .field_value_sequence(
                WorthQueryEvidenceTag::new("mapped_aspect_masked"),
                mapped_aspects
                    .masked()
                    .iter()
                    .map(terminal_declaration_aspect_projection),
            )
            .field_value_sequence(
                WorthQueryEvidenceTag::new("mapped_aspect_conflicting"),
                mapped_aspects
                    .conflicting()
                    .iter()
                    .map(terminal_declaration_aspect_projection),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("mapping_fit"),
                mapping_fit.as_str(),
            )
            .field_value(
                WorthQueryEvidenceTag::new("future_projection"),
                future_projection.projection_digest(),
            )
            .field_value(
                WorthQueryEvidenceTag::new("basis_lifecycle_support"),
                basis_lifecycle_support_digest,
            )
            .field_value(
                WorthQueryEvidenceTag::new("evidence_origin"),
                envelope.evidence_origin().as_str(),
            )
            .optional_shape(
                WorthQueryEvidenceTag::new("route_cause"),
                route_cause.map(WorthQueryDeclarationRoutePlanDenialCause::as_str),
            )
            .optional_shape(
                WorthQueryEvidenceTag::new("receipt_cause"),
                receipt_cause.map(WorthQueryDeclarationReceiptDenialCause::as_str),
            )
            .seal()?
            .as_str()
            .to_string(),
    )
}

fn canonical_digest_token(digest: &CanonicalDerivedDigest) -> String {
    let hex = digest
        .value()
        .bytes()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    format!("{}:{hex}", digest.metadata().algorithm().id().as_str())
}
