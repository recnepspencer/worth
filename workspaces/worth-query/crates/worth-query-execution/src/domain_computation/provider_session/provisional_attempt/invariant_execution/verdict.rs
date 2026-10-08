use std::sync::Arc;

use worth_query_installation::facade::{
    WorthQueryInstalledInvariantExecutionRequirement, WorthQueryInvariantEnforcement,
};

use super::{
    WorthQueryAdvisoryInvariantReceipt, WorthQueryExhaustedInvariantReceipt,
    WorthQueryIndeterminateInvariantReceipt, WorthQueryInvariantExecutionDenialKind,
    WorthQueryInvariantExecutionFailure, WorthQueryInvariantReceipt,
    WorthQueryInvariantReceiptMaterial, WorthQueryInvariantStructuralCounters,
    WorthQueryPassedInvariantReceipt, WorthQueryViolatedInvariantReceipt,
};

pub struct WorthQueryInvariantVerdictAdmission {
    pub(super) requirement: WorthQueryInstalledInvariantExecutionRequirement,
    pub(super) binding: WorthQueryInvariantReceiptBinding,
    pub(super) load_counters: WorthQueryInvariantStructuralCounters,
    pub(super) max_work_units: Option<u64>,
    pub(super) request:
        Option<worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope>,
}

#[derive(Clone)]
pub(super) struct WorthQueryInvariantReceiptBinding {
    pub(super) requirement_identity: Arc<str>,
    pub(super) provider_identity: Arc<str>,
    pub(super) provider_generation: u64,
    pub(super) session_binding_identity: Arc<str>,
    pub(super) basis_identity: Arc<str>,
    pub(super) proposed_state_identity: Arc<str>,
    pub(super) attempt_generation: u64,
    pub(super) state_load_plan_identity: Arc<str>,
    pub(super) state_load_evidence_identity: Arc<str>,
}

impl WorthQueryInvariantVerdictAdmission {
    pub(super) fn binding(&self) -> &WorthQueryInvariantReceiptBinding {
        &self.binding
    }

    pub fn passed(
        self,
        evidence: WorthQueryInvariantVerdictEvidence,
    ) -> Result<WorthQueryInvariantProviderVerdict, WorthQueryInvariantExecutionFailure> {
        if self.requirement.enforcement() != WorthQueryInvariantEnforcement::Blocking {
            return Err(failure(
                WorthQueryInvariantExecutionDenialKind::VerdictPostureMismatch,
            ));
        }
        Ok(WorthQueryInvariantProviderVerdict::Passed(
            WorthQueryPassedInvariantReceipt {
                material: self.material(evidence)?,
            },
        ))
    }

    pub fn advisory(
        self,
        evidence: WorthQueryInvariantVerdictEvidence,
    ) -> Result<WorthQueryInvariantProviderVerdict, WorthQueryInvariantExecutionFailure> {
        if self.requirement.enforcement() != WorthQueryInvariantEnforcement::Advisory {
            return Err(failure(
                WorthQueryInvariantExecutionDenialKind::VerdictPostureMismatch,
            ));
        }
        Ok(WorthQueryInvariantProviderVerdict::Advisory(
            WorthQueryAdvisoryInvariantReceipt {
                material: self.material(evidence)?,
            },
        ))
    }

    pub fn violated(
        self,
        evidence: WorthQueryInvariantVerdictEvidence,
    ) -> Result<WorthQueryInvariantProviderVerdict, WorthQueryInvariantExecutionFailure> {
        Ok(WorthQueryInvariantProviderVerdict::Violated(
            WorthQueryViolatedInvariantReceipt {
                material: self.material(evidence)?,
            },
        ))
    }

    pub fn indeterminate(
        self,
        evidence: WorthQueryInvariantVerdictEvidence,
    ) -> Result<WorthQueryInvariantProviderVerdict, WorthQueryInvariantExecutionFailure> {
        Ok(WorthQueryInvariantProviderVerdict::Indeterminate(
            WorthQueryIndeterminateInvariantReceipt {
                material: self.material(evidence)?,
            },
        ))
    }

    pub fn exhausted(
        self,
        evidence: WorthQueryInvariantVerdictEvidence,
    ) -> Result<WorthQueryInvariantProviderVerdict, WorthQueryInvariantExecutionFailure> {
        Ok(WorthQueryInvariantProviderVerdict::Exhausted(
            WorthQueryExhaustedInvariantReceipt {
                material: self.material(evidence)?,
            },
        ))
    }

