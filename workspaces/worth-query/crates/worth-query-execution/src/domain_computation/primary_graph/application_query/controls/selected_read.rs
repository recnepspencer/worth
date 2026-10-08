//! Read-only controls for the admitted shared selection progression.
use super::{
    NonZeroUsize, PhantomData, WorthQueryApplicationQueryBasis, WorthQueryApplicationQueryControls,
    WorthQueryApplicationQueryLane, WorthQueryRequestScope,
};
use crate::basis::WorthQueryProductObservationLease;
use crate::domain_computation::primary_graph::{
    application_query::resource_lifecycle::WorthQueryApplicationBasisLease,
    output_lineage::invalidation::InvalidationEditAdmission,
};
use worth_relational::facade::mvcc::CompanionPreflightStop;

impl<'request, Schema> WorthQueryApplicationQueryControls<'request, Schema> {
    pub(in crate::domain_computation::primary_graph) fn selected_read_one_shot(
        product: WorthQueryProductObservationLease,
        application_basis: WorthQueryApplicationBasisLease,
        maximum_result_count: NonZeroUsize,
        maximum_work: NonZeroUsize,
        request_scope: &'request WorthQueryRequestScope,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<Self, CompanionPreflightStop> {
        // Each World observation is exactly two Arc increments. This read has
        // no Bridge source; Query cannot turn these controls into publication.
        let clones = if product.current_security_guard().is_some() {
            4
        } else {
            2
        };
        admission.charge_external_work(clones)?;
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
            maximum_work,
            maximum_inline_result_bytes: None,
            request_scope,
            _schema: PhantomData,
        })
    }
}
