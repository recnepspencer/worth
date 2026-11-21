//! Supplier aliases retain their own demand-dependent static discovery.
use super::*;

impl<S: ApplicationSchema> SynchronousOutputWave<S> {
    pub(super) fn discover_alias_continuations(
        &mut self,
        context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        candidate: &RelationalAuthoredCoupledCandidate<'_>,
    ) -> Result<bool, SynchronousOutputFailure> {
        if self.ready_alias_continuations.is_empty() {
            return Ok(false);
        }
        let aliases = std::mem::take(&mut self.ready_alias_continuations);
        let mut next = Self::new();
        let mut selections = Vec::new();
        let pin = candidate
            .pin_source(context.execution)
            .map_err(CoupledSourceReadStop::from)?;
        for (&i, ()) in aliases.iter() {
            if self.tasks[i].continuation_done {
                continue;
            }
            let canonical = self.canonical(i);
            checkpoint(context.execution, self.tasks[i].subject, 1)?;
            // Aliases always point to an earlier retained row. Its authentic
            // Parent supplies data; this row supplies its own static intent.
            let (earlier, remaining) = self.tasks.split_at_mut(i);
            let task = &mut remaining[0];
            // A canonical that is not done (reopened) runs it after it is.
            if let Some(parent) = earlier[canonical].parent_mut() {
                parent.pin = pin.clone();
                // This row's own static intent is owed by its own demand.
                context.owner.owe(&task.owed);
                context.owner.discover_for(Some(i));
                let start = next.tasks.len();
                let discovered = task
                    .tree
                    .discover_static(context, candidate, parent, &mut next);
                context.owner.discover_for(None);
                discovered?;
                task.continuation_done = true;
                selections.push((i, start..next.tasks.len()));
            }
        }
        let appended = !next.tasks.is_empty();
        self.adopt_discovered(context, next, selections)?;
        Ok(appended)
    }
}
