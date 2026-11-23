//! Re-close pure outgoing discovery at the candidate actually leaving a wave.
//! Touch-filtered (PR C §4): only completed rows whose outgoing support changed
//! since it was retained, or that hold none, run their selectors again. A row
//! skipped owns at least one compared entry, so the scan is charged there.
use super::*;

impl<S: ApplicationSchema> SynchronousOutputWave<S> {
    pub fn refresh_outgoing(
        &mut self,
        context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        candidate: &RelationalAuthoredCoupledCandidate<'_>,
        allowed: Option<&[String]>,
    ) -> Result<(), SynchronousOutputFailure> {
        // Only the wave's replaceable outgoing evidence is removed. Every
        // consumed Parent, incoming query and static selector stays strict.
        // Failure anywhere below stops this whole attempt before publication.
        let pin = candidate
            .pin_source(context.execution)
            .map_err(CoupledSourceReadStop::from)?;
        let kept = context
            .owner
            .refresh_outgoing_support(context.execution, &pin)
            .map_err(CoupledSourceReadStop::from)?;
        let mut next = Self::new();
        let mut selections = Vec::new();
        let mut revisited = 0;
        for (&i, ()) in self.done_components.iter() {
            if kept.binary_search(&i).is_ok() {
                continue;
            }
            revisited += 1;
            let owed = self.tasks[i].owed.clone();
            let task = &mut self.tasks[i];
            checkpoint(context.execution, task.subject, 1)?;
            if let state::TaskState::Done {
                parent: Some(parent),
            } = &mut task.state
            {
                parent.pin = pin.clone();
                context.owner.owe(&owed);
                context.owner.discover_for(Some(i));
                let start = next.tasks.len();
                let discovered = task
                    .tree
                    .discover_occurrences(context, candidate, parent, &mut next);
                context.owner.discover_for(None);
                discovered?;
                selections.push((i, start..next.tasks.len()));
            }
        }
        observation::record_outgoing_rows_revisited(revisited);
        // Refreshed selections join what each row selected, so a reopen
        // checks a child first linked at quiescence too.
        self.adopt_discovered(context, next, selections, Selection::Accumulate)?;
        // Refreshed component demands carry that component only, not another
        // copy of a root's static selector. Existing aliases still close by
        // their complete Native route/context; genuine new suppliers remain
        // pending for the ordinary ranked drive, never invoked by this pass.
        self.close_frontier(context, candidate, allowed)
    }
}
