//! Bank-owned authority for the approved emergency-access phase.

use worth_query_host::facade::primary_graph::WorthQueryApprovedElevation;

/// Move-only Bank authority proving that the exact estate elevation was approved.
#[derive(Debug)]
pub struct BankApprovedEstateElevation {
    query: WorthQueryApprovedElevation,
}

impl BankApprovedEstateElevation {
    pub(in crate::estate_progression) const fn from_query(
        query: WorthQueryApprovedElevation,
    ) -> Self {
        Self { query }
    }

    pub(crate) const fn query(&self) -> &WorthQueryApprovedElevation {
        &self.query
    }

    pub(crate) fn into_query(self) -> WorthQueryApprovedElevation {
        self.query
    }

    pub fn requester_differs_from_approver(&self) -> bool {
        self.query.requester() != self.query.approver()
    }

    pub const fn approval_changed_record_count(&self) -> usize {
        self.query.approval_changed_record_count()
    }

    pub const fn approval_emitted_effect_count(&self) -> usize {
        self.query.approval_emitted_effect_count()
    }

    pub const fn approval_canonical_work(&self) -> crate::BankCommitCanonicalWorkPhases {
        crate::BankCommitCanonicalWorkPhases::from_query(self.query.approval_canonical_work())
    }

    pub fn approval_retained_preimage_present(&self) -> bool {
        self.query.approval_retained_preimage().is_some()
    }

    pub fn request_retained_preimage_present(&self) -> bool {
        self.query.request_retained_preimage().is_some()
    }
}
