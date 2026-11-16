//! Supplier aliases retain their own demand-dependent static discovery.
use super::*;

impl<S: ApplicationSchema> SynchronousOutputWave<S> {
    pub(super) fn discover_alias_continuations(
        &mut self,
        context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        candidate: &RelationalAuthoredCoupledCandidate<'_>,
    ) -> Result<bool, SynchronousOutputFailure> {
        if !self.tasks.iter().any(|task| {
            !task.continuation_done && task.alias.is_some_and(|prior| self.tasks[prior].done)
        }) {
            return Ok(false);
        }
        let mut next = Self::new();
        let pin = candidate
            .pin_source(context.execution)
            .map_err(CoupledSourceReadStop::from)?;
        for i in 0..self.tasks.len() {
            if self.tasks[i].alias.is_none() || self.tasks[i].continuation_done {
                continue;
            }
            let canonical = self.canonical(i);
            if !self.tasks[canonical].done {
                continue;
            }
            context.checkpoint(self.tasks[i].subject)?;
            // Aliases always point to an earlier retained row. Its authentic
            // Parent supplies data; this row supplies its own static intent.
            let (earlier, remaining) = self.tasks.split_at_mut(i);
            let task = &mut remaining[0];
            if let Some(parent) = &mut earlier[canonical].parent {
                parent.pin = pin.clone();
                task.tree.discover_static(context, parent, &mut next)?;
            }
            task.continuation_done = true;
        }
        let appended = !next.tasks.is_empty();
        for task in next.tasks {
            self.reserve_slot(context, task.subject)?;
            self.tasks.push(task);
        }
        Ok(appended)
    }
}
