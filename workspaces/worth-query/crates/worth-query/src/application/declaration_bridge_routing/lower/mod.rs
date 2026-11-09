use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
mod continuation_requests;
mod evidence;
mod preparation_bindings;

use continuation_requests::{preview_session_request, runtime_route_request, truth_view_request};
use evidence::runtime_surface_identity;
use preparation_bindings::{
    preview_promotion_binding, subscription_preparation_request, writeback_preparation_request,
};

use worth_runtime_bridge::facade::{
    BridgeDeliveryIntent, BridgeDiagnosticsTier, BridgePreviewRetainedArtifactSchema,
    BridgePreviewSessionBasis, BridgeReplayMode, BridgeSourceCapability, BridgeSourceCapabilitySet,
    BridgeTruthViewSelector, HistoricalEvaluationDeclaration,
};

use crate::application::WorthQueryDeclarationEnvelope;
use crate::evidence_identity::WorthQueryEvidenceIdentity;

use super::{
    query_truth_branch_identity, query_truth_commit_identity, query_truth_snapshot_identity,
};

use super::{
    artifact::WorthQueryDeclarationBridgeBinding,
    contract::{
        WorthQueryDeclarationBridgeContinuationContract,
        WorthQueryDeclarationBridgeContinuationFamily,
    },
    request::{
        WorthQueryDeclarationBridgeContinuationMode,
        WorthQueryDeclarationBridgeContinuationRequest, WorthQueryDeclarationBridgeTruthContext,
    },
};

pub(crate) fn worth_query_lower_bridge_binding<
    D: crate::application::WorthQueryDomainEntryMarker,
    I: crate::application::WorthQueryDeclarationInput<D>,
>(
    envelope: &WorthQueryDeclarationEnvelope<D, I>,
    contract: WorthQueryDeclarationBridgeContinuationContract,
) -> Result<
    (
        WorthQueryDeclarationBridgeContinuationRequest,
        WorthQueryDeclarationBridgeContinuationFamily,
        WorthQueryDeclarationBridgeBinding,
    ),
    IdentityDerivationDenial,
> {
    let request = contract.request();
    let lowering = BridgeLoweringContext::new(envelope)?;
    let binding = match request.mode() {
        WorthQueryDeclarationBridgeContinuationMode::RuntimeRoute => {
            WorthQueryDeclarationBridgeBinding::RuntimeRoute(runtime_route_request(envelope))
        }
        WorthQueryDeclarationBridgeContinuationMode::TruthView => {
            WorthQueryDeclarationBridgeBinding::TruthView(truth_view_request(
                &lowering, envelope, request,
            ))
        }
        WorthQueryDeclarationBridgeContinuationMode::PreviewSession => {
            WorthQueryDeclarationBridgeBinding::PreviewSession(preview_session_request(
                envelope, &lowering, request,
            )?)
        }
        WorthQueryDeclarationBridgeContinuationMode::PreviewPromotion => {
            WorthQueryDeclarationBridgeBinding::PreviewPromotion(preview_promotion_binding(
                envelope, &lowering, request,
            )?)
        }
        WorthQueryDeclarationBridgeContinuationMode::SubscriptionPreparation => {
            WorthQueryDeclarationBridgeBinding::SubscriptionPreparation(
                subscription_preparation_request(envelope, &lowering, request)?,
            )
        }
        WorthQueryDeclarationBridgeContinuationMode::WritebackPreparation => {
            WorthQueryDeclarationBridgeBinding::WritebackPreparation(writeback_preparation_request(
                envelope, &lowering, request,
            )?)
        }
    };
    Ok((request, contract.family(), binding))
}

struct BridgeLoweringContext {
    runtime_surface_identity: WorthQueryEvidenceIdentity,
}

impl BridgeLoweringContext {
    fn new<
        D: crate::application::WorthQueryDomainEntryMarker,
        I: crate::application::WorthQueryDeclarationInput<D>,
    >(
        envelope: &WorthQueryDeclarationEnvelope<D, I>,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self {
            runtime_surface_identity: runtime_surface_identity(envelope)?,
        })
    }

    fn truth_view_selector<
        D: crate::application::WorthQueryDomainEntryMarker,
        I: crate::application::WorthQueryDeclarationInput<D>,
    >(
        &self,
        envelope: &WorthQueryDeclarationEnvelope<D, I>,
        truth_context: WorthQueryDeclarationBridgeTruthContext,
    ) -> BridgeTruthViewSelector {
        let branch_identity = query_truth_branch_identity(
            "query-branch",
            envelope.operating_context_identity_digest(),
        );
        match truth_context {
            WorthQueryDeclarationBridgeTruthContext::Current => {
                BridgeTruthViewSelector::branch_head(branch_identity)
            }
            WorthQueryDeclarationBridgeTruthContext::Historical => {
                BridgeTruthViewSelector::historical_commit(
                    branch_identity,
                    query_truth_commit_identity("query-commit", envelope.declaration_digest()),
                )
            }
            WorthQueryDeclarationBridgeTruthContext::Preview => {
                BridgeTruthViewSelector::branch_snapshot(
                    branch_identity,
                    query_truth_snapshot_identity(
                        "query-snapshot",
                        envelope.route_plan_digest().unwrap_or("none"),
                    ),
                )
            }
        }
    }

    fn truth_view_basis_digest<
        D: crate::application::WorthQueryDomainEntryMarker,
        I: crate::application::WorthQueryDeclarationInput<D>,
    >(
        &self,
        envelope: &WorthQueryDeclarationEnvelope<D, I>,
        truth_context: WorthQueryDeclarationBridgeTruthContext,
    ) -> String {
        let declaration = HistoricalEvaluationDeclaration::new(
            self.truth_view_selector(envelope, truth_context),
            BridgeReplayMode::Enabled,
            BridgeDiagnosticsTier::Standard,
            BridgeDeliveryIntent::PrepareSignalEvaluation,
        );
        declaration.digest().to_string()
    }

    fn preview_session_basis<
        D: crate::application::WorthQueryDomainEntryMarker,
        I: crate::application::WorthQueryDeclarationInput<D>,
    >(
        &self,
        envelope: &WorthQueryDeclarationEnvelope<D, I>,
        truth_context: WorthQueryDeclarationBridgeTruthContext,
    ) -> BridgePreviewSessionBasis {
        BridgePreviewSessionBasis::new(
            self.truth_view_selector(envelope, truth_context),
            BridgeSourceCapabilitySet::new(vec![BridgeSourceCapability::SnapshotRead]),
            BridgePreviewRetainedArtifactSchema::PreviewLifecycleArtifactsV1,
        )
    }

    fn runtime_surface_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.runtime_surface_identity
    }
}
