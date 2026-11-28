use super::{BridgeErrorKind, *};
use worth_execution::report::RetryHint;
use worth_execution::report::{DenialClass::*, Disposition::*};

impl BridgeErrorKind for BridgeWritebackErrorKind {
    fn report(&self) -> (DenialClass, Disposition, &'static str) {
        match self {
            Self::WritebackNotRequested => (
                InvalidInput,
                CorrectInput,
                "bridge.writeback.writeback_not_requested",
            ),
            Self::PolicyRejected => (
                Authorization,
                Reauthorize,
                "bridge.writeback.policy_rejected",
            ),
            Self::FamilyBindingMismatch => (
                Invariant,
                Escalate,
                "bridge.writeback.family_binding_mismatch",
            ),
            Self::StrategyDescriptorMismatch => (
                Invariant,
                Escalate,
                "bridge.writeback.strategy_descriptor_mismatch",
            ),
            Self::IdempotenceBasisMismatch => (
                Conflict,
                Final,
                "bridge.writeback.idempotence_basis_mismatch",
            ),
            Self::StaleTruthBasis => (
                Conflict,
                Retry(RetryHint::AfterBasisAdvance),
                "bridge.writeback.stale_truth_basis",
            ),
            Self::InvariantRejected => (Invariant, Escalate, "bridge.writeback.invariant_rejected"),
            Self::StrategyFailed => (ExternalEffect, Escalate, "bridge.writeback.strategy_failed"),
            Self::StrategyPanicked => (Defect, Escalate, "bridge.writeback.strategy_panicked"),
            Self::AuthorityDenied => (
                Authorization,
                Reauthorize,
                "bridge.writeback.authority_denied",
            ),
            Self::PreviewWritebackRejected => (
                Authorization,
                Reauthorize,
                "bridge.writeback.preview_writeback_rejected",
            ),
        }
    }

    fn summary(&self) -> &'static str {
        "Bridge could not complete writeback."
    }
}

impl BridgeErrorKind for crate::subscription::BridgeSubscriptionBasisResolutionFailureKind {
    fn report(&self) -> (DenialClass, Disposition, &'static str) {
        match self {
            Self::MissingBranchHeadSource => (
                Unavailable,
                Escalate,
                "bridge.subscription_basis.missing_branch_head_source",
            ),
            Self::BranchHeadResolutionFailure => (
                ExternalEffect,
                Escalate,
                "bridge.subscription_basis.branch_head_resolution_failure",
            ),
            Self::BranchHeadMismatch => (
                Conflict,
                Final,
                "bridge.subscription_basis.branch_head_mismatch",
            ),
            Self::SnapshotAcquisitionFailure => (
                ExternalEffect,
                Escalate,
                "bridge.subscription_basis.snapshot_acquisition_failure",
            ),
            Self::SnapshotIdentityMismatch => (
                Invariant,
                Escalate,
                "bridge.subscription_basis.snapshot_identity_mismatch",
            ),
        }
    }

    fn summary(&self) -> &'static str {
        "Bridge could not resolve the subscription basis."
    }
}
