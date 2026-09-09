use std::num::NonZeroUsize;
use std::time::Instant;

use worth_query_admission::facade::{
    application_query::WorthQueryApplicationQueryLane,
    authenticated_principal::WorthQueryRequestScope,
};

use super::basis::{
    WorthQueryApplicationHistoricalBasis, WorthQueryApplicationPinnedBasis,
    WorthQueryApplicationPreviewBasis,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryApplicationQueryBasisPosture {
    Current,
    Pinned,
    Historical,
    Preview,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryApplicationQueryConsistency {
    Committed,
    PinnedSnapshot,
    HistoricalSnapshot,
    PreviewSnapshot,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryApplicationQueryFreshness {
    CurrentAtAdmission,
    Pinned,
    Historical,
    PreviewAtAdmission,
}

pub struct WorthQueryApplicationQueryControls<'a, Schema> {
    basis: WorthQueryApplicationQueryBasis<Schema>,
    lane: WorthQueryApplicationQueryLane,
    maximum_result_count: NonZeroUsize,
    maximum_work: NonZeroUsize,
    request_scope: &'a WorthQueryRequestScope,
}

pub struct WorthQueryApplicationQueryResumeControls<'a> {
    maximum_page_width: NonZeroUsize,
    maximum_work: NonZeroUsize,
    request_scope: &'a WorthQueryRequestScope,
}

pub struct WorthQueryAdmittedApplicationQueryControls<'a> {
    basis: WorthQueryApplicationQueryBasisPosture,
    consistency: WorthQueryApplicationQueryConsistency,
    freshness: WorthQueryApplicationQueryFreshness,
    lane: WorthQueryApplicationQueryLane,
    basis_deadline: Option<Instant>,
    maximum_result_count: NonZeroUsize,
    maximum_work: NonZeroUsize,
    request_scope: &'a WorthQueryRequestScope,
}

pub(super) enum WorthQueryApplicationQueryBasis<Schema> {
    Product {
        product: crate::basis::WorthQueryProductBranchLease,
        application_basis: super::resource_lifecycle::WorthQueryApplicationBasisLease,
    },
    Current,
    Pinned(WorthQueryApplicationPinnedBasis<Schema>),
    Historical(WorthQueryApplicationHistoricalBasis<Schema>),
    Preview(WorthQueryApplicationPreviewBasis<Schema>),
    Continuation {
        descriptor: worth_relational::facade::branch::RelationalBranchBasisDescriptor,
        retention: worth_relational::facade::branch::RelationalBranchRetentionLease,
    },
}

impl<'a, Schema> WorthQueryApplicationQueryControls<'a, Schema> {
    pub(super) fn has_product_basis(&self) -> bool {
        matches!(self.basis, WorthQueryApplicationQueryBasis::Product { .. })
    }
    pub(in crate::domain_computation::primary_graph) fn product_one_shot(
        product: crate::basis::WorthQueryProductBranchLease,
        application_basis: super::resource_lifecycle::WorthQueryApplicationBasisLease,
        maximum_result_count: NonZeroUsize,
        maximum_work: NonZeroUsize,
        request_scope: &'a WorthQueryRequestScope,
    ) -> Self {
        Self {
            basis: WorthQueryApplicationQueryBasis::Product {
                product,
                application_basis,
            },
            lane: WorthQueryApplicationQueryLane::OneShot,
            maximum_result_count,
            maximum_work,
            request_scope,
        }
    }

    pub fn current_one_shot(
        maximum_result_count: NonZeroUsize,
        maximum_work: NonZeroUsize,
        request_scope: &'a WorthQueryRequestScope,
    ) -> Self {
        Self {
            basis: WorthQueryApplicationQueryBasis::Current,
            lane: WorthQueryApplicationQueryLane::OneShot,
            maximum_result_count,
            maximum_work,
            request_scope,
        }
    }

    pub fn pinned_one_shot(
        basis: WorthQueryApplicationPinnedBasis<Schema>,
        maximum_result_count: NonZeroUsize,
        maximum_work: NonZeroUsize,
        request_scope: &'a WorthQueryRequestScope,
    ) -> Self {
        Self {
            basis: WorthQueryApplicationQueryBasis::Pinned(basis),
            lane: WorthQueryApplicationQueryLane::OneShot,
            maximum_result_count,
            maximum_work,
            request_scope,
        }
    }

