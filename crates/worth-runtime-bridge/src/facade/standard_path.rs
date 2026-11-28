use super::*;

mod diagnostics;
mod routing;

pub use diagnostics::{BridgeDiagnostics, BridgeStandardDiagnosticsExplanation};
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
