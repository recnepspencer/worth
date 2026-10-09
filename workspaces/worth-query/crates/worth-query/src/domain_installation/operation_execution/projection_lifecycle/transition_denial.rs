use super::WorthQueryProjectionPromotionCounters;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryProjectionTransitionDenialKind {
    ExecutionRequest(
        worth_query_execution::facade::application_contribution::WorthQueryAdvancementDenial,
    ),
    Authority(crate::domain_installation::WorthQueryDomainHandleDenialKind),
    BoundAuthorityMismatch,
    WrongCompatibilityPair,
    StaleCompatibilityAuthority,
    StaleConditionalLowering,
    CandidateStale,
    CandidateRebindRequired,
    CandidateAuthorityRevalidationRequired,
    CandidatePromotion(super::WorthQueryProjectionPromotionDenialKind),
    ConditionalDeferred,
    ConditionalDenied,
    ConditionalFailed,
    ManagedLiveOpen,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WorthQueryProjectionTransitionWork {
    authority_checks: usize,
    compatibility_readmissions: usize,
    candidate: WorthQueryProjectionPromotionCounters,
}

impl WorthQueryProjectionTransitionWork {
    pub(super) fn new() -> Self {
        Self::default()
    }

    pub(super) fn retain_authority_checks(&mut self, checks: usize) {
        self.authority_checks += checks;
    }

    pub(super) fn retain_compatibility_readmission(&mut self) {
        self.compatibility_readmissions = 1;
    }

    pub(super) fn retain_candidate(&mut self, counters: WorthQueryProjectionPromotionCounters) {
        self.candidate = counters;
    }

    pub fn authority_checks(self) -> usize {
        self.authority_checks
    }

    pub fn compatibility_readmissions(self) -> usize {
        self.compatibility_readmissions
    }

    pub fn candidate(self) -> WorthQueryProjectionPromotionCounters {
        self.candidate
    }
}