    pub fn historical(
        basis: WorthQueryApplicationHistoricalBasis<Schema>,
        maximum_result_count: NonZeroUsize,
        maximum_work: NonZeroUsize,
        request_scope: &'a WorthQueryRequestScope,
    ) -> Self {
        Self {
            basis: WorthQueryApplicationQueryBasis::Historical(basis),
            lane: WorthQueryApplicationQueryLane::Historical,
            maximum_result_count,
            maximum_work,
            request_scope,
        }
    }

    pub fn preview(
        basis: WorthQueryApplicationPreviewBasis<Schema>,
        maximum_result_count: NonZeroUsize,
        maximum_work: NonZeroUsize,
        request_scope: &'a WorthQueryRequestScope,
    ) -> Self {
        Self {
            basis: WorthQueryApplicationQueryBasis::Preview(basis),
            lane: WorthQueryApplicationQueryLane::Preview,
            maximum_result_count,
            maximum_work,
            request_scope,
        }
    }

    pub fn current_continuation_page(
        maximum_page_width: NonZeroUsize,
        maximum_work: NonZeroUsize,
        request_scope: &'a WorthQueryRequestScope,
    ) -> Self {
        Self {
            basis: WorthQueryApplicationQueryBasis::Current,
            lane: WorthQueryApplicationQueryLane::Continuation,
            maximum_result_count: maximum_page_width,
            maximum_work,
            request_scope,
        }
    }

    pub(super) fn current_live(
        maximum_materialized_record_count: NonZeroUsize,
        maximum_work: NonZeroUsize,
        request_scope: &'a WorthQueryRequestScope,
    ) -> Self {
        Self {
            basis: WorthQueryApplicationQueryBasis::Current,
            lane: WorthQueryApplicationQueryLane::Live,
            maximum_result_count: maximum_materialized_record_count,
            maximum_work,
            request_scope,
        }
    }

    pub fn pinned_continuation_page(
        basis: WorthQueryApplicationPinnedBasis<Schema>,
        maximum_page_width: NonZeroUsize,
        maximum_work: NonZeroUsize,
        request_scope: &'a WorthQueryRequestScope,
    ) -> Self {
        Self {
            basis: WorthQueryApplicationQueryBasis::Pinned(basis),
            lane: WorthQueryApplicationQueryLane::Continuation,
            maximum_result_count: maximum_page_width,
            maximum_work,
            request_scope,
        }
    }

    pub const fn basis_posture(&self) -> WorthQueryApplicationQueryBasisPosture {
        match &self.basis {
            WorthQueryApplicationQueryBasis::Product { .. } => {
                WorthQueryApplicationQueryBasisPosture::Pinned
            }
            WorthQueryApplicationQueryBasis::Current => {
                WorthQueryApplicationQueryBasisPosture::Current
            }
            WorthQueryApplicationQueryBasis::Pinned(_) => {
                WorthQueryApplicationQueryBasisPosture::Pinned
            }
            WorthQueryApplicationQueryBasis::Historical(_) => {
                WorthQueryApplicationQueryBasisPosture::Historical
            }
            WorthQueryApplicationQueryBasis::Preview(_) => {
                WorthQueryApplicationQueryBasisPosture::Preview
            }
            WorthQueryApplicationQueryBasis::Continuation { .. } => {
                WorthQueryApplicationQueryBasisPosture::Pinned
            }
        }
    }

    pub const fn consistency(&self) -> WorthQueryApplicationQueryConsistency {
        match self.basis_posture() {
            WorthQueryApplicationQueryBasisPosture::Current => {
                WorthQueryApplicationQueryConsistency::Committed
            }
            WorthQueryApplicationQueryBasisPosture::Pinned => {
                WorthQueryApplicationQueryConsistency::PinnedSnapshot
            }
            WorthQueryApplicationQueryBasisPosture::Historical => {
                WorthQueryApplicationQueryConsistency::HistoricalSnapshot
            }
            WorthQueryApplicationQueryBasisPosture::Preview => {
                WorthQueryApplicationQueryConsistency::PreviewSnapshot
            }
        }
    }

