use super::WorthQueryArtifactNativeAccessCounters;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryArtifactNativeAccessDenialKind {
    AccessPathDenied,
    NativeProviderUnavailable,
    ForeignRuntime,
    StaleInstallationGeneration,
    OperationMismatch,
    RunMismatch,
    StageMismatch,
    BasisMismatch,
    PayloadOwnerMismatch,
    ArtifactContractMismatch,
    ForeignThread,
    ProviderSessionMismatch,
    LayoutMismatch,
    FieldNotDeclared,
    FieldSliceDenied,
    RowBatchDenied,
    ProviderNativeProjectionRequired,
    ChunkingDenied,
    ProjectionDenied,
    ScalarFallbackDenied,
    BoundsExceeded,
    AlignmentMismatch,
    StaleBorrowGeneration,
    AlreadyDisposed,
    ProviderDenied,
    ProviderShapeMismatch,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryArtifactNativeAccessDenial {
    kind: WorthQueryArtifactNativeAccessDenialKind,
    artifact_family: Option<String>,
    detail: &'static str,
    counters: WorthQueryArtifactNativeAccessCounters,
}

impl WorthQueryArtifactNativeAccessDenial {
    pub fn new(
        kind: WorthQueryArtifactNativeAccessDenialKind,
        artifact_family: Option<&str>,
        detail: &'static str,
        counters: WorthQueryArtifactNativeAccessCounters,
    ) -> Self {
        Self {
            kind,
            artifact_family: artifact_family.map(str::to_owned),
            detail,
            counters,
        }
    }

    pub const fn kind(&self) -> WorthQueryArtifactNativeAccessDenialKind {
        self.kind
    }

    pub fn artifact_family(&self) -> Option<&str> {
        self.artifact_family.as_deref()
    }

    pub const fn detail(&self) -> &'static str {
        self.detail
    }

    pub const fn counters(&self) -> WorthQueryArtifactNativeAccessCounters {
        self.counters
    }
}

pub(super) const fn denial_detail(kind: WorthQueryArtifactNativeAccessDenialKind) -> &'static str {
    use WorthQueryArtifactNativeAccessDenialKind as Kind;
    match kind {
        Kind::AccessPathDenied => "installed artifact contract denies native access",
        Kind::NativeProviderUnavailable => "artifact provider exposes no native access contract",
        Kind::ForeignRuntime => "artifact belongs to a different Query runtime",
        Kind::StaleInstallationGeneration => "artifact installation generation is stale",
        Kind::OperationMismatch => "artifact belongs to a different operation binding",
        Kind::RunMismatch => "artifact belongs to a different workflow run",
        Kind::StageMismatch => "artifact belongs to a different workflow stage",
        Kind::BasisMismatch => "artifact belongs to a different admitted basis",
        Kind::PayloadOwnerMismatch => "artifact belongs to a different payload owner",
        Kind::ArtifactContractMismatch => "artifact contract does not match stage authority",
        Kind::ForeignThread => "artifact native access is bound to its creation thread",
        Kind::ProviderSessionMismatch => "provider access session does not match",
        Kind::LayoutMismatch => "provider or request layout does not match installed layout",
        Kind::FieldNotDeclared => "requested artifact field is not uniquely declared",
        Kind::FieldSliceDenied => "installed artifact contract denies this field slice",
        Kind::RowBatchDenied => "installed artifact contract denies borrowed row batches",
        Kind::ProviderNativeProjectionRequired => {
            "artifact field requires a declared provider-native projection"
        }
        Kind::ChunkingDenied => "installed artifact contract denies this chunk bound",
        Kind::ProjectionDenied => "installed artifact contract denies this destination projection",
        Kind::ScalarFallbackDenied => "installed artifact contract denies scalar fallback",
        Kind::BoundsExceeded => "artifact native access exceeds its admitted bounds",
        Kind::AlignmentMismatch => "artifact native access alignment does not match",
        Kind::StaleBorrowGeneration => "artifact native access borrow generation is stale",
        Kind::AlreadyDisposed => "artifact owner is disposed or closed",
        Kind::ProviderDenied => "artifact provider denied native access",
        Kind::ProviderShapeMismatch => "provider native result violates the installed shape",
    }
}
