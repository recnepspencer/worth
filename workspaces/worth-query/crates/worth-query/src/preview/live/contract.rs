use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::live::LiveQueryPlan;
use crate::preview::binding::PreviewSessionPlanBinding;
use crate::preview::execution::PreviewExecutionError;
use crate::preview::ScopedPreviewLiveSessionPlanBinding;
use worth_runtime_bridge::facade::BridgePreviewLifecycleStateKind;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PreviewLiveCounters {
    pub(in crate::preview) preview_live_admission_count: usize,
    pub(in crate::preview) preview_live_execution_count: usize,
    pub(in crate::preview) preview_live_lifecycle_check_count: usize,
    pub(in crate::preview) preview_live_drift_denial_count: usize,
    pub(in crate::preview) preview_live_rebind_available_count: usize,
    pub(in crate::preview) preview_live_broad_fallback_denial_count: usize,
}

impl PreviewLiveCounters {
    pub fn preview_live_admission_count(&self) -> usize {
        self.preview_live_admission_count
    }

    pub fn preview_live_execution_count(&self) -> usize {
        self.preview_live_execution_count
    }

    pub fn preview_live_lifecycle_check_count(&self) -> usize {
        self.preview_live_lifecycle_check_count
    }

    pub fn preview_live_drift_denial_count(&self) -> usize {
        self.preview_live_drift_denial_count
    }

    pub fn preview_live_rebind_available_count(&self) -> usize {
        self.preview_live_rebind_available_count
    }

    pub fn preview_live_broad_fallback_denial_count(&self) -> usize {
        self.preview_live_broad_fallback_denial_count
    }

