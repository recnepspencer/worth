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
pub(crate) fn test_execution_lease(
    cancellation: worth_execution::CancellationToken,
) -> worth_execution::ExecutionResourceLease<'static> {
    use std::{num::NonZeroUsize, sync::OnceLock};
    use worth_execution::{ExecutionAuthority, ExecutionAuthorityConfig, LeaseRequest};
    use worth_foundational::{
        DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
    };

    static AUTHORITY: OnceLock<ExecutionAuthority> = OnceLock::new();
    let authority = AUTHORITY.get_or_init(|| {
        ExecutionAuthority::try_construct(ExecutionAuthorityConfig {
            max_workers: NonZeroUsize::new(2).unwrap(),
            charged_memory_bytes: Some(4096),
        })
        .unwrap()
    });
    authority
        .request_lease(LeaseRequest {
            policy: ExecutionRequestPolicy::new(
                ExecutionPosture::Automatic,
                DeterminismContract::CanonicalBitwise,
                ExecutionBudget::new(NonZeroUsize::new(2).unwrap(), 4096, 10),
            ),
            deadline: None,
            cancellation,
        })
        .unwrap()
}
