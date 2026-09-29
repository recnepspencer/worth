//! Bank-owned terminal receipt for one completed mandatory review.

use worth_foundational::facade::AspectValue;
use worth_query_host::facade::primary_graph::WorthQueryReviewedElevation;

use super::BankEstateElevationClosureKind;

/// Terminal move-only Bank receipt for the exact reviewed elevation.
#[derive(Debug)]
pub struct BankReviewedEstateElevation {
    query: WorthQueryReviewedElevation,
}

impl BankReviewedEstateElevation {
    pub(super) const fn from_query(query: WorthQueryReviewedElevation) -> Self {
        Self { query }
    }

    pub const fn reviewed_at(&self) -> &AspectValue {
        self.query.reviewed_at()
    }

    pub const fn closure_kind(&self) -> BankEstateElevationClosureKind {
        match self.query.closure_kind() {
            worth_query_host::facade::primary_graph::WorthQueryElevationClosureKind::Revoked => {
                BankEstateElevationClosureKind::Revoked
            }
            worth_query_host::facade::primary_graph::WorthQueryElevationClosureKind::Expired => {
                BankEstateElevationClosureKind::Expired
            }
        }
    }

    pub fn reviewer_differs_from_requester(&self) -> bool {
        self.query.reviewer() != self.query.requester()
    }

    pub fn reviewer_differs_from_approver(&self) -> bool {
        self.query.reviewer() != self.query.approver()
    }

    pub fn review_changed_record_count(&self) -> usize {
        self.query.publication_source().changed_record_count()
    }

    pub const fn review_canonical_work(&self) -> crate::BankCommitCanonicalWorkPhases {
        crate::BankCommitCanonicalWorkPhases::from_query(self.query.canonical_work())
    }
}