    #[cfg(test)]
    pub(crate) fn absorb(&mut self, other: &Self) {
        self.preview_live_admission_count += other.preview_live_admission_count;
        self.preview_live_execution_count += other.preview_live_execution_count;
        self.preview_live_lifecycle_check_count += other.preview_live_lifecycle_check_count;
        self.preview_live_drift_denial_count += other.preview_live_drift_denial_count;
        self.preview_live_rebind_available_count += other.preview_live_rebind_available_count;
        self.preview_live_broad_fallback_denial_count +=
            other.preview_live_broad_fallback_denial_count;
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PreviewLiveFailureClass {
    EvidenceIdentityDerivationDenied(IdentityDerivationDenial),
    PreviewLiveQueryDigestMismatch,
    PreviewLivePlanDigestMismatch,
    PreviewLiveCollectionDigestMismatch,
    PreviewLiveBasisMismatch,
    PreviewLiveScopedBasisMismatch,
    PreviewLiveLifecycleDrifted,
    PreviewLiveRebindBindingRejected,
    PreviewLiveBroadFallbackForbidden,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreviewLiveError {
    pub(in crate::preview) failure_class: PreviewLiveFailureClass,
    pub(in crate::preview) message: &'static str,
    pub(in crate::preview) counters: PreviewLiveCounters,
}

impl PreviewLiveError {
    pub(in crate::preview) fn identity_refused(
        cause: IdentityDerivationDenial,
        counters: PreviewLiveCounters,
    ) -> Self {
        Self {
            failure_class: PreviewLiveFailureClass::EvidenceIdentityDerivationDenied(cause),
            message: "preview live evidence identity refused",
            counters,
        }
    }
    pub fn failure_class(&self) -> &PreviewLiveFailureClass {
        &self.failure_class
    }

    pub fn message(&self) -> &'static str {
        self.message
    }

    pub fn counters(&self) -> &PreviewLiveCounters {
        &self.counters
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreviewLiveAdmissionReport {
    pub(super) digest: String,
    pub(super) preview_binding_digest: String,
    pub(super) live_subscription_digest: String,
    pub(super) live_family: String,
    pub(super) counters: PreviewLiveCounters,
}

impl PreviewLiveAdmissionReport {
    pub fn digest(&self) -> &str {
        &self.digest
    }

    pub fn preview_binding_digest(&self) -> &str {
        &self.preview_binding_digest
    }

    pub fn live_subscription_digest(&self) -> &str {
        &self.live_subscription_digest
    }

    pub fn live_family(&self) -> &str {
        &self.live_family
    }

    pub fn counters(&self) -> &PreviewLiveCounters {
        &self.counters
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PreviewLiveSessionPlanBinding {
    pub(super) preview_binding: PreviewSessionPlanBinding,
    pub(super) live_plan: LiveQueryPlan,
    pub(super) report: PreviewLiveAdmissionReport,
}

impl PreviewLiveSessionPlanBinding {
    pub(crate) fn preview_binding(&self) -> &PreviewSessionPlanBinding {
        &self.preview_binding
    }

    pub(crate) fn live_plan(&self) -> &LiveQueryPlan {
        &self.live_plan
    }

    pub(crate) fn report(&self) -> &PreviewLiveAdmissionReport {
        &self.report
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreviewLiveExecutionEnvelope {
    pub(in crate::preview) preview_live: ScopedPreviewLiveSessionPlanBinding,
    pub(in crate::preview) counters: PreviewLiveCounters,
}

impl PreviewLiveExecutionEnvelope {
    pub fn preview_live(&self) -> &ScopedPreviewLiveSessionPlanBinding {
        &self.preview_live
    }

    pub fn counters(&self) -> &PreviewLiveCounters {
        &self.counters
    }

    pub fn check_invariants(&self) -> Result<(), PreviewExecutionError> {
        if self.counters.preview_live_admission_count() != 1 {
            return Err(PreviewExecutionError::PreviewExecutionInvariantViolation {
                message:
                    "preview-live execution must preserve exactly one preview-live admission proof",
            });
        }

        if self.counters.preview_live_execution_count() != 1 {
            return Err(PreviewExecutionError::PreviewExecutionInvariantViolation {
                message: "preview-live execution must record exactly one preview-live execution",
            });
        }

        if self.counters.preview_live_lifecycle_check_count() != 0
            || self.counters.preview_live_drift_denial_count() != 0
            || self.counters.preview_live_rebind_available_count() != 0
            || self.counters.preview_live_broad_fallback_denial_count() != 0
        {
            return Err(PreviewExecutionError::PreviewExecutionInvariantViolation {
                message:
                    "steady-state preview-live execution cannot smuggle drift or fallback counters",
            });
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreviewLiveDriftDenied {
    pub(super) prior_preview_live_digest: String,
    pub(super) lifecycle_state_kind: BridgePreviewLifecycleStateKind,
    pub(super) error: PreviewLiveError,
}

impl PreviewLiveDriftDenied {
    pub fn prior_preview_live_digest(&self) -> &str {
        &self.prior_preview_live_digest
    }

    pub fn lifecycle_state_kind(&self) -> BridgePreviewLifecycleStateKind {
        self.lifecycle_state_kind
    }

    pub fn error(&self) -> &PreviewLiveError {
        &self.error
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreviewLiveMaintained {
    pub(super) maintained_preview_live: ScopedPreviewLiveSessionPlanBinding,
    pub(super) counters: PreviewLiveCounters,
}

impl PreviewLiveMaintained {
    pub fn maintained_preview_live(&self) -> &ScopedPreviewLiveSessionPlanBinding {
        &self.maintained_preview_live
    }

    pub fn counters(&self) -> &PreviewLiveCounters {
        &self.counters
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreviewLiveRebindArtifact {
    pub(super) prior_preview_live_digest: String,
    pub(super) rebound_preview_live: ScopedPreviewLiveSessionPlanBinding,
    pub(super) counters: PreviewLiveCounters,
}

impl PreviewLiveRebindArtifact {
    pub fn prior_preview_live_digest(&self) -> &str {
        &self.prior_preview_live_digest
    }

    pub fn rebound_preview_live(&self) -> &ScopedPreviewLiveSessionPlanBinding {
        &self.rebound_preview_live
    }

    pub fn counters(&self) -> &PreviewLiveCounters {
        &self.counters
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PreviewLiveDriftOutcome {
    Maintained(PreviewLiveMaintained),
    DriftDenied(PreviewLiveDriftDenied),
    ExplicitRebindAvailable(PreviewLiveRebindArtifact),
}
