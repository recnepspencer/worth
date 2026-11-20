//! Re-close pure outgoing discovery at the candidate actually leaving a wave.
use super::*;

impl<S: ApplicationSchema> SynchronousOutputWave<S> {
    pub(super) fn refresh_outgoing(
        &mut self,
        context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        candidate: &RelationalAuthoredCoupledCandidate<'_>,
        allowed: Option<&[String]>,
    ) -> Result<(), SynchronousOutputFailure> {
        // Only the wave's replaceable outgoing evidence is removed. Every
        // consumed Parent, incoming query and static selector stays strict.
        // Failure anywhere below stops this whole attempt before publication.
        context
            .owner
            .refresh_outgoing_support(context.execution)
            .map_err(CoupledSourceReadStop::from)?;
        let pin = candidate
            .pin_source(context.execution)
            .map_err(CoupledSourceReadStop::from)?;
        let mut next = Self::new();
        for task in &mut self.tasks {
            if task.alias.is_some() || !task.done || task.component.is_none() {
                continue;
            }
            checkpoint(context.execution, task.subject, 1)?;
            if let Some(parent) = &mut task.parent {
                parent.pin = pin.clone();
                context.owner.owe(&task.owed);
                task.tree
                    .discover_occurrences(context, candidate, parent, &mut next)?;
            }
        }
        for task in next.tasks {
            self.reserve_slot(context, task.subject)?;
            self.tasks.push(task);
        }
        // Refreshed component demands carry that component only, not another
        // copy of a root's static selector. Existing aliases still close by
        // their complete Native route/context; genuine new suppliers remain
        // pending for the ordinary ranked drive, never invoked by this pass.
        self.close_frontier(context, candidate, allowed)
    }
}
