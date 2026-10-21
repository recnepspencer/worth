//! Read-only controls for the admitted shared selection progression.
use super::NonZeroUsize;
use super::PhantomData;
use super::WorthQueryApplicationQueryBasis;
use super::WorthQueryApplicationQueryControls;
use super::WorthQueryApplicationQueryLane;
use super::WorthQueryRequestScope;
use crate::basis::WorthQueryProductObservationLease;
use crate::domain_computation::primary_graph::application_query::resource_lifecycle::WorthQueryApplicationBasisLease;
use worth_relational::facade::mvcc::CompanionPreflightStop;

impl<'request, Schema> WorthQueryApplicationQueryControls<'request, Schema> {
    pub(in crate::domain_computation::primary_graph) fn selected_read_one_shot(
        product: WorthQueryProductObservationLease,
        application_basis: WorthQueryApplicationBasisLease,
        maximum_result_count: NonZeroUsize,

        request_scope: &'request WorthQueryRequestScope,
    ) -> Result<Self, CompanionPreflightStop> {
        // Each World observation is exactly two Arc increments. This read has
        // no Bridge source; Query cannot turn these controls into publication.

        let security_product = product.retained_clone();
        Ok(Self {
            basis: WorthQueryApplicationQueryBasis::Selected {
                product,
                application_basis,
            },
            security_product,
            publication_product: None,
            lane: WorthQueryApplicationQueryLane::OneShot,
            maximum_result_count,

            request_scope,
            _schema: PhantomData,
        })
    }
}
