use crate::domain_computation::primary_graph::provider::session_commit::provider_failure;
use crate::domain_computation::{
    WorthQueryProviderSessionDenialKind, WorthQueryProviderSessionFailure,
    WorthQueryProviderSessionProtocolCounters, WorthQueryProviderSessionProtocolStage,
    WorthQueryProviderSessionRecoveryPosture,
};

pub(super) fn failure(detail: &'static str) -> WorthQueryProviderSessionFailure {
    provider_failure(WorthQueryProviderSessionProtocolStage::Commit, detail)
        .with_recovery_posture(WorthQueryProviderSessionRecoveryPosture::RecoveryRequired)
}

pub(super) fn snapshot_capacity_failure(detail: &'static str) -> WorthQueryProviderSessionFailure {
    WorthQueryProviderSessionFailure::new(
        WorthQueryProviderSessionDenialKind::ActiveSnapshotCapacityExhausted {
            maximum_active_snapshots: 0,
        },
        WorthQueryProviderSessionProtocolStage::Commit,
        detail,
        WorthQueryProviderSessionProtocolCounters::default(),
    )
    .with_recovery_posture(WorthQueryProviderSessionRecoveryPosture::RecoveryRequired)
}

pub(super) fn basis_retention_failure(
    denial: worth_relational::facade::branch::RelationalBranchBasisDenial,
) -> WorthQueryProviderSessionFailure {
    let kind = match denial {
        worth_relational::facade::branch::RelationalBranchBasisDenial::RetentionCapacityExhausted => {
            WorthQueryProviderSessionDenialKind::RetentionCapacityExhausted
        }
        worth_relational::facade::branch::RelationalBranchBasisDenial::RetentionIdentityExhausted => {
            WorthQueryProviderSessionDenialKind::RetentionIdentityExhausted
        }
        worth_relational::facade::branch::RelationalBranchBasisDenial::SnapshotIdentityExhausted => {
            WorthQueryProviderSessionDenialKind::SnapshotIdentityExhausted
        }
        _ => WorthQueryProviderSessionDenialKind::ProviderRejected,
    };
    WorthQueryProviderSessionFailure::new(
        kind,
        WorthQueryProviderSessionProtocolStage::Commit,
        format!("application commit basis could not be retained: {denial:?}"),
        WorthQueryProviderSessionProtocolCounters::default(),
    )
    .with_recovery_posture(WorthQueryProviderSessionRecoveryPosture::RecoveryRequired)
}

pub(super) fn bridge_head_failure(
    denial: worth_relational::facade::branch::RelationalBranchBasisDenial,
) -> WorthQueryProviderSessionFailure {
    let kind = match denial {
        worth_relational::facade::branch::RelationalBranchBasisDenial::RetentionCapacityExhausted => {
            WorthQueryProviderSessionDenialKind::RetentionCapacityExhausted
        }
        worth_relational::facade::branch::RelationalBranchBasisDenial::RetentionIdentityExhausted => {
            WorthQueryProviderSessionDenialKind::RetentionIdentityExhausted
        }
        worth_relational::facade::branch::RelationalBranchBasisDenial::SnapshotIdentityExhausted => {
            WorthQueryProviderSessionDenialKind::SnapshotIdentityExhausted
        }
        _ => WorthQueryProviderSessionDenialKind::ProviderRejected,
    };
    WorthQueryProviderSessionFailure::new(
        kind,
        WorthQueryProviderSessionProtocolStage::Commit,
        format!("application commit succeeded but Bridge head binding failed: {denial:?}"),
        WorthQueryProviderSessionProtocolCounters::default(),
    )
    .with_recovery_posture(WorthQueryProviderSessionRecoveryPosture::RecoveryRequired)
}
