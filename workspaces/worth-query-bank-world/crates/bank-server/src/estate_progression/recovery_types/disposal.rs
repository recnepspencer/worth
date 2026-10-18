//! Disposal owns native terminal proof separately from the Bank transition description.
use super::{BankCommitRecoveryHandle, BankRecoveryTransitionReceipt};
use crate::estate_progression::BankEstateProgressionDenial;
use worth_query_host::facade::primary_graph::WorthQueryRecoveryDisposalReceipt;

/// Actual disposal proof, not a business settlement or physical producer cleanup.
/// A descriptive transition cannot substitute for native disposal:
/// ```compile_fail,E0308
/// fn substitute(value: bank_server::BankRecoveryTransitionReceipt) -> bank_server::BankRecoveryDisposalReceipt { value }
/// ```
/// A real owning result can be transferred without copying:
/// ```
/// fn transfer(value: bank_server::BankRecoveryDisposalReceipt) -> bank_server::BankRecoveryDisposalReceipt { value }
/// ```
#[derive(Debug)]
pub struct BankRecoveryDisposalReceipt {
    transition: BankRecoveryTransitionReceipt,
    disposal: WorthQueryRecoveryDisposalReceipt,
}
impl BankRecoveryDisposalReceipt {
    pub(in crate::estate_progression) fn new(
        transition: BankRecoveryTransitionReceipt,
        disposal: WorthQueryRecoveryDisposalReceipt,
    ) -> Self {
        Self {
            transition,
            disposal,
        }
    }
    pub fn transition_receipt(&self) -> &BankRecoveryTransitionReceipt {
        &self.transition
    }
    pub fn native_disposal_receipt(&self) -> &WorthQueryRecoveryDisposalReceipt {
        &self.disposal
    }
    pub fn into_parts(
        self,
    ) -> (
        BankRecoveryTransitionReceipt,
        WorthQueryRecoveryDisposalReceipt,
    ) {
        (self.transition, self.disposal)
    }
}

/// Opening/admission refusal keeps the original handle; only native terminal consumption removes it.
/// ```compile_fail,E0599
/// fn cannot_copy(value: bank_server::BankRecoveryDisposalDenial) { let _ = value.clone(); }
/// ```
/// The real denial transfers its unique handle on refusal:
/// ```
/// fn transfer(value: bank_server::BankRecoveryDisposalDenial) -> (bank_server::BankEstateProgressionDenial, Option<bank_server::BankCommitRecoveryHandle>) { value.into_parts() }
/// ```
#[derive(Debug)]
pub enum BankRecoveryDisposalDenial {
    Retained {
        denial: BankEstateProgressionDenial,
        handle: Box<BankCommitRecoveryHandle>,
    },
    Terminal(BankEstateProgressionDenial),
}
impl BankRecoveryDisposalDenial {
    pub(in crate::estate_progression) fn retained(
        denial: BankEstateProgressionDenial,
        handle: BankCommitRecoveryHandle,
    ) -> Self {
        Self::Retained {
            denial,
            handle: Box::new(handle),
        }
    }
    pub fn denial(&self) -> &BankEstateProgressionDenial {
        match self {
            Self::Retained { denial, .. } | Self::Terminal(denial) => denial,
        }
    }
    pub fn into_parts(
        self,
    ) -> (
        BankEstateProgressionDenial,
        Option<BankCommitRecoveryHandle>,
    ) {
        match self {
            Self::Retained { denial, handle } => (denial, Some(*handle)),
            Self::Terminal(denial) => (denial, None),
        }
    }
}
