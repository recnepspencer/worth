//! Only a needed occurrence captures incoming support; every captured parent drains.
use super::*;

#[derive(Default)]
struct Frontier {
    first: Option<usize>,
    last: Option<usize>,
}
impl<S: ApplicationSchema> SynchronousOutputWave<S> {
    pub(super) fn close_frontier(
        &mut self,
        context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        candidate: &RelationalAuthoredCoupledCandidate<'_>,
        allowed: Option<&[String]>,
    ) -> Result<(), SynchronousOutputFailure> {
        // Identity remains eager: an alias of a supplied occurrence still owns
        // its distinct static selector even when no supplier is needed here.
        for i in 0..self.tasks.len() {
            self.identify(context, candidate, i)?;
        }
        let _planning_memory = self.quote_frontier(context, allowed.map_or(0, |v| v.len()))?;
        let mut frontier = Frontier::default();
        for (i, needed) in self.needed(allowed).into_iter().enumerate() {
            if needed {
                self.enqueue_prerequisite(&mut frontier, i);
            }
        }
        let pin = candidate
            .pin_source(context.execution)
            .map_err(CoupledSourceReadStop::from)?;
        while let Some(i) = self.pop_prerequisite(&mut frontier) {
            context.checkpoint(self.tasks[i].subject)?;
            self.tasks[i].closed = true;
            let Some(key) = &self.tasks[i].key else {
                continue;
            };
            let mut incoming = Self::new();
            self.tasks[i]
                .tree
                .prerequisites(context, &pin, key.scope, &mut incoming)?;
            self.append_prerequisites(context, candidate, i, incoming, &mut frontier)?;
        }
        Ok(())
    }
    fn append_prerequisites(
        &mut self,
        context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        candidate: &RelationalAuthoredCoupledCandidate<'_>,
        i: usize,
        incoming: Self,
        frontier: &mut Frontier,
    ) -> Result<(), SynchronousOutputFailure> {
        let count = incoming.tasks.len();
        if count == 0 {
            return Ok(());
        }
        let bytes = count
            .checked_mul(size_of::<usize>())
            .and_then(|v| u64::try_from(v).ok())
            .ok_or_else(|| {
                super::super::occurrences::refusal(self.tasks[i].subject, "prerequisite capacity")
            })?;
        let memory = context.execution.allocation_policy().reserve_memory(bytes).map_err(|cause| {
            SynchronousOutputFailure(CoupledSourceReadStop::capture(
                crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial::allocation_denied(
                    self.tasks[i].subject, cause)))
        })?;
        let mut parents = Vec::new();
        parents.try_reserve_exact(count).map_err(|_| {
            super::super::occurrences::refusal(self.tasks[i].subject, "prerequisite allocation")
        })?;
        for mut task in incoming.tasks {
            // A prerequisite is reached through this owner's current parent
            // set; it never becomes an independent final delivery root.
            task.root = false;
            self.reserve_slot(context, task.subject)?;
            let index = self.tasks.len();
            parents.push(index);
            self.tasks.push(task);
            self.identify(context, candidate, index)?;
            self.enqueue_prerequisite(frontier, index);
        }
        self.tasks[i].prerequisites = parents;
        self.tasks[i]._prerequisite_memory = memory;
        Ok(())
    }
    // The retained task storage already funds these links. Each canonical task
    // enters this worklist once, including parents aliased to an earlier row.
    fn enqueue_prerequisite(&mut self, frontier: &mut Frontier, i: usize) {
        let i = self.canonical(i);
        if self.tasks[i].closed || self.tasks[i].done || self.tasks[i].prerequisite_queued {
            return;
        }
        self.tasks[i].prerequisite_queued = true;
        if let Some(last) = frontier.last {
            self.tasks[last].next_prerequisite = Some(i);
        } else {
            frontier.first = Some(i);
        }
        frontier.last = Some(i);
    }
    fn pop_prerequisite(&mut self, frontier: &mut Frontier) -> Option<usize> {
        let i = frontier.first?;
        frontier.first = self.tasks[i].next_prerequisite.take();
        if frontier.first.is_none() {
            frontier.last = None;
        }
        self.tasks[i].prerequisite_queued = false;
        Some(i)
    }
}
