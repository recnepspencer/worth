use crate::basis_lifecycle::BasisOperationLane;

use super::{WorthQueryCurrentDomainProjection, WorthQueryProjectionPromotionCounters};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryProjectionPromotionDenialKind {
    ExecutionRequest(
        worth_query_execution::facade::application_contribution::WorthQueryAdvancementDenial,
    ),
    ForeignRuntime,
    DomainNotInstalled,
    BoundAuthorityMismatch,
    LiveSupportUnavailable,
    InstalledReadUnavailable,
    ConditionalLoweringNotLive,
    ConditionalDeferred,
    ConditionalEvaluation,
    ConditionalReentry,
    ManagedLiveOpen,
}

pub struct WorthQueryProjectionPromotionStop<D, O, F, L: BasisOperationLane> {
    current: WorthQueryCurrentDomainProjection<D, O, F, L>,
    kind: WorthQueryProjectionPromotionDenialKind,
    detail: String,
    counters: WorthQueryProjectionPromotionCounters,
}

impl<D, O, F, L: BasisOperationLane> WorthQueryProjectionPromotionStop<D, O, F, L> {
    pub(super) fn new(
        current: WorthQueryCurrentDomainProjection<D, O, F, L>,
        kind: WorthQueryProjectionPromotionDenialKind,
        detail: impl Into<String>,
        counters: WorthQueryProjectionPromotionCounters,
    ) -> Self {
        Self {
            current,
            kind,
            detail: detail.into(),
            counters,
        }
    }

    pub fn kind(&self) -> WorthQueryProjectionPromotionDenialKind {
        self.kind
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }

    pub fn counters(&self) -> WorthQueryProjectionPromotionCounters {
        self.counters
    }

    pub fn into_current(self) -> WorthQueryCurrentDomainProjection<D, O, F, L> {
        self.current
    }
}
