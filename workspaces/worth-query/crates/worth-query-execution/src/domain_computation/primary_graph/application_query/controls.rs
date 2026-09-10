use std::marker::PhantomData;
use std::num::NonZeroUsize;
use std::time::Instant;

use worth_query_admission::facade::{
    application_query::WorthQueryApplicationQueryLane,
    authenticated_principal::WorthQueryRequestScope,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryApplicationQueryBasisPosture {
    SelectedProduct,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryApplicationQueryConsistency {
    SelectedProductSnapshot,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryApplicationQueryFreshness {
    SelectedAtAdmission,
}

pub struct WorthQueryApplicationQueryControls<'a, Schema> {
    basis: WorthQueryApplicationQueryBasis,
    publication_product: Option<crate::basis::WorthQueryProductBranchLease>,
    lane: WorthQueryApplicationQueryLane,
    maximum_result_count: NonZeroUsize,
    maximum_work: NonZeroUsize,
    request_scope: &'a WorthQueryRequestScope,
    _schema: PhantomData<fn() -> Schema>,
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

pub(super) enum WorthQueryApplicationQueryBasis {
    Selected {
        product: crate::basis::WorthQueryProductObservationLease,
        application_basis: super::resource_lifecycle::WorthQueryApplicationBasisLease,
    },
    RetainedContinuation {
        product: crate::basis::WorthQueryProductObservationLease,
    },
}

impl<'a, Schema> WorthQueryApplicationQueryControls<'a, Schema> {
    pub(in crate::domain_computation::primary_graph) fn publication_product_branch(
        &self,
    ) -> Option<&crate::basis::WorthQueryProductBranchLease> {
        self.publication_product.as_ref()
    }
    pub(in crate::domain_computation::primary_graph) fn product_one_shot(
        product: crate::basis::WorthQueryProductBranchLease,
        application_basis: super::resource_lifecycle::WorthQueryApplicationBasisLease,
        maximum_result_count: NonZeroUsize,
        maximum_work: NonZeroUsize,
        request_scope: &'a WorthQueryRequestScope,
    ) -> Self {
        let read = product.read_lease();
        Self {
            basis: WorthQueryApplicationQueryBasis::Selected {
                product: read,
                application_basis,
            },
            publication_product: Some(product),
            lane: WorthQueryApplicationQueryLane::OneShot,
            maximum_result_count,
            maximum_work,
            request_scope,
            _schema: PhantomData,
        }
    }

    pub(in crate::domain_computation::primary_graph) fn product_continuation(
        product: crate::basis::WorthQueryProductObservationLease,
        application_basis: super::resource_lifecycle::WorthQueryApplicationBasisLease,
        maximum_page_width: NonZeroUsize,
        maximum_work: NonZeroUsize,
        request_scope: &'a WorthQueryRequestScope,
    ) -> Self {
        Self {
            basis: WorthQueryApplicationQueryBasis::Selected {
                product,
                application_basis,
            },
            publication_product: None,
            lane: WorthQueryApplicationQueryLane::Continuation,
            maximum_result_count: maximum_page_width,
            maximum_work,
            request_scope,
            _schema: PhantomData,
        }
    }

    pub(super) fn product_live(
        product: crate::basis::WorthQueryProductObservationLease,
        application_basis: super::resource_lifecycle::WorthQueryApplicationBasisLease,
        maximum_materialized_record_count: NonZeroUsize,
        maximum_work: NonZeroUsize,
        request_scope: &'a WorthQueryRequestScope,
    ) -> Self {
        Self {
            basis: WorthQueryApplicationQueryBasis::Selected {
                product,
                application_basis,
            },
            publication_product: None,
            lane: WorthQueryApplicationQueryLane::Live,
            maximum_result_count: maximum_materialized_record_count,
            maximum_work,
            request_scope,
            _schema: PhantomData,
        }
    }

    pub const fn basis_posture(&self) -> WorthQueryApplicationQueryBasisPosture {
        WorthQueryApplicationQueryBasisPosture::SelectedProduct
    }

    pub const fn consistency(&self) -> WorthQueryApplicationQueryConsistency {
        WorthQueryApplicationQueryConsistency::SelectedProductSnapshot
    }

    pub const fn freshness(&self) -> WorthQueryApplicationQueryFreshness {
        WorthQueryApplicationQueryFreshness::SelectedAtAdmission
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
        WorthQueryApplicationQueryBasis,
        WorthQueryAdmittedApplicationQueryControls<'a>,
    ) {
        let basis_deadline = Some(self.request_scope.deadline());
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
        product: crate::basis::WorthQueryProductObservationLease,
        controls: WorthQueryApplicationQueryResumeControls<'a>,
    ) -> Self {
        Self {
            basis: WorthQueryApplicationQueryBasis::RetainedContinuation { product },
            publication_product: None,
            lane: WorthQueryApplicationQueryLane::Continuation,
            maximum_result_count: controls.maximum_page_width,
            maximum_work: controls.maximum_work,
            request_scope: controls.request_scope,
            _schema: PhantomData,
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
