//! The sealed prerequisite roster carries actual Native publication consumption.
use super::*;

impl<S: ApplicationSchema> SynchronousOutputWave<S> {
    pub(super) fn retain_parent_inputs(
        &self,
        context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        candidate: &mut RelationalAuthoredCoupledCandidate<'_>,
        index: usize,
    ) -> Result<(), SynchronousOutputFailure> {
        let task = &self.tasks[index];
        let Some(consumer) = task
            .parent
            .as_ref()
            .and_then(|parent| parent.input.as_ref())
        else {
            return Ok(());
        };
        let capacity = task.prerequisites.len();
        let bytes = capacity
            .checked_mul(std::mem::size_of::<
                &worth_relational::facade::mvcc::RelationalRequiredOutputInput,
            >())
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or_else(|| {
                super::super::occurrences::refusal(task.subject, "required input capacity")
            })?;
        let _memory = context.execution.allocation_policy().reserve_memory(bytes).map_err(|cause| {
            SynchronousOutputFailure(CoupledSourceReadStop::capture(
                crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial::allocation_denied(task.subject, cause)))
        })?;
        let mut inputs = Vec::new();
        inputs.try_reserve_exact(capacity).map_err(|_| {
            super::super::occurrences::refusal(task.subject, "required input allocation")
        })?;
        for parent in &task.prerequisites {
            context.checkpoint(task.subject)?;
            let parent = &self.tasks[self.canonical(*parent)];
            if let Some(input) = parent
                .parent
                .as_ref()
                .and_then(|parent| parent.input.as_ref())
            {
                inputs.push(input);
            }
        }
        candidate
            .retain_output_inputs(context.execution, consumer, &inputs)
            .map_err(CoupledSourceReadStop::from)?;
        Ok(())
    }
}
