use crate::continuity::BridgeContinuityOutcomeClass;
use crate::writeback::{
    BridgeContinuityMutationFamily, BridgeNamingMutationFamily, BridgeNamingMutationOutcome,
    BridgeSymbolicTargetReferenceFamily, BridgeWritebackFailureClass, BridgeWritebackOutcomeClass,
};

pub(super) const fn symbolic_target_family_label(
    family: BridgeSymbolicTargetReferenceFamily,
) -> &'static str {
    match family {
        BridgeSymbolicTargetReferenceFamily::SameBatchDeclaredTarget => {
            "same-batch-declared-target"
        }
    }
}

pub(super) const fn naming_family_label(family: BridgeNamingMutationFamily) -> &'static str {
    match family {
        BridgeNamingMutationFamily::AttachNewTarget => "attach-new-target",
        BridgeNamingMutationFamily::AttachExistingTarget => "attach-existing-target",
        BridgeNamingMutationFamily::RebindTarget => "rebind-target",
        BridgeNamingMutationFamily::Remove => "remove",
    }
}

pub(super) const fn naming_outcome_label(outcome: BridgeNamingMutationOutcome) -> &'static str {
    match outcome {
        BridgeNamingMutationOutcome::AttachedToNewTarget => "attached-to-new-target",
        BridgeNamingMutationOutcome::AttachedToExistingTarget => "attached-to-existing-target",
        BridgeNamingMutationOutcome::ReboundTarget => "rebound-target",
        BridgeNamingMutationOutcome::Removed => "removed",
    }
}

pub(super) const fn continuity_family_label(
    family: BridgeContinuityMutationFamily,
) -> &'static str {
    match family {
        BridgeContinuityMutationFamily::RebindExistingTarget => "rebind-existing-target",
        BridgeContinuityMutationFamily::SplitExistingTarget => "split-existing-target",
    }
}

pub(super) const fn continuity_outcome_label(
    outcome: BridgeContinuityOutcomeClass,
) -> &'static str {
    match outcome {
        BridgeContinuityOutcomeClass::ContinuesAsSingleSuccessor => "continues-as-single-successor",
        BridgeContinuityOutcomeClass::ContinuesAsSplitSuccessors => "continues-as-split-successors",
        BridgeContinuityOutcomeClass::ContinuesViaTruthLoweredCanonicalMergeSuccessor => {
            "continues-via-truth-lowered-canonical-merge-successor"
        }
        BridgeContinuityOutcomeClass::RejectedNoAuthoritativeSuccessor => {
            "rejected-no-authoritative-successor"
        }
        BridgeContinuityOutcomeClass::RejectedAmbiguousSuccessor => "rejected-ambiguous-successor",
        BridgeContinuityOutcomeClass::RejectedUnsupportedContinuityClass => {
            "rejected-unsupported-continuity-class"
        }
        BridgeContinuityOutcomeClass::RejectedHistoricalResolutionFailure => {
            "rejected-historical-resolution-failure"
        }
    }
}

pub(super) const fn writeback_outcome_label(outcome: BridgeWritebackOutcomeClass) -> &'static str {
    match outcome {
        BridgeWritebackOutcomeClass::CanonicalNoop => "canonical-noop",
        BridgeWritebackOutcomeClass::AuthoritativeCommit => "authoritative-commit",
        BridgeWritebackOutcomeClass::Rejected => "rejected",
    }
}

pub(super) const fn writeback_failure_label(failure: BridgeWritebackFailureClass) -> &'static str {
    match failure {
        BridgeWritebackFailureClass::WritebackNotRequested => "writeback-not-requested",
        BridgeWritebackFailureClass::PolicyRejected => "policy-rejected",
        BridgeWritebackFailureClass::StrategyUnavailable => "strategy-unavailable",
        BridgeWritebackFailureClass::FamilyBindingMismatch => "family-binding-mismatch",
        BridgeWritebackFailureClass::StrategyDescriptorMismatch => "strategy-descriptor-mismatch",
        BridgeWritebackFailureClass::IdempotenceBasisMismatch => "idempotence-basis-mismatch",
        BridgeWritebackFailureClass::CausalityEffectMismatch => "causality-effect-mismatch",
        BridgeWritebackFailureClass::StaleTruthBasis => "stale-truth-basis",
        BridgeWritebackFailureClass::InvariantRejected => "invariant-rejected",
        BridgeWritebackFailureClass::StrategyFailed => "strategy-failed",
        BridgeWritebackFailureClass::StrategyPanicked => "strategy-panicked",
        BridgeWritebackFailureClass::ReplayMismatch => "replay-mismatch",
        BridgeWritebackFailureClass::AuthorityDenied => "authority-denied",
        BridgeWritebackFailureClass::PreviewWritebackRejected => "preview-writeback-rejected",
    }
}
