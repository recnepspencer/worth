//! Bank-owned obligation produced when an estate elevation closes.

use worth_foundational::facade::AspectValue;
use worth_query_host::facade::primary_graph::{
    WorthQueryElevationClosureKind, WorthQueryMandatoryReview,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BankEstateElevationClosureKind {
    Revoked,
    Expired,
}

/// Move-only Bank obligation to complete the exact estate mandatory review.
#[derive(Debug)]
pub struct BankEstateMandatoryReview {
    query: WorthQueryMandatoryReview,
}

impl BankEstateMandatoryReview {
    pub(in crate::estate_progression) const fn from_query(
        query: WorthQueryMandatoryReview,
    ) -> Self {
        Self { query }
    }

    pub(crate) fn into_query(self) -> WorthQueryMandatoryReview {
        self.query
    }

    pub const fn closure_kind(&self) -> BankEstateElevationClosureKind {
        match self.query.closure_kind() {
            WorthQueryElevationClosureKind::Revoked => BankEstateElevationClosureKind::Revoked,
            WorthQueryElevationClosureKind::Expired => BankEstateElevationClosureKind::Expired,
        }
    }

    pub const fn closed_at(&self) -> &AspectValue {
        self.query.closed_at()
    }

    pub fn close_changed_record_count(&self) -> usize {
        self.query.publication_source().changed_record_count()
    }

    pub const fn close_canonical_work(&self) -> crate::BankCommitCanonicalWorkPhases {
        crate::BankCommitCanonicalWorkPhases::from_query(self.query.canonical_work())
    }
}
