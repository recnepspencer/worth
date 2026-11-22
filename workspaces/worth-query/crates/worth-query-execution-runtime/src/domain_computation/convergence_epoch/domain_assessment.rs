use worth_query_admission::facade::domain_computation::WorthQueryAdmittedConvergenceContract;

use crate::domain_computation::{
    WorthQueryBoundGraphExecutionReceipt, WorthQueryRetainedConvergenceCandidateEvidence,
};

pub struct WorthQueryConvergenceAssessment<'a> {
    contract: &'a WorthQueryAdmittedConvergenceContract,
    receipt: &'a WorthQueryBoundGraphExecutionReceipt,
    iteration_ordinal: usize,
    incumbents: &'a [WorthQueryRetainedConvergenceCandidateEvidence],
}

impl<'a> WorthQueryConvergenceAssessment<'a> {
    pub fn new(
        contract: &'a WorthQueryAdmittedConvergenceContract,
        receipt: &'a WorthQueryBoundGraphExecutionReceipt,
        iteration_ordinal: usize,
        incumbents: &'a [WorthQueryRetainedConvergenceCandidateEvidence],
    ) -> Self {
        Self {
            contract,
            receipt,
            iteration_ordinal,
            incumbents,
        }
    }

    pub fn contract(&self) -> &WorthQueryAdmittedConvergenceContract {
        self.contract
    }

    pub fn receipt(&self) -> &WorthQueryBoundGraphExecutionReceipt {
        self.receipt
    }

    pub const fn iteration_ordinal(&self) -> usize {
        self.iteration_ordinal
    }

    pub const fn incumbents(&self) -> &[WorthQueryRetainedConvergenceCandidateEvidence] {
        self.incumbents
    }
}
