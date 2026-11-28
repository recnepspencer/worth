use super::{BridgeErrorKind, *};
use worth_execution::report::{DenialClass::*, Disposition::*};

impl BridgeErrorKind for BridgeReplayErrorKind {
    fn report(&self) -> (DenialClass, Disposition, &'static str) {
        match self {
            Self::ObservationRead(_) => (Unavailable, Escalate, "bridge.replay.observation_read"),
            Self::ReplayArtifactsDisabled => (
                Unavailable,
                Escalate,
                "bridge.replay.replay_artifacts_disabled",
            ),
            Self::CanonicalArtifactCoherenceFailure => (
                Invariant,
                Escalate,
                "bridge.replay.canonical_artifact_coherence_failure",
            ),
            Self::StructuralReplayBasisTruncated => (
                Unavailable,
                Escalate,
                "bridge.replay.structural_replay_basis_truncated",
            ),
            Self::BulkPlanReplayMismatch => (
                Invariant,
                Escalate,
                "bridge.replay.bulk_plan_replay_mismatch",
            ),
            Self::HistoricalEvaluationDeclarationMismatch => (
                Conflict,
                Final,
                "bridge.replay.historical_evaluation_declaration_mismatch",
            ),
            Self::HistoricalEvaluationPolicyMismatch => (
                Conflict,
                Final,
                "bridge.replay.historical_evaluation_policy_mismatch",
            ),
            Self::HistoricalEvaluationAuthorityMismatch => (
                Authorization,
                Reauthorize,
                "bridge.replay.historical_evaluation_authority_mismatch",
            ),
            Self::RouteMismatch => (Invariant, Escalate, "bridge.replay.route_mismatch"),
            Self::InvalidationMismatch => {
                (Invariant, Escalate, "bridge.replay.invalidation_mismatch")
            }
            Self::SubscriptionSliceMismatch => (
                Invariant,
                Escalate,
                "bridge.replay.subscription_slice_mismatch",
            ),
            Self::ContinuityRequestMismatch => (
                Invariant,
                Escalate,
                "bridge.replay.continuity_request_mismatch",
            ),
            Self::ContinuityResolutionMismatch => (
                Invariant,
                Escalate,
                "bridge.replay.continuity_resolution_mismatch",
            ),
            Self::ContinuityArtifactMismatch => (
                Invariant,
                Escalate,
                "bridge.replay.continuity_artifact_mismatch",
            ),
            Self::DigestMismatch => (Invariant, Escalate, "bridge.replay.digest_mismatch"),
            Self::PlanningContractMismatch => (
                Invariant,
                Escalate,
                "bridge.replay.planning_contract_mismatch",
            ),
            Self::LoweringContractMismatch => (
                Invariant,
                Escalate,
                "bridge.replay.lowering_contract_mismatch",
            ),
        }
    }

    fn summary(&self) -> &'static str {
        "Bridge could not complete replay."
    }
}

impl BridgeErrorKind for BridgeContinuityErrorKind {
    fn report(&self) -> (DenialClass, Disposition, &'static str) {
        match self {
            Self::ObservationRead(_) => {
                (Unavailable, Escalate, "bridge.continuity.observation_read")
            }
            Self::MissingLineageContext => (
                InvalidInput,
                CorrectInput,
                "bridge.continuity.missing_lineage_context",
            ),
            Self::MissingLineageSource => (
                Unavailable,
                Escalate,
                "bridge.continuity.missing_lineage_source",
            ),
            Self::UnsupportedContinuityClass => (
                InvalidInput,
                CorrectInput,
                "bridge.continuity.unsupported_continuity_class",
            ),
            Self::InvalidContinuityRequestSet => (
                InvalidInput,
                CorrectInput,
                "bridge.continuity.invalid_continuity_request_set",
            ),
            Self::LineageAuthorityMismatch => (
                Authorization,
                Reauthorize,
                "bridge.continuity.lineage_authority_mismatch",
            ),
            Self::HistoricalResolutionFailure => (
                Unavailable,
                Escalate,
                "bridge.continuity.historical_resolution_failure",
            ),
        }
    }

    fn summary(&self) -> &'static str {
        "Bridge could not resolve continuity."
    }
}

impl BridgeErrorKind for BridgeLineageSourceErrorKind {
    fn report(&self) -> (DenialClass, Disposition, &'static str) {
        match self {
            Self::ObservationRead(_) => (
                Unavailable,
                Escalate,
                "bridge.lineage_source.observation_read",
            ),
            Self::UnsupportedContinuityClass => (
                InvalidInput,
                CorrectInput,
                "bridge.lineage_source.unsupported_continuity_class",
            ),
            Self::HistoricalResolutionFailure => (
                Unavailable,
                Escalate,
                "bridge.lineage_source.historical_resolution_failure",
            ),
            Self::DuplicateResolvedLineageIdentities => (
                Invariant,
                Escalate,
                "bridge.lineage_source.duplicate_resolved_lineage_identities",
            ),
            Self::NonCanonicalResolvedLineageIdentities => (
                Invariant,
                Escalate,
                "bridge.lineage_source.non_canonical_resolved_lineage_identities",
            ),
            Self::DuplicateResolvedRecordIdentities => (
                Invariant,
                Escalate,
                "bridge.lineage_source.duplicate_resolved_record_identities",
            ),
            Self::NonCanonicalResolvedRecordIdentities => (
                Invariant,
                Escalate,
                "bridge.lineage_source.non_canonical_resolved_record_identities",
            ),
            Self::DuplicateTraversedEventIds => (
                Invariant,
                Escalate,
                "bridge.lineage_source.duplicate_traversed_event_ids",
            ),
            Self::NonCanonicalTraversedEventIds => (
                Invariant,
                Escalate,
                "bridge.lineage_source.non_canonical_traversed_event_ids",
            ),
        }
    }

    fn summary(&self) -> &'static str {
        "Bridge could not read lineage."
    }
}