    pub const fn freshness(&self) -> WorthQueryApplicationQueryFreshness {
        match self.basis_posture() {
            WorthQueryApplicationQueryBasisPosture::Current => {
                WorthQueryApplicationQueryFreshness::CurrentAtAdmission
            }
            WorthQueryApplicationQueryBasisPosture::Pinned => {
                WorthQueryApplicationQueryFreshness::Pinned
            }
            WorthQueryApplicationQueryBasisPosture::Historical => {
                WorthQueryApplicationQueryFreshness::Historical
            }
            WorthQueryApplicationQueryBasisPosture::Preview => {
                WorthQueryApplicationQueryFreshness::PreviewAtAdmission
            }
        }
    }

    pub const fn lane(&self) -> WorthQueryApplicationQueryLane {
        self.lane
    }

    pub const fn maximum_result_count(&self) -> NonZeroUsize {
        self.maximum_result_count
    }

    pub const fn maximum_work(&self) -> NonZeroUsize {
        self.maximum_work
    }

    pub const fn request_scope(&self) -> &'a WorthQueryRequestScope {
        self.request_scope
    }

    pub(super) fn into_admission_parts(
        self,
    ) -> (
        WorthQueryApplicationQueryBasis<Schema>,
        WorthQueryAdmittedApplicationQueryControls<'a>,
    ) {
        let basis_deadline = match &self.basis {
            WorthQueryApplicationQueryBasis::Product { .. } => Some(self.request_scope.deadline()),
            WorthQueryApplicationQueryBasis::Current => None,
            WorthQueryApplicationQueryBasis::Pinned(basis) => Some(basis.expires_at()),
            WorthQueryApplicationQueryBasis::Historical(basis) => Some(basis.expires_at()),
            WorthQueryApplicationQueryBasis::Preview(basis) => Some(basis.expires_at()),
            WorthQueryApplicationQueryBasis::Continuation { .. } => {
                Some(self.request_scope.deadline())
            }
        };
        let admitted = WorthQueryAdmittedApplicationQueryControls {
            basis: self.basis_posture(),
            consistency: self.consistency(),
            freshness: self.freshness(),
            lane: self.lane(),
            basis_deadline,
            maximum_result_count: self.maximum_result_count,
            maximum_work: self.maximum_work,
            request_scope: self.request_scope,
        };
        (self.basis, admitted)
    }

    pub(super) fn continuation_resume(
        descriptor: worth_relational::facade::branch::RelationalBranchBasisDescriptor,
        retention: worth_relational::facade::branch::RelationalBranchRetentionLease,
        controls: WorthQueryApplicationQueryResumeControls<'a>,
    ) -> Self {
        Self {
            basis: WorthQueryApplicationQueryBasis::Continuation {
                descriptor,
                retention,
            },
            lane: WorthQueryApplicationQueryLane::Continuation,
            maximum_result_count: controls.maximum_page_width,
            maximum_work: controls.maximum_work,
            request_scope: controls.request_scope,
        }
    }
}

impl<'a> WorthQueryApplicationQueryResumeControls<'a> {
    pub fn new(
        maximum_page_width: NonZeroUsize,
        maximum_work: NonZeroUsize,
        request_scope: &'a WorthQueryRequestScope,
    ) -> Self {
        Self {
            maximum_page_width,
            maximum_work,
            request_scope,
        }
    }

    pub const fn maximum_page_width(&self) -> NonZeroUsize {
        self.maximum_page_width
    }

    pub const fn maximum_work(&self) -> NonZeroUsize {
        self.maximum_work
    }

    pub const fn request_scope(&self) -> &'a WorthQueryRequestScope {
        self.request_scope
    }
}

impl WorthQueryAdmittedApplicationQueryControls<'_> {
    pub const fn basis_posture(&self) -> WorthQueryApplicationQueryBasisPosture {
        self.basis
    }

    pub const fn consistency(&self) -> WorthQueryApplicationQueryConsistency {
        self.consistency
    }

    pub const fn freshness(&self) -> WorthQueryApplicationQueryFreshness {
        self.freshness
    }

    pub const fn lane(&self) -> WorthQueryApplicationQueryLane {
        self.lane
    }

    pub const fn basis_deadline(&self) -> Option<Instant> {
        self.basis_deadline
    }

    pub fn basis_is_expired(&self) -> bool {
        self.basis_deadline
            .is_some_and(|deadline| Instant::now() >= deadline)
    }

    pub const fn maximum_result_count(&self) -> NonZeroUsize {
        self.maximum_result_count
    }

    pub const fn maximum_work(&self) -> NonZeroUsize {
        self.maximum_work
    }

    pub const fn request_scope(&self) -> &WorthQueryRequestScope {
        self.request_scope
    }
}
