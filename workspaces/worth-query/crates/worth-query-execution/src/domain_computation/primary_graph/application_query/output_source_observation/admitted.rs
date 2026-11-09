//! One selected Source dispatcher preserves the original admitted plan and loan.
use super::*;
impl<Schema: ApplicationSchema> SourceQueryReaderAccess<'_, Schema> {
    pub(in crate::domain_computation::primary_graph::application_query) fn read_admitted<B>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        plan: SourceReadPlan<'_, Schema, B>,
        native: &mut ApplicationQueryNativeAccess<'_>,
    ) -> Result<
        super::one_shot::outcome::CompletedOneShotRead<B::Query, SourceValue<Schema, B>>,
        SourceQueryObservationStop,
    >
    where
        B: ApplicationQueryBinding<Schema>,
        SourceValue<Schema, B>: WorthQueryApplicationProjection<Schema, B::Query>,
    {
        let result = if plan.query.read_graph().managed_index_selection().is_some() {
            super::managed_index::read_source_admitted::<Schema, B>(execution, self, plan, native)
        } else {
            self.application
                .execute_application_query_one_shot_admitted_with_native_access(
                    execution, plan, native,
                )
                .map_err(SourceQueryObservationStop::Read)
        };
        let completed = result.map_err(|stop| self.record_control_refusal(stop))?;
        self.confirm_control(execution, native, completed)
            .map_err(|stop| self.record_control_refusal(stop))
    }
    /// Only the completed public read crosses into producer re-admission.
    pub(in crate::domain_computation::primary_graph) fn read_admitted_result<B>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        plan: SourceReadPlan<'_, Schema, B>,
        native: &mut ApplicationQueryNativeAccess<'_>,
    ) -> Result<
        WorthQueryApplicationOneShotResult<B::Query, SourceValue<Schema, B>>,
        SourceQueryObservationStop,
    >
    where
        B: ApplicationQueryBinding<Schema>,
        SourceValue<Schema, B>: WorthQueryApplicationProjection<Schema, B::Query>,
    {
        self.read_admitted::<B>(execution, plan, native)
            .map(|completed| completed.read)
    }
}
