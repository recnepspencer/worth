use worth_signal::facade::{ResourceAttemptId, ResourceRequestHandle};
use worth_store_physical_backend::BackendQueueExecutionPlanBinding;

use super::DispatchedPhysicalWork;
use crate::physical_runtime::work::{
    PhysicalWorkEffectClass, PhysicalWorkEffectFate, PhysicalWorkRecoveryDisposition,
    PhysicalWorkSettlementEvidence, PhysicalWorkTerminalStage,
};

pub struct SettledPhysicalWork {
    dispatched: DispatchedPhysicalWork,
    evidence: PhysicalWorkSettlementEvidence,
    recovery: PhysicalWorkRecoveryDisposition,
}

impl SettledPhysicalWork {
    pub const fn intent(&self) -> &crate::physical_runtime::work::PhysicalWorkIntent {
        self.dispatched.intent()
    }

    pub const fn evidence(&self) -> &PhysicalWorkSettlementEvidence {
        &self.evidence
    }

    pub(in crate::physical_runtime) fn into_evidence(self) -> PhysicalWorkSettlementEvidence {
        self.evidence
    }

    pub fn effect_identity(&self) -> Option<crate::physical_runtime::PhysicalEffectIdentity> {
        let backend = match &self.evidence {
            PhysicalWorkSettlementEvidence::Inspection { physical, .. } => physical.operation(),
            PhysicalWorkSettlementEvidence::InspectionDenied(_) => return None,
            PhysicalWorkSettlementEvidence::Metadata { physical, .. } => physical.operation(),
            PhysicalWorkSettlementEvidence::Read { physical, .. } => physical.operation(),
            PhysicalWorkSettlementEvidence::Write { physical, .. }
            | PhysicalWorkSettlementEvidence::Publication { physical, .. } => physical.operation(),
            PhysicalWorkSettlementEvidence::NewArtifact { physical, .. } => {
                physical.write_operation()
            }
            #[cfg(feature = "recovery-runtime-owner")]
            PhysicalWorkSettlementEvidence::RecoveryStaging { physical, .. } => {
                if let Some(created) = physical.created() {
                    created.write_operation()
                } else if let Some(appended) = physical.appended() {
                    appended.operation()
                } else if let Some(verified) = physical.verified() {
                    verified.operation()
                } else {
                    return None;
                }
            }
            PhysicalWorkSettlementEvidence::PublicationEffect { physical, .. } => {
                physical.physical().operation()
            }
            PhysicalWorkSettlementEvidence::WalAppend { physical, .. } => physical.operation(),
            PhysicalWorkSettlementEvidence::WalSegmentCreate { physical, .. } => {
                physical.write_operation()
            }
            PhysicalWorkSettlementEvidence::WalBarrier { physical, .. } => {
                physical.physical().operation()
            }
            PhysicalWorkSettlementEvidence::Checkpoint { physical, .. } => physical.operation(),
            PhysicalWorkSettlementEvidence::WalReclamation { physical, .. } => physical.operation(),
            PhysicalWorkSettlementEvidence::TerminalFailure(failure) => failure.backend_operation(),
            PhysicalWorkSettlementEvidence::NoEffect(_)
            | PhysicalWorkSettlementEvidence::StaleOrForeign => return None,
        };
        Some(crate::physical_runtime::PhysicalEffectIdentity::new(
            self.intent().identity(),
            backend,
        ))
    }

    pub const fn signal_request(&self) -> ResourceRequestHandle {
        self.dispatched.signal_request()
    }

    pub const fn request_attempt(&self) -> ResourceAttemptId {
        self.dispatched.request_attempt()
    }

    pub const fn scheduler_binding(&self) -> BackendQueueExecutionPlanBinding {
        self.dispatched.scheduler_binding()
    }

    pub const fn recovery_disposition(&self) -> PhysicalWorkRecoveryDisposition {
        self.recovery
    }

    pub(in crate::physical_runtime) const fn signal_binding(
        &self,
    ) -> crate::physical_runtime::work::PhysicalSignalAspectBindingDigest {
        self.dispatched.admitted.authority().binding()
    }

    pub(in crate::physical_runtime) const fn signal_family(
        &self,
    ) -> crate::physical_runtime::work::PhysicalWorkSignalFamily {
        self.dispatched.admitted.authority().signal_family()
    }

    pub(in crate::physical_runtime) fn retry_is_physically_safe(&self) -> bool {
        if self.evidence.fate() != PhysicalWorkEffectFate::ProvenNoEffect
            || self.recovery != self.intent().recovery()
        {
            return false;
        }
        matches!(
            (self.intent().effect(), self.intent().recovery()),
            (
                PhysicalWorkEffectClass::ReadOnly,
                PhysicalWorkRecoveryDisposition::NoEffect,
            ) | (
                PhysicalWorkEffectClass::IdempotentExactWrite,
                PhysicalWorkRecoveryDisposition::RetryExact,
            )
        )
    }

    pub(in crate::physical_runtime) const fn signal_evidence(
        &self,
    ) -> &super::super::PhysicalSignalReadinessEvidence {
        self.dispatched.signal_evidence()
    }

    pub(in crate::physical_runtime) fn from_settlement(
        mut dispatched: DispatchedPhysicalWork,
        evidence: PhysicalWorkSettlementEvidence,
        recovery_obligation: crate::physical_runtime::PhysicalEffectRecoveryObligation,
    ) -> Self {
        dispatched.release_scheduler_capacity();
        dispatched
            .admitted
            .mark_stage(PhysicalWorkTerminalStage::Settling);
        let recovery = if recovery_obligation.is_retained() {
            PhysicalWorkRecoveryDisposition::InspectionRequired
        } else {
            evidence.recovery_disposition(dispatched.intent().recovery())
        };
        if !recovery_obligation.is_retained()
            && retry_is_physically_safe(dispatched.intent(), &evidence)
        {
            dispatched.admitted.mark_retry_pending();
        } else {
            dispatched
                .admitted
                .release_settled(evidence.fate(), recovery);
        }
        Self {
            dispatched,
            evidence,
            recovery,
        }
    }

    pub(in crate::physical_runtime) fn into_retry_parts(
        self,
        admitted: worth_signal::facade::AdmittedResourceRetry,
    ) -> Option<(
        super::super::ReadyPhysicalWork,
        crate::physical_runtime::work::PhysicalRetryCommand,
    )> {
        if !self.retry_is_physically_safe() {
            return None;
        }
        let identity = self.intent().identity();
        let signal = self.dispatched.signal.for_retry(admitted);
        let retry = match self.evidence {
            PhysicalWorkSettlementEvidence::NoEffect(evidence) => evidence.retry,
            _ => return None,
        };
        Some((
            super::super::ReadyPhysicalWork::new(self.dispatched.admitted, signal),
            crate::physical_runtime::work::PhysicalRetryCommand::new(identity, retry),
        ))
    }
}

fn retry_is_physically_safe(
    intent: &crate::physical_runtime::work::PhysicalWorkIntent,
    evidence: &PhysicalWorkSettlementEvidence,
) -> bool {
    if evidence.fate() != PhysicalWorkEffectFate::ProvenNoEffect {
        return false;
    }
    matches!(
        (intent.effect(), intent.recovery()),
        (
            PhysicalWorkEffectClass::ReadOnly,
            PhysicalWorkRecoveryDisposition::NoEffect,
        ) | (
            PhysicalWorkEffectClass::IdempotentExactWrite,
            PhysicalWorkRecoveryDisposition::RetryExact,
        )
    )
}
