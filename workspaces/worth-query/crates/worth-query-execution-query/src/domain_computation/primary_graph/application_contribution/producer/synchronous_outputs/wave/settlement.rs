//! R3 in the wave: a reopened task's done consumers wait for its new
//! derivation, then settle to it (they keep their consumption and do not run
//! again) or reopen with it at the same invalidator.
use super::*;
use crate::domain_computation::primary_graph::provider::ParentRead;
use worth_execution::ExecutionMemoryReservation;

/// What a reopened task owes its done consumers once it derives again.
pub(super) struct Settling {
    /// Its output before the reopen: what the consumers read.
    previous: Option<AppliedOutputParent>,
    /// The consumer rows, the parent read each consumed, and its owed position.
    consumers: Vec<(usize, ParentRead, usize)>,
    _memory: Option<ExecutionMemoryReservation>,
    invalidator: usize,
}

impl<S: ApplicationSchema> SynchronousOutputWave<S> {
    /// Defer `consumers` of reopened task `i` until it derives again. A task
    /// reopened again before then keeps the output its consumers read.
    pub(super) fn defer_consumers(
        &mut self,
        context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        i: usize,
        previous: Option<AppliedOutputParent>,
        consumers: Vec<(usize, ParentRead, usize)>,
        invalidator: usize,
    ) -> Result<(), SynchronousOutputFailure> {
        let subject = self.tasks[i].subject;
        let settling = self.tasks[i].settling.get_or_insert_with(|| Settling {
            previous,
            consumers: Vec::new(),
            _memory: None,
            invalidator,
        });
        for consumer in consumers {
            state::push_funded(
                &mut settling.consumers,
                &mut settling._memory,
                consumer,
                context.execution,
                subject,
            )?;
        }
        Ok(())
    }

    /// Reopened task `i` owes at least `owed`; so does each consumer it
    /// deferred, which would reopen at the same invalidator (R1 stays exact).
    pub(super) fn raise_owed(&mut self, i: usize, owed: usize) {
        if self.tasks[i].owed.position() < owed {
            self.tasks[i].owed = crate::domain_computation::primary_graph::application_contribution::coupled_publication::OwedDemand::at(owed);
        }
        if let Some(settling) = self.tasks[i].settling.as_mut() {
            for consumer in &mut settling.consumers {
                consumer.2 = consumer.2.max(owed);
            }
        }
    }

    /// Task `i` derived again: settle or reopen each consumer it deferred.
    /// Returns whether any consumer reopened.
    pub(super) fn settle_consumers(
        &mut self,
        context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        candidate: &mut RelationalAuthoredCoupledCandidate<'_>,
        i: usize,
    ) -> Result<bool, SynchronousOutputFailure> {
        let Some(settling) = self.tasks[i].settling.take() else {
            return Ok(false);
        };
        let mut reopened = false;
        for &(consumer, consumed, owed) in &settling.consumers {
            checkpoint(context.execution, self.tasks[i].subject, 1)?;
            if !self.tasks[consumer].is_done() {
                continue;
            }
            let settled = match (
                settling.previous.as_ref(),
                self.tasks[i].parent(),
                self.tasks[consumer].parent(),
            ) {
                (Some(previous), Some(replacement), Some(reader)) => context
                    .owner
                    .settle_consumer(context.execution, candidate, reader, previous, replacement)?,
                _ => false,
            };
            if settled {
                observation::record(|| format!("settle consumer task={consumer}"));
                continue;
            }
            context
                .owner
                .supersede_consumer(context.execution, consumed, settling.invalidator)?;
            observation::record(|| format!("reopen consumer task={consumer} owed={owed}"));
            self.reopen_task(context, candidate, consumer, owed, settling.invalidator)?;
            reopened = true;
        }
        Ok(reopened)
    }
}