    fn material(
        self,
        evidence: WorthQueryInvariantVerdictEvidence,
    ) -> Result<WorthQueryInvariantReceiptMaterial, WorthQueryInvariantExecutionFailure> {
        super::request_control::check_live(self.request.as_ref())?;
        let total_work = self
            .load_counters
            .load_work_units()
            .checked_add(evidence.counters.execution_work_units())
            .ok_or_else(execution_budget_exhausted)?;
        if self
            .max_work_units
            .is_some_and(|maximum| total_work > maximum)
        {
            return Err(WorthQueryInvariantExecutionFailure::exhausted(
                WorthQueryInvariantExecutionDenialKind::ExecutionBudgetExceeded,
                "invariant load and validator exhausted the installed requirement work restriction",
            ));
        }
        let identity = Arc::clone(&evidence.physical_execution_evidence);
        Ok(WorthQueryInvariantReceiptMaterial {
            identity,
            requirement_identity: self.binding.requirement_identity,
            requirement: self.requirement,
            provider_identity: self.binding.provider_identity,
            provider_generation: self.binding.provider_generation,
            session_binding_identity: self.binding.session_binding_identity,
            basis_identity: self.binding.basis_identity,
            proposed_state_identity: self.binding.proposed_state_identity,
            attempt_generation: self.binding.attempt_generation,
            state_load_plan_identity: self.binding.state_load_plan_identity,
            state_load_evidence_identity: self.binding.state_load_evidence_identity,
            counters: WorthQueryInvariantStructuralCounters::new(
                self.load_counters.loaded_facts(),
                self.load_counters.load_work_units(),
                evidence.counters.execution_work_units(),
            ),
            affected_scope: evidence.affected_scope,
            diagnostic_disposition: evidence.diagnostic_disposition,
            physical_execution_evidence: evidence.physical_execution_evidence,
        })
    }
}

pub struct WorthQueryInvariantVerdictEvidence {
    affected_scope: Arc<str>,
    diagnostic_disposition: Arc<str>,
    physical_execution_evidence: Arc<str>,
    counters: WorthQueryInvariantStructuralCounters,
}

impl WorthQueryInvariantVerdictEvidence {
    pub fn new(
        affected_scope: impl Into<Arc<str>>,
        diagnostic_disposition: impl Into<Arc<str>>,
        physical_execution_evidence: impl Into<Arc<str>>,
        execution_work_units: u64,
    ) -> Result<Self, WorthQueryInvariantExecutionFailure> {
        Ok(Self {
            affected_scope: canonical(affected_scope)?,
            diagnostic_disposition: canonical(diagnostic_disposition)?,
            physical_execution_evidence: canonical(physical_execution_evidence)?,
            counters: WorthQueryInvariantStructuralCounters::new(0, 0, execution_work_units),
        })
    }
}

pub enum WorthQueryInvariantProviderVerdict {
    Passed(WorthQueryPassedInvariantReceipt),
    Advisory(WorthQueryAdvisoryInvariantReceipt),
    Violated(WorthQueryViolatedInvariantReceipt),
    Indeterminate(WorthQueryIndeterminateInvariantReceipt),
    Exhausted(WorthQueryExhaustedInvariantReceipt),
}

impl WorthQueryInvariantProviderVerdict {
    pub(super) fn belongs_to(&self, expected: &WorthQueryInvariantReceiptBinding) -> bool {
        let material = match self {
            Self::Passed(receipt) => &receipt.material,
            Self::Advisory(receipt) => &receipt.material,
            Self::Violated(receipt) => &receipt.material,
            Self::Indeterminate(receipt) => &receipt.material,
            Self::Exhausted(receipt) => &receipt.material,
        };
        material.requirement_identity == expected.requirement_identity
            && material.provider_identity == expected.provider_identity
            && material.provider_generation == expected.provider_generation
            && material.session_binding_identity == expected.session_binding_identity
            && material.basis_identity == expected.basis_identity
            && material.proposed_state_identity == expected.proposed_state_identity
            && material.attempt_generation == expected.attempt_generation
            && material.state_load_plan_identity == expected.state_load_plan_identity
            && material.state_load_evidence_identity == expected.state_load_evidence_identity
    }

    pub(super) fn into_receipt(self) -> WorthQueryInvariantReceipt {
        match self {
            Self::Passed(receipt) => WorthQueryInvariantReceipt::Passed(receipt),
            Self::Advisory(receipt) => WorthQueryInvariantReceipt::Advisory(receipt),
            Self::Violated(receipt) => WorthQueryInvariantReceipt::Violated(receipt),
            Self::Indeterminate(receipt) => WorthQueryInvariantReceipt::Indeterminate(receipt),
            Self::Exhausted(receipt) => WorthQueryInvariantReceipt::Exhausted(receipt),
        }
    }
}

fn canonical(value: impl Into<Arc<str>>) -> Result<Arc<str>, WorthQueryInvariantExecutionFailure> {
    let value = value.into();
    if value.trim().is_empty() || value.trim() != value.as_ref() {
        Err(failure(
            WorthQueryInvariantExecutionDenialKind::ProviderRejected,
        ))
    } else {
        Ok(value)
    }
}

fn failure(kind: WorthQueryInvariantExecutionDenialKind) -> WorthQueryInvariantExecutionFailure {
    WorthQueryInvariantExecutionFailure::new(kind, "invariant execution progression denied")
}

fn execution_budget_exhausted() -> WorthQueryInvariantExecutionFailure {
    WorthQueryInvariantExecutionFailure::exhausted(
        WorthQueryInvariantExecutionDenialKind::ExecutionBudgetExceeded,
        "invariant load and validator work overflowed checked representability",
    )
}
