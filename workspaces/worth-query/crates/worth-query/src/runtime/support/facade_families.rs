use crate::runtime::{WorthQueryAuthorityLane, WorthQueryEffectPolicy};

#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum WorthQueryRuntimeBackendPosture {
    Primary,
    Scaffold,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum WorthQueryRuntimeBatchAuthority {
    BackendAtomicFull,
    BackendAtomicDirect,
    Unsupported,
}

impl WorthQueryRuntimeBackendPosture {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Primary => "primary",
            Self::Scaffold => "scaffold",
        }
    }
}

impl std::fmt::Display for WorthQueryRuntimeBackendPosture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum WorthQueryRuntimeFacadeFamily {
    Read,
    Live,
    Computed,
    SharedRead,
    Submission,
    Replay,
    Effect,
    BranchPreview,
    Write,
    Intent,
    Inspect,
    Temporal,
    AsyncResource,
    MixedCauseDelivery,
    StoreBackedExecution,
    DurableArtifacts,
}

impl WorthQueryRuntimeFacadeFamily {
    pub const ALL: [Self; 16] = [
        Self::Read,
        Self::Live,
        Self::Computed,
        Self::SharedRead,
        Self::Submission,
        Self::Replay,
        Self::Effect,
        Self::BranchPreview,
        Self::Write,
        Self::Intent,
        Self::Inspect,
        Self::Temporal,
        Self::AsyncResource,
        Self::MixedCauseDelivery,
        Self::StoreBackedExecution,
        Self::DurableArtifacts,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Read => "read",
            Self::Live => "live",
            Self::Computed => "computed",
            Self::SharedRead => "shared-read",
            Self::Submission => "submission",
            Self::Replay => "replay",
            Self::Effect => "effect",
            Self::BranchPreview => "branch-preview",
            Self::Write => "write",
            Self::Intent => "intent",
            Self::Inspect => "inspect",
            Self::Temporal => "temporal",
            Self::AsyncResource => "async-resource",
            Self::MixedCauseDelivery => "mixed-cause-delivery",
            Self::StoreBackedExecution => "store-backed-execution",
            Self::DurableArtifacts => "durable-artifacts",
        }
    }
}

impl std::fmt::Display for WorthQueryRuntimeFacadeFamily {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum WorthQueryRuntimeFamilySupportStatus {
    Supported,
    DeferredDebt,
    Unsupported,
}

impl WorthQueryRuntimeFamilySupportStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Supported => "supported",
            Self::DeferredDebt => "deferred-debt",
            Self::Unsupported => "unsupported",
        }
    }
}

impl std::fmt::Display for WorthQueryRuntimeFamilySupportStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum WorthQueryRuntimeFamilyTeachingPosture {
    OrdinaryRuntimeDx,
    VisibleButDeferred,
    VisibleVocabularyOnly,
    SupportGateOnly,
}

impl WorthQueryRuntimeFamilyTeachingPosture {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::OrdinaryRuntimeDx => "ordinary-runtime-dx",
            Self::VisibleButDeferred => "visible-but-deferred",
            Self::VisibleVocabularyOnly => "visible-vocabulary-only",
            Self::SupportGateOnly => "support-gate-only",
        }
    }
}

impl std::fmt::Display for WorthQueryRuntimeFamilyTeachingPosture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryRuntimeFamilySupport {
    family: WorthQueryRuntimeFacadeFamily,
    status: WorthQueryRuntimeFamilySupportStatus,
    teaching_posture: WorthQueryRuntimeFamilyTeachingPosture,
    authority_lanes: Vec<WorthQueryAuthorityLane>,
    effect_policies: Vec<WorthQueryEffectPolicy>,
    evidence: Vec<String>,
    denial_reason: Option<String>,
}

impl WorthQueryRuntimeFamilySupport {
    pub fn supported(
        family: WorthQueryRuntimeFacadeFamily,
        authority_lanes: impl IntoIterator<Item = WorthQueryAuthorityLane>,
        effect_policies: impl IntoIterator<Item = WorthQueryEffectPolicy>,
        evidence: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        Self::supported_with_teaching_posture(
            family,
            WorthQueryRuntimeFamilyTeachingPosture::OrdinaryRuntimeDx,
            authority_lanes,
            effect_policies,
            evidence,
        )
    }

    pub fn supported_with_teaching_posture(
        family: WorthQueryRuntimeFacadeFamily,
        teaching_posture: WorthQueryRuntimeFamilyTeachingPosture,
        authority_lanes: impl IntoIterator<Item = WorthQueryAuthorityLane>,
        effect_policies: impl IntoIterator<Item = WorthQueryEffectPolicy>,
        evidence: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        Self {
            family,
            status: WorthQueryRuntimeFamilySupportStatus::Supported,
            teaching_posture,
            authority_lanes: authority_lanes.into_iter().collect(),
            effect_policies: effect_policies.into_iter().collect(),
            evidence: evidence.into_iter().map(Into::into).collect(),
            denial_reason: None,
        }
    }

