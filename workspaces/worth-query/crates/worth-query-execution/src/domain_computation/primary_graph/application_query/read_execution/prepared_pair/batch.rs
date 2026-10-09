//! Reserved shared-loan work and row custody carried by one closed read.
use super::super::super::{
    batch::BatchReadReservation, WorthQueryAdmittedApplicationQueryPlan,
    WorthQueryApplicationQueryBatchAdmission, WorthQueryApplicationQueryBatchMemory,
    WorthQueryApplicationQueryBatchResourceDenial as Resource,
};
use worth_query_declaration::facade::application_schema::ApplicationSchema;

pub(in crate::domain_computation::primary_graph::application_query) struct PreparedBatchRead {
    reservation: BatchReadReservation,
    rows: WorthQueryApplicationQueryBatchMemory,
    meter: WorthQueryApplicationQueryBatchAdmission,
}
impl PreparedBatchRead {
    pub(in crate::domain_computation::primary_graph::application_query) fn admit<
        S: ApplicationSchema,
        Q,
        P,
        R,
        A,
        I,
        C,
    >(
        plan: &mut WorthQueryAdmittedApplicationQueryPlan<'_, S, Q, P, R, A, I, C>,
        batch: &WorthQueryApplicationQueryBatchAdmission,
    ) -> Result<Self, Resource> {
        match plan.planned_batch_item.take() {
            Some(item) if !item.belongs_to(batch) => return Err(Resource::ForeignPlan),
            Some(_) => {}
            None => {
                batch.take_planned_read(
                    plan.runtime_authority.as_u64(),
                    plan.query.binding_identity(),
                    plan.query.identity(),
                    plan.controls.maximum_work().get(),
                    plan.controls.maximum_result_count().get(),
                )?;
            }
        }
        let rows = batch.claim_memory(
            batch.inline_result_bytes(
                plan.graph_read_plan()
                    .budget_check()
                    .max_inline_result_bytes(),
            ),
        )?;
        let reservation = batch.reserve_read(plan.controls.maximum_work().get())?;
        Ok(Self {
            reservation,
            rows,
            meter: batch.retained_meter(),
        })
    }
    pub(super) fn maximum(&self) -> usize {
        self.reservation.maximum()
    }

    /// No observation keeps the admitted maximum spent; observed success and
    /// failure both settle the native root/tree work. The row claim accompanies
    /// a successful raw result until the owner's projection completes.
    pub(super) fn settle(
        self,
        actual: Option<usize>,
    ) -> Result<Option<WorthQueryApplicationQueryBatchMemory>, Resource> {
        if let Some(actual) = actual {
            self.reservation.settle(actual)?;
            if let Some(denial) = self.meter.memory_denial() {
                return Err(denial);
            }
            Ok(Some(self.rows))
        } else {
            Ok(None)
        }
    }
}
