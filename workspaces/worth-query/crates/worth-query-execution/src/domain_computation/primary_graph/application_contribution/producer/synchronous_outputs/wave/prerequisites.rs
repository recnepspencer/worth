//! Only a needed occurrence captures incoming support; every captured parent drains.
use super::*;
use state::{map_insert, map_remove, push_funded, TaskState};

#[derive(Default)]
pub(super) struct Frontier {
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
        self.sync_allowed(context, allowed)?;
        self.refresh_input_captures(context, candidate, allowed)?;
        // Identity remains eager: an alias of a supplied occurrence still owns
        // its distinct static selector even when no supplier is needed here.
        while self.unidentified_from < self.tasks.len() {
            let i = self.unidentified_from;
            self.unidentified_from += 1;
            self.identify(context, candidate, i, allowed)?;
        }
        if self.needed_dirty {
            self.recompute_needed(context, allowed)?;
        }
        if self.pending_close.is_empty() {
            return Ok(());
        }
        let pending = std::mem::take(&mut self.pending_close);
        let mut frontier = Frontier::default();
        for (&i, ()) in pending.iter() {
            self.enqueue_prerequisite(&mut frontier, i);
        }
        let pin = candidate
            .pin_source(context.execution)
            .map_err(CoupledSourceReadStop::from)?;
        while let Some(i) = self.pop_prerequisite(&mut frontier) {
            if !matches!(self.tasks[i].state, TaskState::Identified) {
                continue;
            }
            checkpoint(context.execution, self.tasks[i].subject, 1)?;
            let Some(source) = self.tasks[i].key.as_ref().map(|key| key.scope) else {
                self.finish_close(context, i, None, &[])?;
                continue;
            };
            context.owner.supersede(self.tasks[i].reopened);
            let inputs =
                self.tasks[i]
                    .tree
                    .capture_inputs(context, candidate, self.tasks[i].component);
            context.owner.supersede(false);
            let inputs = inputs?;
            let mut incoming = Self::new();
            // Its suppliers are owed by the demand that owes this task.
            context.owner.owe(&self.tasks[i].owed);
            context.owner.discover_for(Some(i));
            let closed = self.tasks[i].tree.prerequisites(
                context,
                candidate,
                &pin,
                source,
                &PrerequisiteInputs::from(inputs.as_ref()),
                &mut incoming,
            );
            context.owner.discover_for(None);
            closed?;
            if let Some(PreparedCoupledInputs::Captured(captured)) = &inputs {
                let drained = incoming
                    .tasks
                    .iter()
                    .filter(|task| task.tree.expected_source().is_some())
                    .count();
                if drained != captured.parents().len() {
                    return Err(super::super::occurrences::refusal(
                        self.tasks[i].subject,
                        "incomplete captured parent roster",
                    ));
                }
            }
            self.append_prerequisites(
                context,
                candidate,
                i,
                inputs,
                incoming,
                &mut frontier,
                allowed,
            )?;
        }
        Ok(())
    }

    pub(super) fn refresh_input_captures(
        &mut self,
        context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        candidate: &RelationalAuthoredCoupledCandidate<'_>,
        allowed: Option<&[String]>,
    ) -> Result<bool, SynchronousOutputFailure> {
        if self.captured_active.is_empty() {
            return Ok(false);
        }
        if self.needed_dirty {
            self.recompute_needed(context, allowed)?;
        }
        let pin = candidate
            .pin_source(context.execution)
            .map_err(CoupledSourceReadStop::from)?;
        let mut changed = false;
        let mut cursor = 0usize;
        while let Some((&index, ())) = self.captured_active.first_at_or_after(&cursor) {
            cursor = index + 1;
            checkpoint(context.execution, self.tasks[index].subject, 1)?;
            if !self.tasks[index].needed || self.tasks[index].is_done() {
                continue;
            }
            let equal = match self.tasks[index].inputs() {
                Some(inputs) => inputs.support_equal(context.execution, &pin)?,
                None => true,
            };
            if !equal {
                self.clear_input_capture(context, index)?;
                changed = true;
            }
        }
        Ok(changed)
    }

    pub(super) fn clear_input_capture(
        &mut self,
        context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        i: usize,
    ) -> Result<(), SynchronousOutputFailure> {
        let subject = self.tasks[i].subject;
        let rank = self.tasks[i].rank;
        if self.captured_active.contains_key(&i) {
            map_remove(&mut self.captured_active, &i, context.execution, subject)?;
        }
        if self.ready_queue.contains_key(&(rank, i)) {
            map_remove(
                &mut self.ready_queue,
                &(rank, i),
                context.execution,
                subject,
            )?;
        }
        let task = &mut self.tasks[i];
        task.close_epoch = task.close_epoch.wrapping_add(1);
        task.prerequisites = Vec::new();
        task._prerequisite_memory = None;
        task.state = TaskState::Identified;
        self.needed_dirty = true;
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn append_prerequisites(
        &mut self,
        context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        candidate: &RelationalAuthoredCoupledCandidate<'_>,
        i: usize,
        inputs: Option<PreparedCoupledInputs>,
        incoming: Self,
        frontier: &mut Frontier,
        allowed: Option<&[String]>,
    ) -> Result<(), SynchronousOutputFailure> {
        let count = incoming.tasks.len();
        if count == 0 {
            return self.finish_close(context, i, inputs, &[]);
        }
        let subject = self.tasks[i].subject;
        let bytes = count
            .checked_mul(size_of::<usize>())
            .and_then(|v| u64::try_from(v).ok())
            .ok_or_else(|| super::super::occurrences::refusal(subject, "prerequisite capacity"))?;
        let memory = context.execution.allocation_policy().reserve_memory(bytes).map_err(|cause| {
            SynchronousOutputFailure(CoupledSourceReadStop::capture(
                crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial::allocation_denied(
                    subject, cause)))
        })?;
        let mut parents = Vec::new();
        parents
            .try_reserve_exact(count)
            .map_err(|_| super::super::occurrences::refusal(subject, "prerequisite allocation"))?;
        let _canonical_memory = context.execution.allocation_policy().reserve_memory(bytes).map_err(|cause| {
            SynchronousOutputFailure(CoupledSourceReadStop::capture(
                crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial::allocation_denied(
                    subject, cause)))
        })?;
        let mut canonical_parents = Vec::new();
        canonical_parents
            .try_reserve_exact(count)
            .map_err(|_| super::super::occurrences::refusal(subject, "prerequisite allocation"))?;
        for mut task in incoming.tasks {
            // A prerequisite is reached through this owner's current parent
            // set; it never becomes an independent final delivery root.
            task.root = false;
            self.reserve_slot(context, task.subject)?;
            let index = self.tasks.len();
            parents.push(index);
            self.tasks.push(task);
            self.unidentified_from = self.tasks.len();
            self.identify(context, candidate, index, allowed)?;
            let canonical = self.canonical(index);
            canonical_parents.push(canonical);
            self.mark_needed(context, canonical, Some(frontier))?;
            if canonical != i {
                self.enqueue_prerequisite(frontier, canonical);
            }
        }
        self.tasks[i].prerequisites = parents;
        self.tasks[i]._prerequisite_memory = memory;
        canonical_parents.sort_unstable();
        canonical_parents.dedup();
        self.finish_close(context, i, inputs, &canonical_parents)
    }

    fn finish_close(
        &mut self,
        context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        i: usize,
        inputs: Option<PreparedCoupledInputs>,
        unique_parents: &[usize],
    ) -> Result<(), SynchronousOutputFailure> {
        let subject = self.tasks[i].subject;
        let epoch = self.tasks[i].close_epoch;
        let mut waiting = 0usize;
        for &parent in unique_parents {
            if parent != i {
                let parent_task = &mut self.tasks[parent];
                push_funded(
                    &mut parent_task.readers,
                    &mut parent_task._readers_memory,
                    (i, epoch),
                    context.execution,
                    subject,
                )?;
            }
            match self.tasks[parent].state {
                TaskState::Done { parent: Some(_) } => {}
                TaskState::Done { parent: None } => {
                    waiting += 1;
                }
                TaskState::Identified | TaskState::Closed { .. } => {
                    waiting += 1;
                    if parent != i {
                        let parent_task = &mut self.tasks[parent];
                        push_funded(
                            &mut parent_task.dependents,
                            &mut parent_task._dependents_memory,
                            (i, epoch),
                            context.execution,
                            subject,
                        )?;
                    }
                }
                TaskState::Pending | TaskState::Alias { .. } => {
                    unreachable!("every prerequisite is identified before finish_close")
                }
            }
        }
        let is_captured = matches!(inputs, Some(PreparedCoupledInputs::Captured(_)));
        self.tasks[i].state = TaskState::Closed { inputs, waiting };
        if is_captured {
            map_insert(
                &mut self.captured_active,
                &i,
                &(),
                context.execution,
                subject,
            )?;
        }
        if waiting == 0 && self.tasks[i].needed {
            let key = (self.tasks[i].rank, i);
            map_insert(&mut self.ready_queue, &key, &(), context.execution, subject)?;
        }
        Ok(())
    }

    // The retained task storage already funds these links. Each canonical task
    // enters this worklist once, including parents aliased to an earlier row.
    pub(super) fn enqueue_prerequisite(&mut self, frontier: &mut Frontier, i: usize) {
        let i = self.canonical(i);
        if !matches!(self.tasks[i].state, TaskState::Identified)
            || self.tasks[i].prerequisite_queued
        {
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
