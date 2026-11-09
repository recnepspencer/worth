use worth_query_installation::facade::ApplicationSchema;

use super::{
    WorthQueryConditionalClockHandle, WorthQueryConditionalClockObservationDenial,
    WorthQueryConditionalClockObservationDenialKind, WorthQueryConditionalClockObservationPort,
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
        if runtime
            .conditional_cleanup
            .definitions
            .route(handle.binding_identity(), handle.lease())
            .is_some()
        {
            return Err(WorthQueryConditionalClockObservationDenial::new(
                WorthQueryConditionalClockObservationDenialKind::NotReadmitted {
                    family:
                        worth_foundational::facade::DurableStateFamily::BridgeManagedTemporalIntents,
                    phase: 7,
                },
                handle.binding_identity(),
            ));
        }
        let operation = runtime
            .conditional_cleanup
            .operations
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
        let bridge_root = runtime.runtime.bridge.conditional_operations();
        let bridge = bridge_root
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let memory = bridge.memory().map_err(|cause| WorthQueryConditionalClockObservationDenial::new(
            WorthQueryConditionalClockObservationDenialKind::ProductAdmission(
                super::super::installation::WorthQueryConditionalRuntimeInstallationDenialKind::BridgeConditional(Box::new(cause.kind()))), cause.detail()))?;
        operation
            .admit_product_binding(execution, memory, runtime, &truth)
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
            runtime,
            operation,
            truth,
            marker: std::marker::PhantomData,
        })
    }
}
