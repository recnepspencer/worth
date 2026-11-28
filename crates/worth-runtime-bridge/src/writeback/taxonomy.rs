#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BridgeWritebackFamilyKind {
    ProjectedStateDiff,
    AspectReconciliation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BridgeWritebackRequestMode {
    ReadOnly,
    WritebackCapable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BridgeWritebackEffectClass {
    ProjectedStateDiff,
    AspectReconciliation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BridgeWritebackStrategyClass {
    ProjectedStateDiffReconciliation,
    AspectReconciliationCommit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BridgeWritebackIdempotenceClass {
    RequireSemanticNoopSuppression,
    AllowRepeatedAuthorityAttempt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BridgeWritebackOutcomeClass {
    CanonicalNoop,
    AuthoritativeCommit,
    Rejected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BridgeWritebackLoopDisposition {
    AllowAuthoritativeAttempt,
    CanonicalNoop,
    RejectAsUnsafeFeedback,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BridgeWritebackRetryDisposition {
    SemanticNoopSuppressionRequired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BridgeWritebackFailureClass {
    WritebackNotRequested,
    PolicyRejected,
    StrategyUnavailable,
    FamilyBindingMismatch,
    StrategyDescriptorMismatch,
    IdempotenceBasisMismatch,
    CausalityEffectMismatch,
    StaleTruthBasis,
    InvariantRejected,
    StrategyFailed,
    StrategyPanicked,
    ReplayMismatch,
    AuthorityDenied,
    PreviewWritebackRejected,
}

#[cfg(test)]
mod tests {
    use super::{
        BridgeWritebackEffectClass, BridgeWritebackFailureClass, BridgeWritebackFamilyKind,
        BridgeWritebackIdempotenceClass, BridgeWritebackLoopDisposition,
        BridgeWritebackOutcomeClass, BridgeWritebackRequestMode, BridgeWritebackRetryDisposition,
        BridgeWritebackStrategyClass,
    };

    #[test]
    fn writeback_taxonomy_remains_closed_world_for_phase_1() {
        assert_eq!(
            [
                BridgeWritebackFamilyKind::ProjectedStateDiff,
                BridgeWritebackFamilyKind::AspectReconciliation,
            ]
            .len(),
            2
        );
        assert_eq!(
            [
                BridgeWritebackRequestMode::ReadOnly,
                BridgeWritebackRequestMode::WritebackCapable
            ]
            .len(),
            2
        );
        assert_eq!(
            [
                BridgeWritebackEffectClass::ProjectedStateDiff,
                BridgeWritebackEffectClass::AspectReconciliation,
            ]
            .len(),
            2
        );
        assert_eq!(
            [
                BridgeWritebackStrategyClass::ProjectedStateDiffReconciliation,
                BridgeWritebackStrategyClass::AspectReconciliationCommit,
            ]
            .len(),
            2
        );
        assert_eq!(
            [
                BridgeWritebackIdempotenceClass::RequireSemanticNoopSuppression,
                BridgeWritebackIdempotenceClass::AllowRepeatedAuthorityAttempt,
            ]
            .len(),
            2
        );
        assert_eq!(
            [
                BridgeWritebackOutcomeClass::CanonicalNoop,
                BridgeWritebackOutcomeClass::AuthoritativeCommit,
                BridgeWritebackOutcomeClass::Rejected,
            ]
            .len(),
            3
        );
        assert_eq!(
            [
                BridgeWritebackLoopDisposition::AllowAuthoritativeAttempt,
                BridgeWritebackLoopDisposition::CanonicalNoop,
                BridgeWritebackLoopDisposition::RejectAsUnsafeFeedback,
            ]
            .len(),
            3
        );
        assert_eq!(
            [BridgeWritebackRetryDisposition::SemanticNoopSuppressionRequired].len(),
            1
        );
        assert_eq!(
            [
                BridgeWritebackFailureClass::WritebackNotRequested,
                BridgeWritebackFailureClass::PolicyRejected,
                BridgeWritebackFailureClass::StrategyUnavailable,
                BridgeWritebackFailureClass::FamilyBindingMismatch,
                BridgeWritebackFailureClass::StrategyDescriptorMismatch,
                BridgeWritebackFailureClass::IdempotenceBasisMismatch,
                BridgeWritebackFailureClass::CausalityEffectMismatch,
                BridgeWritebackFailureClass::StaleTruthBasis,
                BridgeWritebackFailureClass::InvariantRejected,
                BridgeWritebackFailureClass::StrategyFailed,
                BridgeWritebackFailureClass::StrategyPanicked,
                BridgeWritebackFailureClass::ReplayMismatch,
                BridgeWritebackFailureClass::AuthorityDenied,
                BridgeWritebackFailureClass::PreviewWritebackRejected,
            ]
            .len(),
            14
        );
    }
}
