#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum WorthQueryAuthorityLane {
    AuthoritativeTruth,
    BranchLocalTruth,
    PreviewTruth,
    DerivedRuntimeState,
    EffectDeliveryState,
    PendingWriteIntent,
    BridgeExternalState,
    TemporalExecutionState,
    AsyncResourceState,
}

impl WorthQueryAuthorityLane {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::AuthoritativeTruth => "authoritative-truth",
            Self::BranchLocalTruth => "branch-local-truth",
            Self::PreviewTruth => "preview-truth",
            Self::DerivedRuntimeState => "derived-runtime-state",
            Self::EffectDeliveryState => "effect-delivery-state",
            Self::PendingWriteIntent => "pending-write-intent",
            Self::BridgeExternalState => "bridge-external-state",
            Self::TemporalExecutionState => "temporal-execution-state",
            Self::AsyncResourceState => "async-resource-state",
        }
    }
}

impl std::fmt::Display for WorthQueryAuthorityLane {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}
