use super::*;

mod diagnostics;
mod routing;
mod speculation;
pub use speculation::BridgeSpeculativeSessionHandle;

pub use diagnostics::{
    BridgeDiagnostics, BridgeStandardDiagnosticsExplanation, BridgeStandardSessionExplanation,
};
pub use routing::{BridgeEvaluationTarget, BridgeRoute, BridgeStandardRouteError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeTruthViewEvaluationRequest {
    selector: BridgeTruthViewSelector,
    read_packet: SnapshotReadPacket,
    replay_mode: BridgeReplayMode,
    diagnostics_tier: BridgeDiagnosticsTier,
    delivery_intent: BridgeDeliveryIntent,
}

impl BridgeTruthViewEvaluationRequest {
    /// Creates a request for the current head of a truth branch.
    pub fn for_branch_head(branch_identity: TruthBranchIdentity) -> Self {
        Self::new(BridgeTruthViewSelector::branch_head(branch_identity))
    }

    /// Creates a request for a specific branch-local snapshot.
    pub fn for_branch_snapshot(
        branch_identity: TruthBranchIdentity,
        snapshot_identity: TruthSnapshotIdentity,
    ) -> Self {
        Self::new(BridgeTruthViewSelector::branch_snapshot(
            branch_identity,
            snapshot_identity,
        ))
    }

    /// Creates a request for a historical commit on a branch.
    pub fn for_historical_commit(
        branch_identity: TruthBranchIdentity,
        commit_identity: TruthCommitIdentity,
    ) -> Self {
        Self::new(BridgeTruthViewSelector::historical_commit(
            branch_identity,
            commit_identity,
        ))
    }

    /// Creates a request from an explicit truth-view selector.
    pub fn new(selector: BridgeTruthViewSelector) -> Self {
        Self {
            selector,
            read_packet: SnapshotReadPacket::new(vec![]),
            replay_mode: BridgeReplayMode::Enabled,
            diagnostics_tier: BridgeDiagnosticsTier::Standard,
            delivery_intent: BridgeDeliveryIntent::PrepareSignalEvaluation,
        }
    }

    /// Overrides the read packet used for truth-view materialization.
    pub fn with_read_packet(mut self, read_packet: SnapshotReadPacket) -> Self {
        self.read_packet = read_packet;
        self
    }

    /// Overrides replay-mode policy for this evaluation.
    pub fn with_replay_mode(mut self, replay_mode: BridgeReplayMode) -> Self {
        self.replay_mode = replay_mode;
        self
    }

    /// Overrides diagnostics-tier policy for this evaluation.
    pub fn with_diagnostics_tier(mut self, diagnostics_tier: BridgeDiagnosticsTier) -> Self {
        self.diagnostics_tier = diagnostics_tier;
        self
    }

    /// Overrides delivery intent for this evaluation.
    pub fn with_delivery_intent(mut self, delivery_intent: BridgeDeliveryIntent) -> Self {
        self.delivery_intent = delivery_intent;
        self
    }

    pub fn selector(&self) -> &BridgeTruthViewSelector {
        &self.selector
    }

    pub(crate) fn declaration(&self) -> HistoricalEvaluationDeclaration {
        HistoricalEvaluationDeclaration::new(
            self.selector.clone(),
            self.replay_mode,
            self.diagnostics_tier,
            self.delivery_intent,
        )
    }

    pub(crate) fn read_packet(&self) -> SnapshotReadPacket {
        self.read_packet.clone()
    }
}

pub struct BridgeTruthViewEvaluation {
    observation: crate::snapshot::MaterializedTruthViewObservation,
    canonical_record: BridgeCanonicalHistoricalEvaluationRecord,
}

impl BridgeTruthViewEvaluation {
    pub(crate) fn new(
        observation: crate::snapshot::MaterializedTruthViewObservation,
        canonical_record: BridgeCanonicalHistoricalEvaluationRecord,
    ) -> Self {
        Self {
            observation,
            canonical_record,
        }
    }

    /// Returns the materialized truth-view observation.
    pub fn observation(&self) -> &crate::snapshot::MaterializedTruthViewObservation {
        &self.observation
    }

    /// Returns the canonical historical evaluation record derived from the
    /// materialized observation.
    pub fn record(&self) -> &BridgeCanonicalHistoricalEvaluationRecord {
        &self.canonical_record
    }

    /// Returns the snapshot identity bound by the observation.
    pub fn snapshot_identity(&self) -> &TruthSnapshotIdentity {
        self.observation.snapshot_identity()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeSpeculativeSessionRequest {
    session_identity: BridgePreviewSessionIdentity,
    declaration: BridgePreviewSessionDeclaration,
    preview_artifact_count: usize,
    destroyable_artifact_count: usize,
    retained_non_authoritative_artifact_count: usize,
}

impl BridgeSpeculativeSessionRequest {
    /// Creates a speculative session request.
    pub fn new(
        session_identity: BridgePreviewSessionIdentity,
        declaration: BridgePreviewSessionDeclaration,
        preview_artifact_count: usize,
        destroyable_artifact_count: usize,
        retained_non_authoritative_artifact_count: usize,
    ) -> Self {
        Self {
            session_identity,
            declaration,
            preview_artifact_count,
            destroyable_artifact_count,
            retained_non_authoritative_artifact_count,
        }
    }

    /// Returns the preview session identity.
    pub fn session_identity(&self) -> &BridgePreviewSessionIdentity {
        &self.session_identity
    }

    /// Returns the preview declaration.
    pub fn declaration(&self) -> &BridgePreviewSessionDeclaration {
        &self.declaration
    }

    pub(crate) fn preview_artifact_count(&self) -> usize {
        self.preview_artifact_count
    }

    pub(crate) fn destroyable_artifact_count(&self) -> usize {
        self.destroyable_artifact_count
    }

    pub(crate) fn retained_non_authoritative_artifact_count(&self) -> usize {
        self.retained_non_authoritative_artifact_count
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeSpeculativeComparison {
    preview_session_identity: BridgePreviewSessionIdentity,
    truth_branch_identity: TruthBranchIdentity,
    signal_branch_identity: BridgeSignalBranchIdentity,
    truth_view_selector: BridgeTruthViewSelector,
    truth_view_basis_digest: std::sync::Arc<str>,
}

impl BridgeSpeculativeComparison {
    pub(crate) fn from_active_session(session: &BridgePreviewSession<PreviewActive>) -> Self {
        let declaration = session.declaration().declaration();
        let binding = declaration.branch_binding();
        Self {
            preview_session_identity: session.session_identity().clone(),
            truth_branch_identity: binding.truth_branch_identity().clone(),
            signal_branch_identity: binding.signal_branch_identity().clone(),
            truth_view_selector: declaration.session_basis().truth_view_selector().clone(),
            truth_view_basis_digest: declaration.truth_view_basis_digest().into(),
        }
    }

    /// Returns the preview session identity being compared.
    pub fn preview_session_identity(&self) -> &BridgePreviewSessionIdentity {
        &self.preview_session_identity
    }

    /// Returns the speculative truth branch identity.
    pub fn truth_branch_identity(&self) -> &TruthBranchIdentity {
        &self.truth_branch_identity
    }

    /// Returns the speculative signal branch identity.
    pub fn signal_branch_identity(&self) -> &BridgeSignalBranchIdentity {
        &self.signal_branch_identity
    }

    /// Returns the shared truth-view basis digest for the comparison.
    pub fn truth_view_basis_digest(&self) -> &str {
        self.truth_view_basis_digest.as_ref()
    }

    /// Builds the truth-view request for the speculative side.
    pub fn speculative_evaluation_request(&self) -> BridgeTruthViewEvaluationRequest {
        BridgeTruthViewEvaluationRequest::new(self.truth_view_selector.clone())
    }

    /// Builds the truth-view request for the main side.
    pub fn main_evaluation_request(
        &self,
        main_branch_identity: TruthBranchIdentity,
    ) -> BridgeTruthViewEvaluationRequest {
        BridgeTruthViewEvaluationRequest::for_branch_head(main_branch_identity)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct BridgeSpeculativeDiscardOutcome {
    session: BridgePreviewSession<PreviewDiscarded>,
    record: BridgePreviewDiscardRecord,
}

impl BridgeSpeculativeDiscardOutcome {
    pub(crate) fn new(
        session: BridgePreviewSession<PreviewDiscarded>,
        record: BridgePreviewDiscardRecord,
    ) -> Self {
        Self { session, record }
    }

    /// Returns the discarded terminal session state.
    pub fn session(&self) -> &BridgePreviewSession<PreviewDiscarded> {
        &self.session
    }

    /// Returns the canonical discard record.
    pub fn record(&self) -> &BridgePreviewDiscardRecord {
        &self.record
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct BridgeSpeculativePromotionOutcome {
    session: BridgePreviewSession<PreviewPromoted>,
    record: BridgePreviewPromotionRecord,
}

impl BridgeSpeculativePromotionOutcome {
    pub(crate) fn new(
        session: BridgePreviewSession<PreviewPromoted>,
        record: BridgePreviewPromotionRecord,
    ) -> Self {
        Self { session, record }
    }

    /// Returns the promoted terminal session state.
    pub fn session(&self) -> &BridgePreviewSession<PreviewPromoted> {
        &self.session
    }

    /// Returns the canonical promotion record.
    pub fn record(&self) -> &BridgePreviewPromotionRecord {
        &self.record
    }
}
