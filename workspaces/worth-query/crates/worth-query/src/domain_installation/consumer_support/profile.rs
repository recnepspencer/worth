use crate::runtime::{
    WorthQueryRuntimeFacadeFamily, WorthQueryRuntimeFamilySupportStatus,
    WorthQueryRuntimeSupportProfile,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryConsumerSupportPosture {
    Supported,
    Deferred,
    Unsupported,
}

impl WorthQueryConsumerSupportPosture {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Supported => "supported",
            Self::Deferred => "deferred",
            Self::Unsupported => "unsupported",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryConsumerSupportDimension {
    Basis,
    Live,
    Continuation,
    AsyncResultState,
    Recovery,
    Inspection,
    ProjectionConsumption,
    DependencyImpact,
    Sharing,
    Invalidation,
    CollectionDelivery,
    ConditionalEvaluation,
    ConditionalComparator,
    ConditionalTrigger,
    ConditionalTemporalOrOnDemand,
}

impl WorthQueryConsumerSupportDimension {
    pub const COUNT: usize = 15;
    pub const ALL: [Self; Self::COUNT] = [
        Self::Basis,
        Self::Live,
        Self::Continuation,
        Self::AsyncResultState,
        Self::Recovery,
        Self::Inspection,
        Self::ProjectionConsumption,
        Self::DependencyImpact,
        Self::Sharing,
        Self::Invalidation,
        Self::CollectionDelivery,
        Self::ConditionalEvaluation,
        Self::ConditionalComparator,
        Self::ConditionalTrigger,
        Self::ConditionalTemporalOrOnDemand,
    ];

    pub(crate) const fn index(self) -> usize {
        self as usize
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Basis => "basis",
            Self::Live => "live",
            Self::Continuation => "continuation",
            Self::AsyncResultState => "async-result-state",
            Self::Recovery => "recovery",
            Self::Inspection => "inspection",
            Self::ProjectionConsumption => "projection-consumption",
            Self::DependencyImpact => "dependency-impact",
            Self::Sharing => "sharing",
            Self::Invalidation => "invalidation",
            Self::CollectionDelivery => "collection-delivery",
            Self::ConditionalEvaluation => "conditional-evaluation",
            Self::ConditionalComparator => "conditional-comparator",
            Self::ConditionalTrigger => "conditional-trigger",
            Self::ConditionalTemporalOrOnDemand => "conditional-temporal-or-on-demand",
        }
    }

    fn runtime_family(self) -> Option<WorthQueryRuntimeFacadeFamily> {
        Some(match self {
            Self::Basis => return None,
            Self::Live => WorthQueryRuntimeFacadeFamily::Live,
            Self::Continuation => WorthQueryRuntimeFacadeFamily::Temporal,
            Self::AsyncResultState => WorthQueryRuntimeFacadeFamily::AsyncResource,
            Self::Recovery => WorthQueryRuntimeFacadeFamily::Replay,
            Self::Inspection => WorthQueryRuntimeFacadeFamily::Inspect,
            Self::ProjectionConsumption => WorthQueryRuntimeFacadeFamily::Read,
            Self::DependencyImpact => WorthQueryRuntimeFacadeFamily::Computed,
            Self::Sharing => WorthQueryRuntimeFacadeFamily::SharedRead,
            Self::Invalidation => WorthQueryRuntimeFacadeFamily::MixedCauseDelivery,
            Self::CollectionDelivery => WorthQueryRuntimeFacadeFamily::Read,
            Self::ConditionalEvaluation
            | Self::ConditionalComparator
            | Self::ConditionalTrigger
            | Self::ConditionalTemporalOrOnDemand => return None,
        })
    }

    const fn unsupported_without_registration(self) -> bool {
        matches!(
            self,
            Self::ConditionalEvaluation
                | Self::ConditionalComparator
                | Self::ConditionalTrigger
                | Self::ConditionalTemporalOrOnDemand
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WorthQueryConsumerSupportProfile {
    postures: [WorthQueryConsumerSupportPosture; WorthQueryConsumerSupportDimension::COUNT],
}

impl WorthQueryConsumerSupportProfile {
    pub(crate) fn from_runtime(profile: &WorthQueryRuntimeSupportProfile) -> Self {
        let mut postures = [WorthQueryConsumerSupportPosture::Unsupported;
            WorthQueryConsumerSupportDimension::COUNT];
        for dimension in WorthQueryConsumerSupportDimension::ALL {
            postures[dimension.index()] = match dimension.runtime_family() {
                None if dimension.unsupported_without_registration() => {
                    WorthQueryConsumerSupportPosture::Unsupported
                }
                None => WorthQueryConsumerSupportPosture::Supported,
                Some(family) => profile
                    .support_for(family)
                    .map(|support| match support.status() {
                        WorthQueryRuntimeFamilySupportStatus::Supported => {
                            WorthQueryConsumerSupportPosture::Supported
                        }
                        WorthQueryRuntimeFamilySupportStatus::DeferredDebt => {
                            WorthQueryConsumerSupportPosture::Deferred
                        }
                        WorthQueryRuntimeFamilySupportStatus::Unsupported => {
                            WorthQueryConsumerSupportPosture::Unsupported
                        }
                    })
                    .unwrap_or(WorthQueryConsumerSupportPosture::Unsupported),
            };
        }
        Self { postures }
    }

    pub(crate) fn with_runtime_overrides(
        mut self,
        overrides: [Option<WorthQueryConsumerSupportPosture>;
            WorthQueryConsumerSupportDimension::COUNT],
    ) -> Self {
        for dimension in WorthQueryConsumerSupportDimension::ALL {
            if let Some(posture) = overrides[dimension.index()] {
                self.postures[dimension.index()] = posture;
            }
        }
        self
    }

    pub(crate) fn posture(
        &self,
        dimension: WorthQueryConsumerSupportDimension,
    ) -> WorthQueryConsumerSupportPosture {
        self.postures[dimension.index()]
    }
}
#[cfg(test)]
#[path = "profile_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;
