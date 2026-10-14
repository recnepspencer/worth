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
        ExecutionWorkCeiling::new(self.declared_work)
            .run_with_checkpoint(Some(lease), None, |context| {
                let value = stage(context);
                context.checkpoint(0)?;
                Ok(value)
            })
            .map_err(advancement_denial)
            .and_then(|(value, _report)| {
                value.map_err(|cause| {
                    advancement_denial(worth_execution::WorkCeilingDenial::Stopped(cause))
                })
            })
    }
}
