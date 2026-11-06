use std::collections::BTreeSet;
use std::sync::Arc;

use worth_query_installation::facade::{
    WorthQueryInstalledInvariantExecutionRequirement, WorthQueryInvariantEnforcement,
};

use {crate::domain_computation::vocabulary::provider_session::invariant_execution_denial::WorthQueryInvariantExecutionDenialKind, crate::domain_computation::vocabulary::provider_session::invariant_execution_denial::WorthQueryInvariantExecutionFailure, super::WorthQueryInvariantStructuralCounters};

pub(crate) struct WorthQueryInvariantReceiptMaterial {
    pub(crate) identity: Arc<str>,
    pub(crate) requirement_identity: Arc<str>,
    pub(crate) requirement: WorthQueryInstalledInvariantExecutionRequirement,
    pub(crate) provider_identity: Arc<str>,
    pub(crate) provider_generation: u64,
    pub(crate) session_binding_identity: Arc<str>,
    pub(crate) basis_identity: Arc<str>,
    pub(crate) proposed_state_identity: Arc<str>,
    pub(crate) attempt_generation: u64,
    pub(crate) state_load_plan_identity: Arc<str>,
    pub(crate) state_load_evidence_identity: Arc<str>,
    pub(crate) counters: WorthQueryInvariantStructuralCounters,
    pub(crate) affected_scope: Arc<str>,
    pub(crate) diagnostic_disposition: Arc<str>,
    pub(crate) physical_execution_evidence: Arc<str>,
}

macro_rules! receipt {
    ($name:ident) => {
        pub struct $name {
            pub(crate) material: WorthQueryInvariantReceiptMaterial,
        }

        impl $name {
            pub fn identity(&self) -> &str {
                &self.material.identity
            }

            pub fn invariant_slot(&self) -> &str {
                self.material.requirement.slot()
            }

            pub fn invariant_family(&self) -> &str {
                self.material.requirement.family()
            }

            pub fn invariant_version(&self) -> u32 {
                self.material.requirement.version().get()
            }

            pub fn proposed_state_identity(&self) -> &str {
                &self.material.proposed_state_identity
            }

            pub fn attempt_generation(&self) -> u64 {
                self.material.attempt_generation
            }

            pub fn counters(&self) -> WorthQueryInvariantStructuralCounters {
                self.material.counters
            }

            pub fn affected_scope(&self) -> &str {
                &self.material.affected_scope
            }

            pub fn diagnostic_disposition(&self) -> &str {
                &self.material.diagnostic_disposition
            }

            pub fn provider_identity(&self) -> &str {
                &self.material.provider_identity
            }

            pub fn provider_generation(&self) -> u64 {
                self.material.provider_generation
            }

            pub fn session_binding_identity(&self) -> &str {
                &self.material.session_binding_identity
            }

            pub fn basis_identity(&self) -> &str {
                &self.material.basis_identity
            }

            pub fn state_load_plan_identity(&self) -> &str {
                &self.material.state_load_plan_identity
            }

            pub fn state_load_evidence_identity(&self) -> &str {
                &self.material.state_load_evidence_identity
            }

            pub fn physical_execution_evidence(&self) -> &str {
                &self.material.physical_execution_evidence
            }
        }
    };
}

receipt!(WorthQueryPassedInvariantReceipt);
receipt!(WorthQueryAdvisoryInvariantReceipt);
receipt!(WorthQueryViolatedInvariantReceipt);
receipt!(WorthQueryIndeterminateInvariantReceipt);
receipt!(WorthQueryExhaustedInvariantReceipt);

pub enum WorthQueryInvariantReceipt {
    Passed(WorthQueryPassedInvariantReceipt),
    Advisory(WorthQueryAdvisoryInvariantReceipt),
    Violated(WorthQueryViolatedInvariantReceipt),
    Indeterminate(WorthQueryIndeterminateInvariantReceipt),
    Exhausted(WorthQueryExhaustedInvariantReceipt),
}

#[derive(Debug)]
pub struct WorthQueryInvariantProgressionAuthority {
    receipt_identities: Arc<[Arc<str>]>,
    provider_identity: Arc<str>,
    provider_generation: u64,
    session_binding_identity: Arc<str>,
    basis_identity: Arc<str>,
    proposed_state_identity: Arc<str>,
    attempt_generation: u64,
}