    #[cfg(test)]
    pub(crate) fn supported_with_teaching_posture_and_reason(
        family: WorthQueryRuntimeFacadeFamily,
        teaching_posture: WorthQueryRuntimeFamilyTeachingPosture,
        authority_lanes: impl IntoIterator<Item = WorthQueryAuthorityLane>,
        effect_policies: impl IntoIterator<Item = WorthQueryEffectPolicy>,
        evidence: impl IntoIterator<Item = impl Into<String>>,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            family,
            status: WorthQueryRuntimeFamilySupportStatus::Supported,
            teaching_posture,
            authority_lanes: authority_lanes.into_iter().collect(),
            effect_policies: effect_policies.into_iter().collect(),
            evidence: evidence.into_iter().map(Into::into).collect(),
            denial_reason: Some(reason.into()),
        }
    }

    pub fn unsupported(family: WorthQueryRuntimeFacadeFamily, reason: impl Into<String>) -> Self {
        Self::unsupported_with_evidence(family, reason, std::iter::empty::<String>())
    }

    pub fn unsupported_with_evidence(
        family: WorthQueryRuntimeFacadeFamily,
        reason: impl Into<String>,
        evidence: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        Self {
            family,
            status: WorthQueryRuntimeFamilySupportStatus::Unsupported,
            teaching_posture: WorthQueryRuntimeFamilyTeachingPosture::VisibleVocabularyOnly,
            authority_lanes: Vec::new(),
            effect_policies: Vec::new(),
            evidence: evidence.into_iter().map(Into::into).collect(),
            denial_reason: Some(reason.into()),
        }
    }

    pub fn deferred(family: WorthQueryRuntimeFacadeFamily, reason: impl Into<String>) -> Self {
        Self {
            family,
            status: WorthQueryRuntimeFamilySupportStatus::DeferredDebt,
            teaching_posture: WorthQueryRuntimeFamilyTeachingPosture::VisibleButDeferred,
            authority_lanes: Vec::new(),
            effect_policies: Vec::new(),
            evidence: Vec::new(),
            denial_reason: Some(reason.into()),
        }
    }

    pub fn family(&self) -> WorthQueryRuntimeFacadeFamily {
        self.family
    }

    pub fn status(&self) -> WorthQueryRuntimeFamilySupportStatus {
        self.status
    }

    pub fn teaching_posture(&self) -> WorthQueryRuntimeFamilyTeachingPosture {
        self.teaching_posture
    }

    pub fn authority_lanes(&self) -> &[WorthQueryAuthorityLane] {
        &self.authority_lanes
    }

    pub fn effect_policies(&self) -> &[WorthQueryEffectPolicy] {
        &self.effect_policies
    }

    pub fn evidence(&self) -> &[String] {
        &self.evidence
    }

    pub fn denial_reason(&self) -> Option<&str> {
        self.denial_reason.as_deref()
    }

    pub fn ordinary_downstream_dx(&self) -> bool {
        self.teaching_posture == WorthQueryRuntimeFamilyTeachingPosture::OrdinaryRuntimeDx
    }

    pub fn owner_closure(&self) -> &'static str {
        match (self.family, self.status) {
            (
                WorthQueryRuntimeFacadeFamily::Read
                | WorthQueryRuntimeFacadeFamily::Live
                | WorthQueryRuntimeFacadeFamily::Computed
                | WorthQueryRuntimeFacadeFamily::Effect
                | WorthQueryRuntimeFacadeFamily::BranchPreview
                | WorthQueryRuntimeFacadeFamily::Write
                | WorthQueryRuntimeFacadeFamily::Inspect,
                WorthQueryRuntimeFamilySupportStatus::Supported,
            ) => "Milestone 9.3",
            (
                WorthQueryRuntimeFacadeFamily::SharedRead
                | WorthQueryRuntimeFacadeFamily::Submission
                | WorthQueryRuntimeFacadeFamily::Replay,
                WorthQueryRuntimeFamilySupportStatus::Supported,
            ) => "Milestone 9.7",
            (
                WorthQueryRuntimeFacadeFamily::Intent,
                WorthQueryRuntimeFamilySupportStatus::Unsupported,
            ) => "Milestone 9.x intent-authority-adapter",
            (WorthQueryRuntimeFacadeFamily::Temporal, _) => "Milestone 9.4",
            (WorthQueryRuntimeFacadeFamily::AsyncResource, _) => "Milestone 9.4",
            (WorthQueryRuntimeFacadeFamily::MixedCauseDelivery, _) => "Milestone 9.4",
            (WorthQueryRuntimeFacadeFamily::StoreBackedExecution, _) => "Milestone 10",
            (WorthQueryRuntimeFacadeFamily::DurableArtifacts, _) => "Milestone 11",
            _ => "current-runtime-support-profile",
        }
    }

    pub fn extension_rule(&self) -> &'static str {
        match (self.family, self.teaching_posture) {
            (
                WorthQueryRuntimeFacadeFamily::Temporal
                | WorthQueryRuntimeFacadeFamily::AsyncResource
                | WorthQueryRuntimeFacadeFamily::MixedCauseDelivery,
                WorthQueryRuntimeFamilyTeachingPosture::SupportGateOnly,
            ) => "must-extend-stabilized-handle-state-lane-aspect-inspection-facade",
            (_, WorthQueryRuntimeFamilyTeachingPosture::OrdinaryRuntimeDx) => {
                "stable-runtime-backed-handle-state-lane-aspect-inspection-facade"
            }
            (_, WorthQueryRuntimeFamilyTeachingPosture::VisibleButDeferred) => {
                "must-extend-stabilized-handle-state-lane-aspect-inspection-facade"
            }
            (_, WorthQueryRuntimeFamilyTeachingPosture::VisibleVocabularyOnly) => {
                "must-admit-through-runtime-support-profile-before-public-use"
            }
            (_, WorthQueryRuntimeFamilyTeachingPosture::SupportGateOnly) => {
                "support-matrix-only-certification-gate"
            }
        }
    }

    pub fn parallel_api_forbidden(&self) -> bool {
        true
    }

    pub fn admission_fail_closed(&self) -> bool {
        !matches!(self.status, WorthQueryRuntimeFamilySupportStatus::Supported)
            || self.teaching_posture == WorthQueryRuntimeFamilyTeachingPosture::SupportGateOnly
    }
}
#[cfg(test)]
#[path = "facade_families_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;
