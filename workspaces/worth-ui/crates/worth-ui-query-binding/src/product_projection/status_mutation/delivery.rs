//! Real live disclosure initially; exact current retained snapshot on completed retry.
use super::super::status_owner_error::classify_live_delivery;
use super::advancement::Live;
use super::*;
use worth_query_host::facade::{
    application_entry::WorthQueryApplicationRequest,
    publication::domain_computation::publish_application_result,
};
impl MutationState {
    pub(super) fn deliver(
        &mut self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, WorthUiApplicationSchema>,
        live: &mut Live<'_>,
    ) -> Result<(), WorthUiStatusMutationStop> {
        let retained = self
            .native
            .as_ref()
            .and_then(NativeCustody::retained)
            .ok_or(WorthUiStatusMutationStop::RetainedObservationUnavailable)?;
        let outcome = live
            .next(request)
            .map_err(WorthUiStatusMutationStop::LiveNext)?;
        let update = classify_live_delivery(outcome).map_err(|denial| match denial {
            WorthUiStatusOwnerError::LiveDelivery(stop) => {
                WorthUiStatusMutationStop::LiveDelivery(stop)
            }
            other => WorthUiStatusMutationStop::Setup(other),
        })?;
        if update.product_publication().composite_commit() != retained.selected_commit() {
            return Err(WorthUiStatusMutationStop::PublicationMismatch);
        }
        let current = request
            .retain_current_read()
            .map_err(WorthUiStatusMutationStop::CurrentRead)?;
        if current.selected_commit() != retained.selected_commit() {
            return Err(WorthUiStatusMutationStop::PublicationMismatch);
        }
        let (_, admitted) = update.into_admitted_disclosed();
        let published = publish_application_result(admitted);
        let [row] = published.rows() else {
            return Err(WorthUiStatusMutationStop::Setup(
                WorthUiStatusOwnerError::MissingUniqueRecord,
            ));
        };
        self.publication = Some(WorthUiStatusPublication::from_parts(
            row.clone(),
            published.receipt().clone(),
        ));
        Ok(())
    }
    // Reopening a live subscription cannot replay an earlier cause. This separate
    // current snapshot is admitted and disclosed by Query from the actual retained
    // observation. No live update or prior receipt is relabeled as current evidence.
    pub(super) fn disclose_current_retained(
        &mut self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, WorthUiApplicationSchema>,
    ) -> Result<(), WorthUiStatusMutationStop> {
        let retained = self
            .native
            .as_ref()
            .and_then(NativeCustody::retained)
            .ok_or(WorthUiStatusMutationStop::RetainedObservationUnavailable)?;
        let current = request
            .retain_current_read()
            .map_err(WorthUiStatusMutationStop::CurrentRead)?;
        if current.selected_commit() != retained.selected_commit() {
            return Err(WorthUiStatusMutationStop::PublicationMismatch);
        }
        let published = request
            .at(retained)
            .query(WorthUiStatusQueryRequest::new("platform.pulse.status"))
            .execute()
            .map_err(WorthUiStatusMutationStop::SnapshotQuery)?;
        let current = request
            .retain_current_read()
            .map_err(WorthUiStatusMutationStop::CurrentRead)?;
        if current.selected_commit() != retained.selected_commit() {
            return Err(WorthUiStatusMutationStop::PublicationMismatch);
        }
        let [row] = published.rows() else {
            return Err(WorthUiStatusMutationStop::Setup(
                WorthUiStatusOwnerError::MissingUniqueRecord,
            ));
        };
        self.publication = Some(WorthUiStatusPublication::from_parts(
            row.clone(),
            published.receipt().clone(),
        ));
        Ok(())
    }
}
