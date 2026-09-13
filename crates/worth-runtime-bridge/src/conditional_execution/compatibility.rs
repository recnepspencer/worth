mod correspondence;
mod provider_affinity;
mod work;

pub use work::{
    BridgeConditionalComparisonWork, BridgeConditionalContinuityDenial,
    BridgeConditionalExecutionAffinityDenial,
};

use std::sync::Arc;

use super::BridgeInstalledConditionalLowering;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BridgeConditionalProviderRole {
    Condition,
    DependencyComparator,
    OutputComparator,
    ArtifactReuseComparator,
    Trigger,
    Wake,
    Compute,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BridgeConditionalLoweringAdmissionError {
    NodeNotLive,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BridgeConditionalContinuityMismatch {
    CandidateLoweringNotLive,
    ConditionalContract,
    Location,
    CorrespondenceCount,
    DependencyOrdinal { ordinal: usize },
    DependencyMeaning { ordinal: usize },
    DependencySource { ordinal: usize },
    GraphAdapter { ordinal: usize },
    SourceProfile { ordinal: usize },
    TargetCount { ordinal: usize },
    TargetMeaning { ordinal: usize, target: usize },
    Signal(worth_signal::facade::SignalConditionalSemanticMismatch),
    ProviderAdmission,
    ProviderSemanticContract { role: BridgeConditionalProviderRole },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BridgeConditionalExecutionAffinityMismatch {
    Continuity(BridgeConditionalContinuityMismatch),
    CurrentLoweringNotLive,
    BridgeRuntime,
    SourceCorrespondenceAuthority { ordinal: usize },
    GraphAuthority { ordinal: usize },
    GraphParticipationAuthority { ordinal: usize },
    SignalGraphBinding { ordinal: usize },
    TargetAffinity { ordinal: usize, target: usize },
    Signal(worth_signal::facade::SignalConditionalExecutionAffinityMismatch),
    ProviderIdentity { role: BridgeConditionalProviderRole },
}

impl From<BridgeConditionalContinuityMismatch> for BridgeConditionalExecutionAffinityMismatch {
    fn from(value: BridgeConditionalContinuityMismatch) -> Self {
        Self::Continuity(value)
    }
}

#[must_use]
pub struct BridgeConditionalLoweringRetention {
    lowering: Arc<BridgeInstalledConditionalLowering>,
}

#[must_use]
pub struct BridgeLiveConditionalLowering {
    retention: BridgeConditionalLoweringRetention,
}

#[must_use]
pub struct BridgeConditionalLoweringContinuity {
    current: BridgeConditionalLoweringRetention,
    candidate: BridgeLiveConditionalLowering,
    work: BridgeConditionalComparisonWork,
}

impl std::fmt::Debug for BridgeConditionalLoweringContinuity {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("BridgeConditionalLoweringContinuity")
            .finish_non_exhaustive()
    }
}

#[must_use]
pub struct BridgeConditionalExecutionAffinity {
    continuity: BridgeConditionalLoweringContinuity,
    current_live: BridgeLiveConditionalLowering,
    work: BridgeConditionalComparisonWork,
}

impl std::fmt::Debug for BridgeConditionalExecutionAffinity {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("BridgeConditionalExecutionAffinity")
            .finish_non_exhaustive()
    }
}

impl BridgeInstalledConditionalLowering {
    pub fn retain_conditional_lowering(self: &Arc<Self>) -> BridgeConditionalLoweringRetention {
        BridgeConditionalLoweringRetention {
            lowering: Arc::clone(self),
        }
    }

    pub fn admit_live_conditional_lowering(
        self: &Arc<Self>,
    ) -> Result<BridgeLiveConditionalLowering, BridgeConditionalLoweringAdmissionError> {
        let retention = self.retain_conditional_lowering();
        if !self.lease.is_live() {
            return Err(BridgeConditionalLoweringAdmissionError::NodeNotLive);
        }
        Ok(BridgeLiveConditionalLowering { retention })
    }

    pub fn compare_semantic_continuity(
        self: &Arc<Self>,
        candidate: &Arc<Self>,
    ) -> Result<BridgeConditionalLoweringContinuity, BridgeConditionalContinuityDenial> {
        let mut work = BridgeConditionalComparisonWork::default();
        let current_retention = self.retain_conditional_lowering();
        work.inspect_liveness();
        let candidate_liveness = candidate.admit_live_conditional_lowering().map_err(|_| {
            BridgeConditionalContinuityDenial::new(
                BridgeConditionalContinuityMismatch::CandidateLoweringNotLive,
                work,
            )
        })?;
        work.record_bridge_contract(1);
        if self.contract != candidate.contract {
            return Err(BridgeConditionalContinuityDenial::new(
                BridgeConditionalContinuityMismatch::ConditionalContract,
                work,
            ));
        }
        if self.location != candidate.location {
            return Err(BridgeConditionalContinuityDenial::new(
                BridgeConditionalContinuityMismatch::Location,
                work,
            ));
        }
        correspondence::compare_semantic_correspondences(
            &self.correspondences,
            &candidate.correspondences,
            &mut work,
        )
        .map_err(|mismatch| BridgeConditionalContinuityDenial::new(mismatch, work))?;
        let _signal = self
            .signal_contract()
            .compare_semantic_continuity(candidate.signal_contract())
            .map_err(|denial| {
                work.record_signal(denial.work());
                BridgeConditionalContinuityDenial::new(
                    BridgeConditionalContinuityMismatch::Signal(denial.mismatch().clone()),
                    work,
                )
            })?;
        work.record_signal(_signal.work());
        provider_affinity::compare_provider_admissions(
            &self.provider_admission,
            &candidate.provider_admission,
            &mut work,
        )
        .map_err(|mismatch| BridgeConditionalContinuityDenial::new(mismatch, work))?;
        Ok(BridgeConditionalLoweringContinuity {
            current: current_retention,
            candidate: candidate_liveness,
            work,
        })
    }

    pub fn compare_execution_affinity(
        self: &Arc<Self>,
        candidate: &Arc<Self>,
    ) -> Result<BridgeConditionalExecutionAffinity, BridgeConditionalExecutionAffinityDenial> {
        let mut work = BridgeConditionalComparisonWork::default();
        work.inspect_liveness();
        let current_live = self.admit_live_conditional_lowering().map_err(|_| {
            BridgeConditionalExecutionAffinityDenial::new(
                BridgeConditionalExecutionAffinityMismatch::CurrentLoweringNotLive,
                work,
            )
        })?;
        let continuity = self
            .compare_semantic_continuity(candidate)
            .map_err(|denial| {
                let mut denial_work = denial.work();
                denial_work.inspect_liveness();
                BridgeConditionalExecutionAffinityDenial::new(
                    BridgeConditionalExecutionAffinityMismatch::Continuity(
                        denial.mismatch().clone(),
                    ),
                    denial_work,
                )
            })?;
        work = continuity.work();
        work.inspect_liveness();
        work.inspect_bridge_affinity();
        correspondence::compare_exact_correspondences(
            &self.correspondences,
            &candidate.correspondences,
            &mut work,
        )
        .map_err(|mismatch| BridgeConditionalExecutionAffinityDenial::new(mismatch, work))?;
        if self.bridge_runtime_key != candidate.bridge_runtime_key {
            return Err(BridgeConditionalExecutionAffinityDenial::new(
                BridgeConditionalExecutionAffinityMismatch::BridgeRuntime,
                work,
            ));
        }
        let _signal = self
            .signal_contract()
            .compare_execution_affinity(candidate.signal_contract())
            .map_err(|denial| {
                work.record_signal(denial.work());
                BridgeConditionalExecutionAffinityDenial::new(
                    BridgeConditionalExecutionAffinityMismatch::Signal(denial.mismatch().clone()),
                    work,
                )
            })?;
        work.record_signal(_signal.work());
        provider_affinity::compare_provider_affinity(
            &self.providers,
            &candidate.providers,
            &mut work,
        )
        .map_err(|mismatch| BridgeConditionalExecutionAffinityDenial::new(mismatch, work))?;
        Ok(BridgeConditionalExecutionAffinity {
            continuity,
            current_live,
            work,
        })
    }
}

impl BridgeConditionalLoweringRetention {
    pub fn lowering(&self) -> &Arc<BridgeInstalledConditionalLowering> {
        &self.lowering
    }
}

impl BridgeLiveConditionalLowering {
    pub fn retention(&self) -> &BridgeConditionalLoweringRetention {
        &self.retention
    }

    pub fn is_live(&self) -> bool {
        self.retention.lowering.lease.is_live()
    }
}

impl BridgeConditionalLoweringContinuity {
    pub const fn work(&self) -> BridgeConditionalComparisonWork {
        self.work
    }
    pub fn current_retention(&self) -> &BridgeConditionalLoweringRetention {
        &self.current
    }

    pub fn candidate_liveness(&self) -> &BridgeLiveConditionalLowering {
        &self.candidate
    }

    pub fn candidate_is_live(&self) -> bool {
        self.candidate.is_live()
    }
}

impl BridgeConditionalExecutionAffinity {
    pub const fn work(&self) -> BridgeConditionalComparisonWork {
        self.work
    }
    pub fn continuity(&self) -> &BridgeConditionalLoweringContinuity {
        &self.continuity
    }

    pub fn both_are_live(&self) -> bool {
        self.current_live.is_live() && self.continuity.candidate_is_live()
    }
}
