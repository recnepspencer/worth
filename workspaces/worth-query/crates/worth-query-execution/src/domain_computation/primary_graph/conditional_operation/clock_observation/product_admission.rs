use worth_query_installation::facade::ApplicationSchema;

use {
    super::WorthQueryConditionalClockObservationDenial,
    super::WorthQueryConditionalClockObservationDenialKind,
    super::WorthQueryConditionalClockObservationPort,
    crate::domain_computation::primary_graph::vocabulary::conditional_operation::clock_handle::WorthQueryConditionalClockHandle,
};

impl<'runtime, Schema>
    crate::domain_computation::primary_graph::WorthQuerySelectedProductOperation<'runtime, Schema>
where
    Schema: ApplicationSchema,
{
    /// Admit the exact selected product before returning the observation port.
    /// This installs commit watches early enough for later World publications.
    pub fn conditional_clock<Node, Clock>(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        handle: &'runtime WorthQueryConditionalClockHandle<Schema, Node, Clock>,
    ) -> Result<
        WorthQueryConditionalClockObservationPort<'runtime, Schema, Node, Clock>,
        WorthQueryConditionalClockObservationDenial,
    > {
        let runtime = self.application();
        let operation = runtime
            .conditional_operations()
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .admit_clock(handle.binding_identity(), handle.lease())
            .ok_or_else(|| {
                WorthQueryConditionalClockObservationDenial::new(
                    WorthQueryConditionalClockObservationDenialKind::ForeignRuntime,
                    handle.binding_identity(),
                )
            })?;
        let truth =
            super::super::signal_decision_reentry::WorthQueryConditionalTruthBasis::from_selected(
                self,
            );
        let bridge_root = runtime.bridge().conditional_operations();
        let bridge = bridge_root
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        operation
            .admit_product_binding(execution, &bridge, runtime.application_runtime(), &truth)
            .map_err(|denial| {
                WorthQueryConditionalClockObservationDenial::new(
                    WorthQueryConditionalClockObservationDenialKind::ProductAdmission(
                        denial.kind(),
                    ),
                    denial.subject(),
                )
            })?;
        drop(bridge);
        Ok(WorthQueryConditionalClockObservationPort {
            runtime: runtime.application_runtime(),
            operation,
            truth,
            marker: std::marker::PhantomData,
        })
    }
}
