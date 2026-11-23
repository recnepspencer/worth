//! Pending domain refusals (PR C §1.7) on the wave.
use super::*;
use state::{map_insert, map_remove, TaskState};

impl<S: ApplicationSchema> SynchronousOutputWave<S> {
    /// Task `i` refused. A domain refusal with proofs waits: its proofs are
    /// retained and the task stays undone. Returns the failure otherwise.
    pub(super) fn defer_refusal(
        &mut self,
        context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        i: usize,
        failure: SynchronousOutputFailure,
    ) -> Result<Option<SynchronousOutputFailure>, SynchronousOutputFailure> {
        let proofs = match &failure.0 {
            CoupledSourceReadStop::Query(
                crate::domain_computation::primary_graph::application_contribution::producer::execution::ProducerExecutionStop::ExecutionStopped(denial),
            ) => {
                denial.refusal_proofs().cloned()
            }
            _ => None,
        };
        let Some(proofs) = proofs else {
            return Ok(Some(failure));
        };
        let subject = self.tasks[i].subject;
        let rank = self.tasks[i].rank;
        let read = context.owner.next_refusal_read();
        context
            .owner
            .register_refusal(context.execution, read, &proofs)
            .map_err(SynchronousOutputFailure)?;
        map_insert(&mut self.by_read, &read.0, &i, context.execution, subject)?;
        if self.ready_queue.contains_key(&(rank, i)) {
            map_remove(&mut self.ready_queue, &(rank, i), context.execution, subject)?;
        }
        if self.captured_active.contains_key(&i) {
            map_remove(&mut self.captured_active, &i, context.execution, subject)?;
        }
        self.tasks[i].state = TaskState::Refused {
            failure: Box::new(failure),
            read,
        };
        observation::record(|| format!("pending refusal task={i}"));
        Ok(None)
    }

    /// Whether task `i` waits on a pending refusal.
    pub(super) fn is_refused(&self, i: usize) -> bool {
        matches!(self.tasks[i].state, TaskState::Refused { .. })
    }

    /// The first pending refusal still needed, in stage order (owed position,
    /// then row), reported with its original stop. A refusal no needed task
    /// waits on any more (its consumer dropped it) is not this edit's cause.
    pub(super) fn take_pending_refusal(&mut self) -> Option<SynchronousOutputFailure> {
        let i = (0..self.tasks.len())
            .filter(|&i| self.tasks[i].needed && self.is_refused(i))
            .min_by_key(|&i| (self.tasks[i].owed.position(), i))?;
        match std::mem::replace(&mut self.tasks[i].state, TaskState::Identified) {
            TaskState::Refused { failure, .. } => Some(*failure),
            _ => unreachable!("matched a pending refusal"),
        }
    }

    /// Task `i` leaves a pending refusal (it derives again): the ledger entry
    /// that retained its proofs retires, so a later change to them reopens
    /// nothing.
    pub(super) fn leave_refusal(
        &self,
        context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        i: usize,
    ) {
        if let TaskState::Refused { read, .. } = &self.tasks[i].state {
            context.owner.retire_refusal(*read);
        }
    }
}