impl WorthQueryInvariantReceipt {
    fn material(&self) -> &WorthQueryInvariantReceiptMaterial {
        match self {
            Self::Passed(receipt) => &receipt.material,
            Self::Advisory(receipt) => &receipt.material,
            Self::Violated(receipt) => &receipt.material,
            Self::Indeterminate(receipt) => &receipt.material,
            Self::Exhausted(receipt) => &receipt.material,
        }
    }

    fn admits_installed_posture(&self) -> bool {
        matches!(
            self,
            Self::Passed(receipt)
                if receipt.material.requirement.enforcement()
                    == WorthQueryInvariantEnforcement::Blocking
        ) || matches!(
            self,
            Self::Advisory(receipt)
                if receipt.material.requirement.enforcement()
                    == WorthQueryInvariantEnforcement::Advisory
        )
    }
}

impl crate::domain_computation::provider_session::WorthQueryProposedStateInspection<'_> {
    pub fn admit_invariant_progression(
        &self,
        receipts: impl IntoIterator<Item = WorthQueryInvariantReceipt>,
    ) -> Result<WorthQueryInvariantProgressionAuthority, WorthQueryInvariantExecutionFailure> {
        let requirements = self.proposed.attempt.staged.plan().invariant_requirements();
        let receipts = receipts.into_iter().collect::<Vec<_>>();
        if receipts.len() != requirements.len() {
            return Err(posture_failure());
        }
        let slots = receipts
            .iter()
            .map(|receipt| receipt.material().requirement.slot())
            .collect::<BTreeSet<_>>();
        let exact_slots = requirements
            .iter()
            .map(|requirement| requirement.slot())
            .collect::<BTreeSet<_>>();
        let plan = self.proposed.attempt.staged.plan();
        if slots != exact_slots
            || receipts.iter().any(|receipt| {
                let material = receipt.material();
                let expected = requirements
                    .iter()
                    .find(|requirement| requirement.slot() == material.requirement.slot());
                !receipt.admits_installed_posture()
                    || material.proposed_state_identity.as_ref() != self.proposed.identity()
                    || material.attempt_generation != self.proposed.generation()
                    || material.provider_identity.as_ref() != plan.provider_identity()
                    || material.provider_generation != plan.provider_generation()
                    || material.session_binding_identity.as_ref()
                        != self.proposed.attempt.staged.provisional_binding_identity()
                    || material.basis_identity.as_ref() != plan.basis_identity()
                    || expected.is_none_or(|requirement| {
                        material.requirement_identity.as_ref()
                            != super::requirement_identity(requirement)
                    })
            })
        {
            return Err(posture_failure());
        }
        let mut identities = receipts
            .into_iter()
            .map(|receipt| Arc::clone(&receipt.material().identity))
            .collect::<Vec<_>>();
        identities.sort();
        Ok(WorthQueryInvariantProgressionAuthority {
            receipt_identities: identities.into(),
            provider_identity: plan.provider_identity().into(),
            provider_generation: plan.provider_generation(),
            session_binding_identity: self
                .proposed
                .attempt
                .staged
                .provisional_binding_identity()
                .into(),
            basis_identity: plan.basis_identity().into(),
            proposed_state_identity: self.proposed.identity().into(),
            attempt_generation: self.proposed.generation(),
        })
    }
}

impl WorthQueryInvariantProgressionAuthority {
    pub fn receipt_identities(&self) -> &[Arc<str>] {
        &self.receipt_identities
    }

    pub fn proposed_state_identity(&self) -> &str {
        &self.proposed_state_identity
    }

    pub fn attempt_generation(&self) -> u64 {
        self.attempt_generation
    }

    pub(crate) fn belongs_to(
        &self,
        provider_identity: &str,
        provider_generation: u64,
        session_binding_identity: &str,
        basis_identity: &str,
        proposed_state_identity: &str,
        attempt_generation: u64,
    ) -> bool {
        self.provider_identity.as_ref() == provider_identity
            && self.provider_generation == provider_generation
            && self.session_binding_identity.as_ref() == session_binding_identity
            && self.basis_identity.as_ref() == basis_identity
            && self.proposed_state_identity.as_ref() == proposed_state_identity
            && self.attempt_generation == attempt_generation
    }
}

fn posture_failure() -> WorthQueryInvariantExecutionFailure {
    WorthQueryInvariantExecutionFailure::new(
        WorthQueryInvariantExecutionDenialKind::VerdictPostureMismatch,
        "invariant verdict cannot mint progression for the installed posture",
    )
}
