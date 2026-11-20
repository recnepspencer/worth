//! Seal provisional readers once nothing that does not wait on them can run (#254).
//!
//! A stored captured reader that would verify runs provisionally at its rank:
//! it discovers its outgoing routes in base order but is not done, so no route
//! consumes its read. The drive seals only when no other task is ready and
//! outgoing reclosure at the current candidate linked no new ready work. Every
//! route that does not wait on a provisional reader, including routes found by
//! that reclosure, has then settled, so such a writer has already written
//! whatever the reader reads.
//!
//! Sealing prepares the reader through the ordinary Ready path at the current
//! pin. A read that still verifies is retained there. A changed read returns
//! Recapture: the reader reopens at Source, captures with normal producer
//! admission, and derives as an ordinary task before any consumer runs.
//!
//! When routes wait, only the awaited readers of the earliest batch seal in a
//! round, so readers seal in the order base scheduling ran them: a writer
//! that waits on an earlier reader runs before a later reader re-reads.
//! Readers of one batch, such as same-rank siblings, seal together.
//!
//! Progress: sealing adds no edge and never waits on a reader's consumers.
//! Each empty-frontier pass either finds ready work through reclosure or seals
//! one batch's awaited readers into done tasks or ordinary captured tasks,
//! which never become provisional again at the same pin; the next empty pass
//! recloses the newly done readers' outgoing routes. Seal rounds are bounded
//! by the batches that left a provisional reader, so the drive ends.
//! A consumer that writes what a sealed reader read runs after the seal, so
//! the final support check refuses that edit as before, with no cycle.
use super::*;

impl<S: ApplicationSchema> SynchronousOutputWave<S> {
    /// Seal the earliest batch of provisional readers a waiting route needs,
    /// or all of them when nothing waits. All seals of one round share one
    /// pin: a retained read writes nothing and a recapture only clears its
    /// selection.
    pub(super) fn seal(
        &mut self,
        context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        candidate: &mut RelationalAuthoredCoupledCandidate<'_>,
        needed: &[bool],
    ) -> Result<bool, SynchronousOutputFailure> {
        if !self.tasks.iter().any(|task| task.provisional.is_some()) {
            return Ok(false);
        }
        // The drive's frontier quote already funds this scan of rows and edges.
        let mut awaited = Vec::new();
        awaited.try_reserve_exact(self.tasks.len()).map_err(|_| {
            super::super::occurrences::refusal("required outputs", "provisional reader allocation")
        })?;
        awaited.resize(self.tasks.len(), false);
        let mut first = None;
        for (i, task) in self.tasks.iter().enumerate() {
            if !needed[i] || task.done || task.provisional.is_some() {
                continue;
            }
            for parent in &task.prerequisites {
                let parent = self.canonical(*parent);
                if let Some(batch) = self.tasks[parent].provisional {
                    awaited[parent] = true;
                    first = Some(first.map_or(batch, |first: usize| first.min(batch)));
                }
            }
        }
        for (i, awaited) in awaited.into_iter().enumerate() {
            let Some(batch) = self.tasks[i].provisional else {
                continue;
            };
            if first.is_some_and(|first| !awaited || batch != first) {
                continue;
            }
            context.owner.consume(self.consumed_reads(i));
            let task = &mut self.tasks[i];
            context.checkpoint(task.subject)?;
            context.owner.owe(&task.owed);
            task.provisional = None;
            let prepared =
                task.tree
                    .prepare(context, candidate, task.inputs.take(), task.component);
            context.owner.consume(Vec::new());
            match prepared? {
                PlannedOutput::Prepared(parent) => {
                    task.parent = parent;
                    task.done = true;
                    self.retain_parent_inputs(context, candidate, i)?;
                }
                PlannedOutput::Recapture => {
                    task.parent = None;
                    task.clear_input_capture();
                }
            }
        }
        Ok(true)
    }
}
