//! The original pure Query owner exists before actual World construction.
use super::durable_custody::{
    QueryDurableWorldCustody, QueryPreparingApplicationCommit, RetainedQueryDurablePreparation,
};
use super::WorthQueryPrimaryGraphProvider;
use crate::domain_computation::primary_graph::provider::{
    BeforeWorldQueryReservation, QueryDurablePreparationGuard,
};
pub(super) struct BeforeWorldPreparation {
    query: Option<QueryPreparingApplicationCommit>,
    reservation: Option<BeforeWorldQueryReservation>,
    world: Option<worth_runtime_world::facade::WorldPendingOpenAttempt>,
}
impl BeforeWorldPreparation {
    pub(super) fn new(
        query: QueryPreparingApplicationCommit,
        reservation: BeforeWorldQueryReservation,
    ) -> Self {
        Self {
            query: Some(query),
            reservation: Some(reservation),
            world: None,
        }
    }
    pub(super) fn reserve_successor_observations(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        provider: &WorthQueryPrimaryGraphProvider,
    ) -> Result<
        (bool, bool, bool),
        crate::domain_computation::primary_graph::live_delivery::WorthQueryLiveAdmissionDenial,
    > {
        self.query
            .as_mut()
            .expect("original staged Query")
            .attempt
            .reserve_successor_observations(execution, provider)
    }
    pub(super) fn refuse(mut self) -> QueryPreparingApplicationCommit {
        let query = self.query.take().expect("same original Query preparation");
        drop(self.reservation.take());
        query
    }
    pub(super) fn bind(
        mut self,
        world: worth_runtime_world::facade::WorldPendingOpenAttempt,
        product: &crate::domain_computation::execution_runtime::product_world::WorthQueryProductPublicationBinding,
        control: crate::domain_computation::execution_runtime::product_world::WorthQueryProductPublicationRequestControl,
        flags: (bool, bool, bool),
    ) -> QueryDurablePreparationGuard {
        // Retain the returned raw phase before binding, checkpoints or yielding.
        self.world = Some(world);
        let handle = self.world.as_ref().unwrap().unpublished_recovery_handle();
        let reservation = self
            .reservation
            .take()
            .unwrap()
            .bind_world(handle.clone(), product.recovery());
        let terminal = crate::domain_computation::execution_runtime::product_world::WorthQueryReservedProductPublicationReceipt::new(
            product.root_identity(), handle.clone(), flags.0, flags.1, flags.2);
        reservation.begin_query_preparation(RetainedQueryDurablePreparation {
            query: self.query.take().unwrap(),
            stage: crate::domain_computation::WorthQueryProductPreparationStage::Opening,
            world: QueryDurableWorldCustody::PendingOpen(self.world.take().unwrap()),
            recovery_handle: handle,
            control,
            terminal,
            prepared_lineage_slot: None,
            query_denial: None,
            authored_completion: None,
        })
    }
}
