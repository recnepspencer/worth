//! Bridge snapshot contracts and packetized read surfaces.

mod context;
mod declaration;
mod history;
mod materialization;
mod packet;
mod policy;
mod read_contract;
mod read_correlation;
mod read_error;
mod read_result;
mod read_target;
mod selection;
pub(crate) mod token;
pub(crate) mod validated_value_basis;

pub use context::{AdmittedSnapshotContext, BridgeSnapshotContext, TruthSnapshotReader};
pub use declaration::{
    BridgeDeliveryIntent, BridgeReplayMode, BridgeTruthViewKind, BridgeTruthViewSelector,
    BridgeTruthViewSelectorIdentity, HistoricalEvaluationDeclaration,
    HistoricalEvaluationDeclarationIdentity, ValidatedTruthViewSelectorSet,
};
pub use history::{
    LoweredHistoricalEvaluationArtifact, LoweredHistoricalEvaluationArtifactIdentity,
};
pub use materialization::{MaterializedTruthViewObservation, TruthViewObservationReader};
pub(crate) use packet::validate_snapshot_read_result_contract;
pub use packet::{SnapshotReadPacket, SnapshotReadRequest};
pub use policy::{
    BridgeTruthViewPolicyRejection, BridgeTruthViewPolicyResolution, ResolvedTruthViewPolicy,
    TruthViewPolicyRejectionKind, TruthViewReplayContinuity, TruthViewRetentionAdmission,
    TruthViewSourceCapability,
};
pub use read_contract::SnapshotReadContract;
pub use read_correlation::SnapshotReadCorrelationId;
pub use read_error::{BridgeSnapshotReadError, BridgeSnapshotReadErrorKind};
pub(crate) use read_result::contract_validated_scalar_aspect_value;
pub use read_result::{
    SnapshotReadPacketResult, SnapshotReadRecord, SnapshotReadValue,
    ValidatedSnapshotReadPacketResult, ValidatedSnapshotReadRecord,
};
pub use read_target::{SnapshotReadTarget, SnapshotReadTargetIdentity};
pub use selection::{BridgeTruthViewAuthorityBasis, PlannedTruthViewPacket};
pub use token::{BridgeSnapshotToken, TruthSnapshotIdentity};

#[cfg(test)]
mod test_execution;
#[cfg(test)]
pub(crate) use test_execution::{test_execution_lease, test_execution_lease_for_policy};

#[cfg(test)]
pub(crate) fn test_serial_request() -> worth_execution::SerialRequest {
    crate::policy::BridgeExecutionPolicyBaseline::development()
        .serial_request(worth_execution::CancellationToken::new(), None)
}
