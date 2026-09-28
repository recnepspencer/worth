use super::{UiAllocationReceiptGeneration, UiAllocationReceiptIdentity, UiAllocationReuseDenial};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiAllocationReceiptDenialCause {
    CandidatePlanningDenied,
    ReuseDenied(UiAllocationReuseDenial),
}

/// Immutable denial lineage. A failed commit never mutates a prior receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiAllocationReceiptDenialReport {
    receipt_identity: UiAllocationReceiptIdentity,
    receipt_generation: UiAllocationReceiptGeneration,
    cause: UiAllocationReceiptDenialCause,
}

impl UiAllocationReceiptDenialReport {
    pub(crate) fn candidate_planning_denied(candidate: &super::UiAllocationCandidate) -> Self {
        Self::new(
            candidate,
            UiAllocationReceiptDenialCause::CandidatePlanningDenied,
        )
    }

    #[cfg(test)]
    pub(crate) fn reuse_denied(
        candidate: &super::UiAllocationCandidate,
        denial: UiAllocationReuseDenial,
    ) -> Self {
        Self::new(
            candidate,
            UiAllocationReceiptDenialCause::ReuseDenied(denial),
        )
    }

    fn new(
        candidate: &super::UiAllocationCandidate,
        cause: UiAllocationReceiptDenialCause,
    ) -> Self {
        Self {
            receipt_identity: UiAllocationReceiptIdentity::from_candidate(candidate),
            receipt_generation: UiAllocationReceiptGeneration::from_candidate(candidate),
            cause,
        }
    }

    pub fn receipt_identity(&self) -> &UiAllocationReceiptIdentity {
        &self.receipt_identity
    }

    pub fn receipt_generation(&self) -> UiAllocationReceiptGeneration {
        self.receipt_generation
    }

    pub fn denial(&self) -> Option<UiAllocationReuseDenial> {
        match self.cause {
            UiAllocationReceiptDenialCause::ReuseDenied(denial) => Some(denial),
            UiAllocationReceiptDenialCause::CandidatePlanningDenied => None,
        }
    }

    pub fn cause(&self) -> UiAllocationReceiptDenialCause {
        self.cause
    }

    pub fn inspection_receipt(
        &self,
    ) -> crate::evidence::UiAllocationReceiptDenialInspectionReceipt {
        crate::evidence::project_allocation_receipt_denial_inspection(self)
    }
}
