use crate::facade::{
    BridgeDeliveryError, BridgeDeliveryErrorKind, BridgeTruthViewEvaluation,
    BridgeTruthViewEvaluationRequest, RuntimeBridge, SnapshotReadPacket, TruthBranchIdentity,
    TruthSnapshotIdentity,
};
use crate::identity::{
    BridgeIdentity, PreviewSessionIdentityTag, SpeculativeSignalBranchIdentityTag,
};
use worth_execution::ExecutionRequest;

mod liveness;
use liveness::BridgePreviewSessionLivenessOwner;
pub use liveness::{BridgePreviewSessionLivenessGuard, BridgePreviewSessionLivenessObserver};

pub type BridgePreviewSessionIdentity = BridgeIdentity<PreviewSessionIdentityTag>;
pub type BridgeSignalBranchIdentity = BridgeIdentity<SpeculativeSignalBranchIdentityTag>;

/// The Relational and Signal branches participating in a preview. The owner
/// of each runtime supplies its branch. Binding them does not fork either.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgePreviewBranchBinding {
    relational: TruthBranchIdentity,
    signal: BridgeSignalBranchIdentity,
}

impl BridgePreviewBranchBinding {
    pub fn new(relational: TruthBranchIdentity, signal: BridgeSignalBranchIdentity) -> Self {
        Self { relational, signal }
    }

    pub fn relational(&self) -> &TruthBranchIdentity {
        &self.relational
    }

    pub fn signal(&self) -> &BridgeSignalBranchIdentity {
        &self.signal
    }
}

/// Owns the lifetime of a preview pinned to one Relational snapshot.
///
/// ```
/// use worth_runtime_bridge::facade::BridgePreviewSessionHandle;
/// fn assert_send_sync<T: Send + Sync>() {}
/// assert_send_sync::<BridgePreviewSessionHandle>();
/// ```
#[derive(Debug)]
pub struct BridgePreviewSessionHandle {
    liveness: BridgePreviewSessionLivenessOwner,
    runtime: RuntimeBridge,
    branches: BridgePreviewBranchBinding,
    snapshot: TruthSnapshotIdentity,
}

/// Evidence that the exact preview session was finished for readmission.
/// Authoritative publication requires admission by the publication owner.
#[derive(Debug)]
pub struct FinishedBridgePreviewSession(BridgePreviewSessionLivenessObserver);

impl FinishedBridgePreviewSession {
    pub fn session_identity(&self) -> &BridgePreviewSessionIdentity {
        self.0.session_identity()
    }

    pub(crate) fn belongs_to(&self, session: &BridgePreviewSessionLivenessObserver) -> bool {
        self.0 == *session
    }
}

impl RuntimeBridge {
    /// Bind an existing branch pair and its selected preview snapshot.
    pub fn begin_preview(
        &self,
        identity: BridgePreviewSessionIdentity,
        branches: BridgePreviewBranchBinding,
        snapshot: TruthSnapshotIdentity,
    ) -> BridgePreviewSessionHandle {
        BridgePreviewSessionHandle {
            liveness: BridgePreviewSessionLivenessOwner::new(identity),
            runtime: self.clone(),
            branches,
            snapshot,
        }
    }
}

impl BridgePreviewSessionHandle {
    pub fn session_identity(&self) -> &BridgePreviewSessionIdentity {
        self.liveness.session_identity()
    }

    pub fn liveness_observer(&self) -> BridgePreviewSessionLivenessObserver {
        self.liveness.observer()
    }

    pub fn branches(&self) -> &BridgePreviewBranchBinding {
        &self.branches
    }

    pub fn snapshot_identity(&self) -> &TruthSnapshotIdentity {
        &self.snapshot
    }

    /// Materialize the current main and the pinned preview with the same read
    /// packet so the caller can compare their Relational values.
    pub fn compare_to_main(
        &self,
        main: TruthBranchIdentity,
        packet: SnapshotReadPacket,
        execution: ExecutionRequest<'_, '_>,
    ) -> Result<(BridgeTruthViewEvaluation, BridgeTruthViewEvaluation), BridgeDeliveryError> {
        crate::execution_contact::admit(execution).map_err(|cause| {
            BridgeDeliveryError::new(
                BridgeDeliveryErrorKind::ExecutionDenied(cause),
                cause.label(),
            )
        })?;
        if let Some(source) = (self.runtime.snapshot_read_source.as_ref() as &dyn std::any::Any)
            .downcast_ref::<crate::facade::RuntimeBridgeRelationalSource>()
            && !source
                .snapshot_matches_branch(&self.snapshot, self.branches.relational())
                .map_err(|error| {
                    BridgeDeliveryError::new(error.delivery_kind(), error.to_string())
                })?
        {
            return Err(BridgeDeliveryError::new(
                BridgeDeliveryErrorKind::HistoricalBranchMismatch,
                "preview snapshot belongs to a different Relational branch",
            ));
        }
        let main = self.runtime.evaluate(
            BridgeTruthViewEvaluationRequest::for_branch_head(main)
                .with_read_packet(packet.clone()),
            execution,
        )?;
        let preview = self.runtime.evaluate(
            BridgeTruthViewEvaluationRequest::for_branch_snapshot(
                self.branches.relational().clone(),
                self.snapshot.clone(),
            )
            .with_read_packet(packet),
            execution,
        )?;
        Ok((main, preview))
    }

    pub fn discard(self) {}

    pub fn finish(self) -> FinishedBridgePreviewSession {
        FinishedBridgePreviewSession(self.liveness.observer())
    }
}

impl BridgePreviewSessionIdentity {
    pub fn from_stable_name(value: impl Into<std::sync::Arc<str>>) -> Self {
        Self::admit_bridge_owned(value)
    }
}

impl BridgeSignalBranchIdentity {
    pub fn from_stable_name(value: impl Into<std::sync::Arc<str>>) -> Self {
        Self::admit_bridge_owned(value)
    }
}
