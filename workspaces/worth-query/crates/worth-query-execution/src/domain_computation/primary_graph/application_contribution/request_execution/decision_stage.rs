//! Native decision work borrows the existing request lease, never a new request.
use super::super::execution_denial::lease_denial;
use super::advancement::{advancement_denial, WorthQueryAdvancementDenial};
use super::{ExecutionWorkCeiling, Form, MapKernelContext, QueryRequestExecution};

impl QueryRequestExecution<'_> {
    pub(in crate::domain_computation::primary_graph) fn decision_stage<R>(
        &self,
        stage: impl FnOnce(&mut MapKernelContext<'_, '_>) -> R,
    ) -> Result<R, WorthQueryAdvancementDenial> {
        let lease = match &self.form {
            Form::Leased { lease, .. } | Form::Serial { lease, .. } => lease
                .as_ref()
                .map_err(|cause| WorthQueryAdvancementDenial::Resource(lease_denial(*cause)))?,
        };
        let execute = |context: &mut MapKernelContext<'_, '_>| {
            let value = stage(context);
            context.poll()?;
            Ok(value)
        };
        let outcome = if self.has_accounting() {
            ExecutionWorkCeiling::new(self.declared_work)
                .run_with_checkpoint(Some(lease), None, execute)
                .map(|(value, _report)| value)
        } else {
            worth_execution::ExecutionRequest::leased(lease)
                .run_native_with_checkpoint(execute)
                .map(|(value, _physical)| value)
        };
        outcome.map_err(advancement_denial).and_then(|value| {
            value.map_err(|cause| {
                advancement_denial(worth_execution::WorkCeilingDenial::Stopped(cause))
            })
        })
    }
}
