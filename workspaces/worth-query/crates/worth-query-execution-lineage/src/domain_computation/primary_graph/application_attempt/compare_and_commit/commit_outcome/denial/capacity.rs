//! Capacity, budget, and identity-exhaustion commit denials.

use super::{
    WorthQueryApplicationCommitDenial, WorthQueryApplicationCommitDenialKind,
    WorthQueryApplicationCommitDenialStage,
};

impl WorthQueryApplicationCommitDenial {
    pub const fn active_snapshot_capacity_exhausted(
        stage: WorthQueryApplicationCommitDenialStage,
        maximum_active_snapshots: usize,
    ) -> Self {
        Self {
            kind: WorthQueryApplicationCommitDenialKind::ActiveSnapshotCapacityExhausted {
                maximum_active_snapshots,
            },
            stage,
            detail: None,
            cause: None,
        }
    }

    pub const fn retention_capacity_exhausted(
        stage: WorthQueryApplicationCommitDenialStage,
    ) -> Self {
        Self {
            kind: WorthQueryApplicationCommitDenialKind::RetentionCapacityExhausted,
            stage,
            detail: None,
            cause: None,
        }
    }

    pub const fn snapshot_identity_exhausted(
        stage: WorthQueryApplicationCommitDenialStage,
    ) -> Self {
        Self {
            kind: WorthQueryApplicationCommitDenialKind::SnapshotIdentityExhausted,
            stage,
            detail: None,
            cause: None,
        }
    }

    pub const fn retention_identity_exhausted(
        stage: WorthQueryApplicationCommitDenialStage,
    ) -> Self {
        Self {
            kind: WorthQueryApplicationCommitDenialKind::RetentionIdentityExhausted,
            stage,
            detail: None,
            cause: None,
        }
    }

    pub const fn candidate_identity_exhausted(
        stage: WorthQueryApplicationCommitDenialStage,
    ) -> Self {
        Self {
            kind: WorthQueryApplicationCommitDenialKind::CandidateIdentityExhausted,
            stage,
            detail: None,
            cause: None,
        }
    }

    pub fn index_maintenance_budget_exceeded(
        stage: WorthQueryApplicationCommitDenialStage,
        detail: impl Into<std::sync::Arc<str>>,
    ) -> Self {
        Self {
            kind: WorthQueryApplicationCommitDenialKind::IndexMaintenanceBudgetExceeded,
            stage,
            detail: Some(detail.into()),
            cause: None,
        }
    }

    pub const fn index_generation_identity_exhausted(
        stage: WorthQueryApplicationCommitDenialStage,
    ) -> Self {
        Self {
            kind: WorthQueryApplicationCommitDenialKind::IndexGenerationIdentityExhausted,
            stage,
            detail: None,
            cause: None,
        }
    }
}
