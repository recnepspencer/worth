use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use worth_foundational::facade::AspectKey;
use worth_runtime_bridge::facade::{
    BridgeRequestKind, BridgeRouteIdentity, BridgeSubscriptionContinuationCandidateInput,
    BridgeWritebackCausalityIdentity, BridgeWritebackDeclaration,
    BridgeWritebackDeclarationIdentity, BridgeWritebackEffectClass, BridgeWritebackEffectIntent,
    BridgeWritebackFamilyKind, BridgeWritebackIdempotenceClass,
    BridgeWritebackNativeCausalityInputs, BridgeWritebackStrategyClass,
};

use crate::application::WorthQueryDeclarationEnvelope;

use super::super::artifact::{
    WorthQueryPreviewPromotionContinuationBinding, WorthQueryWritebackPreparationBinding,
};
use super::super::request::WorthQueryDeclarationBridgeContinuationRequest;
use super::super::{query_truth_commit_identity, query_truth_snapshot_identity};
use super::evidence::bridge_lowering_bridge_evidence_identity;
use super::BridgeLoweringContext;

pub(super) fn preview_promotion_binding<
    D: crate::application::WorthQueryDomainEntryMarker,
    I: crate::application::WorthQueryDeclarationInput<D>,
>(
    envelope: &WorthQueryDeclarationEnvelope<D, I>,
    lowering: &BridgeLoweringContext,
    request: WorthQueryDeclarationBridgeContinuationRequest,
) -> Result<WorthQueryPreviewPromotionContinuationBinding, IdentityDerivationDenial> {
    let preview_basis_digest = lowering.truth_view_basis_digest(envelope, request.truth_context());
    let promotion_continuation_digest = crate::evidence_identity::worth_query_evidence_identity(
        crate::evidence_identity::WorthQueryEvidenceScope::PreviewPromotionContinuation,
    )
    .field_value(
        crate::evidence_identity::WorthQueryEvidenceTag::new("preview_basis"),
        &preview_basis_digest,
    )
    .field_value(
        crate::evidence_identity::WorthQueryEvidenceTag::new("declaration"),
        envelope.declaration_digest(),
    )
    .field_shape(
        crate::evidence_identity::WorthQueryEvidenceTag::new("basis_algorithm"),
        envelope
            .envelope_digest()
            .metadata()
            .algorithm()
            .id()
            .as_str(),
    )
    .seal()?
    .as_str()
    .to_string();
    Ok(WorthQueryPreviewPromotionContinuationBinding::new(
        preview_basis_digest,
        promotion_continuation_digest,
        envelope.declaration_digest().to_string(),
    ))
}

pub(super) fn subscription_preparation_request<
    D: crate::application::WorthQueryDomainEntryMarker,
    I: crate::application::WorthQueryDeclarationInput<D>,
>(
    envelope: &WorthQueryDeclarationEnvelope<D, I>,
    lowering: &BridgeLoweringContext,
    request: WorthQueryDeclarationBridgeContinuationRequest,
) -> Result<BridgeSubscriptionContinuationCandidateInput, IdentityDerivationDenial> {
    let authority = bridge_lowering_bridge_evidence_identity(
        "subscription-authority",
        lowering.truth_view_basis_digest(envelope, request.truth_context()),
    )?;
    let locality = bridge_lowering_bridge_evidence_identity(
        "subscription-locality",
        envelope.declaration_family_key(),
    )?;
    let child_basis = bridge_lowering_bridge_evidence_identity(
        "subscription-child-basis",
        envelope.declaration_digest(),
    )?;
    Ok(
        BridgeSubscriptionContinuationCandidateInput::branch_local_continue_from_evidence(
            &authority,
            &locality,
            &child_basis,
        ),
    )
}

pub(super) fn writeback_preparation_request<
    D: crate::application::WorthQueryDomainEntryMarker,
    I: crate::application::WorthQueryDeclarationInput<D>,
>(
    envelope: &WorthQueryDeclarationEnvelope<D, I>,
    lowering: &BridgeLoweringContext,
    request: WorthQueryDeclarationBridgeContinuationRequest,
) -> Result<WorthQueryWritebackPreparationBinding, IdentityDerivationDenial> {
    let basis_digest = lowering.truth_view_basis_digest(envelope, request.truth_context());
    let declaration = BridgeWritebackDeclaration::writeback_capable(
        BridgeWritebackDeclarationIdentity::from_bridge_evidence(
            &bridge_lowering_bridge_evidence_identity(
                "writeback-declaration",
                envelope.declaration_digest(),
            )?,
        ),
        BridgeRequestKind::Authoritative,
        BridgeWritebackFamilyKind::ProjectedStateDiff,
        BridgeWritebackEffectClass::ProjectedStateDiff,
        BridgeWritebackStrategyClass::ProjectedStateDiffReconciliation,
        BridgeWritebackIdempotenceClass::RequireSemanticNoopSuppression,
    );
    let causality = BridgeWritebackNativeCausalityInputs::new(
        BridgeWritebackCausalityIdentity::from_bridge_evidence(
            &bridge_lowering_bridge_evidence_identity("writeback-causality", &basis_digest)?,
        ),
        query_truth_commit_identity("truth-trigger", envelope.handle_identity_digest()),
        BridgeRouteIdentity::from_bridge_evidence(
            &lowering
                .runtime_surface_identity()
                .bridge_external_identity_evidence(),
        ),
        query_truth_snapshot_identity("evaluation", envelope.declaration_digest()),
        query_truth_snapshot_identity("truth-view-basis", basis_digest),
    );
    let effect_intent = BridgeWritebackEffectIntent::validated_scalar_patch(
        BridgeWritebackEffectClass::ProjectedStateDiff,
        AspectKey::new("query.writeback.preparation")
            .expect("static query writeback preparation aspect key is valid"),
        crate::runtime::WorthQueryAuthoredAspectMutation::native_string_value(
            envelope.declaration_digest().to_string(),
        ),
    )
    .expect("query writeback preparation effect intent should validate");

    Ok(WorthQueryWritebackPreparationBinding::new(
        declaration,
        causality,
        effect_intent,
    ))
}
