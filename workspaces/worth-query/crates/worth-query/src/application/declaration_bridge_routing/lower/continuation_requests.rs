use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use worth_runtime_bridge::facade::{
    BridgeDeliveryIntent, BridgeDiagnosticsTier, BridgePreviewSessionDeclaration,
    BridgePreviewSessionDeclarationIdentity, BridgePreviewSessionIdentity, BridgeReplayMode,
    BridgeRequestKind, BridgeRouteRequest, BridgeSignalBranchIdentity,
    BridgeSpeculativeBranchBinding, BridgeSpeculativeBranchBindingIdentity,
    BridgeSpeculativeSessionRequest, BridgeTruthViewEvaluationRequest,
};

use crate::application::WorthQueryDeclarationEnvelope;

use super::super::request::{
    WorthQueryDeclarationBridgeContinuationRequest, WorthQueryDeclarationBridgeTruthContext,
};
use super::super::{query_truth_branch_identity, query_truth_commit_identity};
use super::evidence::bridge_lowering_bridge_evidence_identity;
use super::BridgeLoweringContext;

pub(super) fn runtime_route_request<
    D: crate::application::WorthQueryDomainEntryMarker,
    I: crate::application::WorthQueryDeclarationInput<D>,
>(
    envelope: &WorthQueryDeclarationEnvelope<D, I>,
) -> BridgeRouteRequest {
    BridgeRouteRequest::for_commit(query_truth_commit_identity(
        "query-bridge-route",
        envelope.declaration_digest(),
    ))
}

pub(super) fn truth_view_request<
    D: crate::application::WorthQueryDomainEntryMarker,
    I: crate::application::WorthQueryDeclarationInput<D>,
>(
    lowering: &BridgeLoweringContext,
    envelope: &WorthQueryDeclarationEnvelope<D, I>,
    request: WorthQueryDeclarationBridgeContinuationRequest,
) -> BridgeTruthViewEvaluationRequest {
    let selector = lowering.truth_view_selector(envelope, request.truth_context());
    let evaluation = BridgeTruthViewEvaluationRequest::new(selector)
        .with_diagnostics_tier(BridgeDiagnosticsTier::Standard);
    if request.truth_context() == WorthQueryDeclarationBridgeTruthContext::Historical {
        evaluation
            .with_replay_mode(BridgeReplayMode::Required)
            .with_delivery_intent(BridgeDeliveryIntent::PrepareOnly)
    } else {
        evaluation.with_delivery_intent(BridgeDeliveryIntent::PrepareSignalEvaluation)
    }
}

pub(super) fn preview_session_request<
    D: crate::application::WorthQueryDomainEntryMarker,
    I: crate::application::WorthQueryDeclarationInput<D>,
>(
    envelope: &WorthQueryDeclarationEnvelope<D, I>,
    lowering: &BridgeLoweringContext,
    request: WorthQueryDeclarationBridgeContinuationRequest,
) -> Result<BridgeSpeculativeSessionRequest, IdentityDerivationDenial> {
    let binding = preview_branch_basis(envelope)?;
    let declaration = BridgePreviewSessionDeclaration::new(
        BridgePreviewSessionDeclarationIdentity::from_bridge_evidence(
            &bridge_lowering_bridge_evidence_identity(
                "preview-session-declaration",
                envelope
                    .envelope_digest()
                    .metadata()
                    .algorithm()
                    .id()
                    .as_str(),
            )?,
        ),
        BridgeRequestKind::Preview,
        binding,
        lowering.preview_session_basis(envelope, request.truth_context()),
    );
    Ok(BridgeSpeculativeSessionRequest::new(
        BridgePreviewSessionIdentity::from_bridge_evidence(
            &bridge_lowering_bridge_evidence_identity(
                "preview-session",
                envelope.declaration_digest(),
            )?,
        ),
        declaration,
        1,
        1,
        1,
    ))
}

fn preview_branch_basis<
    D: crate::application::WorthQueryDomainEntryMarker,
    I: crate::application::WorthQueryDeclarationInput<D>,
>(
    envelope: &WorthQueryDeclarationEnvelope<D, I>,
) -> Result<BridgeSpeculativeBranchBinding, IdentityDerivationDenial> {
    Ok(BridgeSpeculativeBranchBinding::new(
        BridgeSpeculativeBranchBindingIdentity::from_bridge_evidence(
            &bridge_lowering_bridge_evidence_identity(
                "preview-branch-binding",
                envelope.declaration_digest(),
            )?,
        ),
        query_truth_branch_identity(
            "query-preview-truth-branch",
            envelope.operating_context_identity_digest(),
        ),
        BridgeSignalBranchIdentity::from_bridge_evidence(
            &bridge_lowering_bridge_evidence_identity(
                "preview-signal-branch",
                envelope.handle_identity_digest(),
            )?,
        ),
    ))
}
