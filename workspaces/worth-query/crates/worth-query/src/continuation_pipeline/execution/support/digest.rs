use super::*;

pub(in crate::continuation_pipeline::execution) fn prepared_digest<
    D: WorthQueryDomainEntryMarker,
    C: WorthQueryDomainOperatingContext<D>,
    I: WorthQueryDeclarationInput<D>,
>(
    handle: &WorthQueryInstalledDomainDeclarationContext<D, C>,
    bridge_request: &WorthQueryDeclarationBridgeContinuationRequest,
    required_contract: &crate::application::WorthQueryDeclarationAspectContract,
    linked: &WorthQueryBindingLinkedArtifacts,
    signal_truth: &ResolvedSignalContinuationTruth,
    bridge_routing: &crate::application::WorthQueryDeclarationBridgeRouting<D, I>,
) -> Result<String, IdentityDerivationDenial> {
    Ok(
        worth_query_evidence_identity(WorthQueryEvidenceScope::ContinuationPreparedDigest)
            .field_value(
                WorthQueryEvidenceTag::new("handle"),
                handle.handle_identity_digest(),
            )
            .field_value(
                WorthQueryEvidenceTag::new("operating_context"),
                handle.operating_context_identity_digest(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("bridge_mode"),
                bridge_request.mode().as_str(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("truth_context"),
                bridge_request.truth_context().as_str(),
            )
            .field_value_sequence(
                WorthQueryEvidenceTag::new("required_aspects"),
                required_contract
                    .required()
                    .iter()
                    .map(terminal_declaration_aspect_projection_for_digest),
            )
            .field_value_sequence(
                WorthQueryEvidenceTag::new("preserved_aspects"),
                required_contract
                    .preserved()
                    .iter()
                    .map(terminal_declaration_aspect_projection_for_digest),
            )
            .field_value_sequence(
                WorthQueryEvidenceTag::new("published_aspects"),
                required_contract
                    .published()
                    .iter()
                    .map(terminal_declaration_aspect_projection_for_digest),
            )
            .field_value_sequence(
                WorthQueryEvidenceTag::new("masked_aspects"),
                required_contract
                    .masked()
                    .iter()
                    .map(terminal_declaration_aspect_projection_for_digest),
            )
            .field_value_sequence(
                WorthQueryEvidenceTag::new("incompatible_aspects"),
                required_contract
                    .incompatible()
                    .iter()
                    .map(terminal_declaration_aspect_projection_for_digest),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("signal_posture"),
                signal_truth.posture.as_str(),
            )
            .optional_shape(
                WorthQueryEvidenceTag::new("signal_execution_family"),
                signal_truth.execution_family.map(|family| family.as_str()),
            )
            .field_value_sequence(
                WorthQueryEvidenceTag::new("signal_basis_families"),
                signal_truth.basis_families.iter().map(BasisFamily::as_str),
            )
            .optional_value(
                WorthQueryEvidenceTag::new("signal_digest"),
                signal_truth.digest.as_deref(),
            )
            .field_value(
                WorthQueryEvidenceTag::new("signal_reason"),
                signal_truth.reason,
            )
            .field_value(
                WorthQueryEvidenceTag::new("future_projection"),
                bridge_routing.future_projection().projection_digest(),
            )
            .field_value(
                WorthQueryEvidenceTag::new("basis_lifecycle_support"),
                bridge_routing.basis_lifecycle_support_digest(),
            )
            .field_value(
                WorthQueryEvidenceTag::new("bridge_routing"),
                bridge_routing.bridge_routing_digest(),
            )
            .field_value(
                WorthQueryEvidenceTag::new("linked_artifacts"),
                linked_artifacts_identity(linked)?,
            )
            .seal()?
            .as_str()
            .to_string(),
    )
}

pub(in crate::continuation_pipeline::execution) fn transcript_digest(
    kind: &str,
    family: &'static str,
    linked: &WorthQueryBindingLinkedArtifacts,
    outcome: &str,
) -> Result<String, IdentityDerivationDenial> {
    Ok(
        worth_query_evidence_identity(WorthQueryEvidenceScope::ContinuationExecutionTranscript)
            .field_shape(WorthQueryEvidenceTag::new("kind"), kind)
            .field_shape(WorthQueryEvidenceTag::new("family"), family)
            .field_shape(WorthQueryEvidenceTag::new("outcome"), outcome)
            .field_value(
                WorthQueryEvidenceTag::new("linked_artifacts"),
                linked_artifacts_identity(linked)?,
            )
            .seal()?
            .as_str()
            .to_string(),
    )
}

pub(in crate::continuation_pipeline::execution) fn linked_artifacts_identity(
    linked: &WorthQueryBindingLinkedArtifacts,
) -> Result<String, IdentityDerivationDenial> {
    Ok(
        worth_query_evidence_identity(WorthQueryEvidenceScope::ContinuationLinkedArtifacts)
            .optional_value(
                WorthQueryEvidenceTag::new("declaration"),
                linked.declaration_digest(),
            )
            .optional_value(
                WorthQueryEvidenceTag::new("progression"),
                linked.progression_digest(),
            )
            .optional_value(
                WorthQueryEvidenceTag::new("route_plan"),
                linked.route_plan_digest(),
            )
            .optional_value(
                WorthQueryEvidenceTag::new("receipt"),
                linked.receipt_digest(),
            )
            .optional_value(
                WorthQueryEvidenceTag::new("envelope"),
                linked.envelope_digest(),
            )
            .optional_value(
                WorthQueryEvidenceTag::new("orchestration"),
                linked.orchestration_digest(),
            )
            .seal()?
            .as_str()
            .to_string(),
    )
}

pub(super) fn canonical_derived_digest_identity(
    role: &'static str,
    digest: &worth_foundational::facade::CanonicalDerivedDigest,
) -> Result<String, IdentityDerivationDenial> {
    Ok(
        worth_query_evidence_identity(WorthQueryEvidenceScope::ContinuationLinkedArtifacts)
            .field_shape(WorthQueryEvidenceTag::new("role"), role)
            .field_shape(
                WorthQueryEvidenceTag::new("algorithm"),
                digest.metadata().algorithm().id().as_str(),
            )
            .field_value_sequence(
                WorthQueryEvidenceTag::new("bytes"),
                digest.value().bytes().iter().map(|byte| hex_byte(*byte)),
            )
            .seal()?
            .as_str()
            .to_string(),
    )
}
