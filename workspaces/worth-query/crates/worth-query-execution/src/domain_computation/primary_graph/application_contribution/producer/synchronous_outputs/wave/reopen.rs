//! A done task whose retained parent a later append made stale derives again
//! (PR C §1.3 `Done -> Superseded`). The reopened task reads its source with
//! no reuse, so normal admission derives it. Its own discovery evidence is
//! superseded at the same invalidator and discovered again when it completes;
//! readers that have not run yet close again and wait on the new derivation.
use super::*;
use state::{map_remove, TaskState};

impl<S: ApplicationSchema> SynchronousOutputWave<S> {
    /// Qualify every suspect retained derivation and reopen the stale ones.
    /// Returns whether any task was reopened.
    pub(super) fn reopen_stale(
        &mut self,
        context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        candidate: &RelationalAuthoredCoupledCandidate<'_>,
    ) -> Result<bool, SynchronousOutputFailure> {
        let by_read = &self.by_read;
        let reopens = context
            .owner
            .qualify(context.execution, candidate, |read| {
                by_read.contains_key(&read.0)
            })?;
        let mut reopened = false;
        for reopen in reopens {
            let subject = "retained parent reopen";
            checkpoint(
                context.execution,
                subject,
                self.by_read.len().max(1).ilog2() as u64 + 1,
            )?;
            let Some(&i) = self.by_read.get(&reopen.read.0) else {
                unreachable!("qualify supersedes only a retention a task owns")
            };
            observation::record(|| format!("reopen task={i} owed={}", reopen.owed));
            self.reopen_task(context, i, reopen.read, reopen.owed, reopen.invalidator)?;
            reopened = true;
        }
        Ok(reopened)
    }

    fn reopen_task(
        &mut self,
        context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        i: usize,
        read: crate::domain_computation::primary_graph::provider::ParentRead,
        owed: usize,
        invalidator: usize,
    ) -> Result<(), SynchronousOutputFailure> {
        let subject = self.tasks[i].subject;
        map_remove(&mut self.by_read, &read.0, context.execution, subject)?;
        if self.done_components.contains_key(&i) {
            map_remove(&mut self.done_components, &i, context.execution, subject)?;
        }
        context
            .owner
            .supersede_discovery(context.execution, i, invalidator)
            .map_err(CoupledSourceReadStop::from)?;
        let task = &mut self.tasks[i];
        task.close_epoch = task.close_epoch.wrapping_add(1);
        task.prerequisites = Vec::new();
        task._prerequisite_memory = None;
        task.state = TaskState::Identified;
        task.reopened = true;
        task.owed = crate::domain_computation::primary_graph::application_contribution::coupled_publication::OwedDemand::at(owed);
        // Rows coalesced onto it discover their static intent again once it
        // completes again.
        let aliases = std::mem::take(&mut self.tasks[i].alias_rows);
        for &alias in &aliases {
            checkpoint(context.execution, subject, 1)?;
            self.tasks[alias].continuation_done = false;
            let prev = self.tasks[i].waiting_aliases_head.replace(alias);
            self.tasks[alias].next_waiting_alias = prev;
        }
        self.tasks[i].alias_rows = aliases;
        // Readers that have not run yet close again and wait on it.
        let readers = std::mem::take(&mut self.tasks[i].readers);
        for &(reader, epoch) in &readers {
            checkpoint(context.execution, subject, 1)?;
            if self.tasks[reader].close_epoch == epoch && !self.tasks[reader].is_done() {
                self.clear_input_capture(context, reader)?;
            }
        }
        self.tasks[i].readers = readers;
        self.needed_dirty = true;
        Ok(())
    }
}
