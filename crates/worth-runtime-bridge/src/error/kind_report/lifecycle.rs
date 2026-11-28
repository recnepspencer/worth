use super::{BridgeErrorKind, *};
use worth_execution::report::RetryHint;
use worth_execution::report::{DenialClass::*, Disposition::*};

impl BridgeErrorKind for BridgeStreamErrorKind {
    fn report(&self) -> (DenialClass, Disposition, &'static str) {
        match self {
            Self::UnsupportedConsumerShape => (
                InvalidInput,
                CorrectInput,
                "bridge.stream.unsupported_consumer_shape",
            ),
            Self::UnsupportedResumeMode => (
                InvalidInput,
                CorrectInput,
                "bridge.stream.unsupported_resume_mode",
            ),
            Self::ProtocolVersionMismatch => (
                InvalidInput,
                CorrectInput,
                "bridge.stream.protocol_version_mismatch",
            ),
            Self::CheckpointContractMismatch => (
                Conflict,
                Final,
                "bridge.stream.checkpoint_contract_mismatch",
            ),
            Self::CheckpointStreamMismatch => {
                (Conflict, Final, "bridge.stream.checkpoint_stream_mismatch")
            }
            Self::CheckpointTruncated => {
                (Unavailable, Escalate, "bridge.stream.checkpoint_truncated")
            }
            Self::IllegalCoalescingBoundary => (
                Invariant,
                Escalate,
                "bridge.stream.illegal_coalescing_boundary",
            ),
            Self::NonIdempotentDuplicateObservation => (
                Invariant,
                Escalate,
                "bridge.stream.non_idempotent_duplicate_observation",
            ),
            Self::BackpressurePolicyViolation => (
                Invariant,
                Escalate,
                "bridge.stream.backpressure_policy_violation",
            ),
            Self::StreamReplayMismatch => {
                (Invariant, Escalate, "bridge.stream.stream_replay_mismatch")
            }
            Self::StreamDeliveryRejected => (
                ExternalEffect,
                Escalate,
                "bridge.stream.stream_delivery_rejected",
            ),
            Self::InvalidStreamMaterial => (
                InvalidInput,
                CorrectInput,
                "bridge.stream.invalid_stream_material",
            ),
        }
    }

    fn summary(&self) -> &'static str {
        "Bridge could not process the stream request."
    }
}

impl BridgeErrorKind for BridgeSpeculationErrorKind {
    fn report(&self) -> (DenialClass, Disposition, &'static str) {
        match self {
            Self::PreviewRequestKindMismatch => (
                InvalidInput,
                CorrectInput,
                "bridge.speculation.preview_request_kind_mismatch",
            ),
            Self::PreviewSessionIdentityConflict => (
                Conflict,
                Final,
                "bridge.speculation.preview_session_identity_conflict",
            ),
            Self::PreviewBranchBindingMismatch => (
                Authorization,
                Reauthorize,
                "bridge.speculation.preview_branch_binding_mismatch",
            ),
            Self::IllegalPreviewLifecycleTransition => (
                Conflict,
                Final,
                "bridge.speculation.illegal_preview_lifecycle_transition",
            ),
            Self::PromotionAdmissibilityMismatch => (
                Authorization,
                Reauthorize,
                "bridge.speculation.promotion_admissibility_mismatch",
            ),
            Self::PreviewReuseEquivalenceMismatch => (
                Conflict,
                Final,
                "bridge.speculation.preview_reuse_equivalence_mismatch",
            ),
            Self::PreviewResidueClassificationMismatch => (
                Invariant,
                Escalate,
                "bridge.speculation.preview_residue_classification_mismatch",
            ),
        }
    }

    fn summary(&self) -> &'static str {
        "Bridge could not complete the preview operation."
    }
}

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
            Self::StrategyUnavailable => (
                Unavailable,
                Escalate,
                "bridge.writeback.strategy_unavailable",
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
            Self::CausalityEffectMismatch => (
                Invariant,
                Escalate,
                "bridge.writeback.causality_effect_mismatch",
            ),
            Self::StaleTruthBasis => (
                Conflict,
                Retry(RetryHint::AfterBasisAdvance),
                "bridge.writeback.stale_truth_basis",
            ),
            Self::InvariantRejected => (Invariant, Escalate, "bridge.writeback.invariant_rejected"),
            Self::StrategyFailed => (ExternalEffect, Escalate, "bridge.writeback.strategy_failed"),
            Self::StrategyPanicked => (Defect, Escalate, "bridge.writeback.strategy_panicked"),
            Self::ReplayMismatch => (Invariant, Escalate, "bridge.writeback.replay_mismatch"),
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
