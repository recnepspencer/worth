//! Close the finite typed occurrence frontier incrementally without quadratic quotes.
use super::*;
use prerequisites::Frontier;
use state::{map_insert, push_funded, TaskState};
use worth_execution::ExecutionOrderedMap;

pub(super) fn allowed_matches(family: &str, allowed: Option<&[String]>) -> bool {
    allowed.is_none_or(|families| families.iter().any(|f| f == family))
}

impl<S: ApplicationSchema> SynchronousOutputWave<S> {
    pub(super) fn blocked_frontier(&self) -> SynchronousOutputFailure {
        use super::super::super::{
            WorthQueryOutputDemandDenial as Denial, WorthQueryOutputDemandDenialKind as Kind,
        };
        for task in self.tasks.iter().filter(|t| t.needed && !t.is_done()) {
            for parent in &task.prerequisites {
                let parent = &self.tasks[self.canonical(*parent)];
                if matches!(parent.state, TaskState::Done { parent: None })
                    && let Some(key) = &parent.key
                {
                    return SynchronousOutputFailure(CoupledSourceReadStop::from(Denial::new(
                        Kind::RequiredOutputParentUnavailable { source: key.scope },
                        task.subject,
                    )));
                }
            }
        }
        if let Some(task) = self.tasks.iter().find(|t| t.needed && !t.is_done())
            && let Some(key) = &task.key
        {
            return SynchronousOutputFailure(CoupledSourceReadStop::from(Denial::new(
                Kind::CyclicRequiredOutputOccurrences { source: key.scope },
                task.subject,
            )));
        }
        super::super::occurrences::refusal("required outputs", "unavailable occurrence identity")
    }
    pub(super) fn canonical(&self, mut i: usize) -> usize {
        while let Some(parent) = self.tasks[i].canonical_parent() {
            i = parent;
        }
        i
    }
    pub(super) fn sync_allowed(
        &mut self,
        context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        allowed: Option<&[String]>,
    ) -> Result<(), SynchronousOutputFailure> {
        let unchanged = match (&self.last_allowed, allowed) {
            (Some(None), None) => true,
            (Some(Some(prev)), Some(curr)) => prev.as_slice() == curr,
            _ => false,
        };
        if unchanged {
            return Ok(());
        }
        checkpoint(
            context.execution,
            "required occurrence frontier",
            allowed.map_or(1, |families| families.len() as u64 + 1),
        )?;
        self.last_allowed = Some(allowed.map(|families| families.to_vec()));
        self.needed_dirty = true;
        Ok(())
    }

    pub(super) fn mark_needed(
        &mut self,
        context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        start: usize,
        mut frontier: Option<&mut Frontier>,
    ) -> Result<(), SynchronousOutputFailure> {
        let first = self.canonical(start);
        if self.tasks[first].needed || self.tasks[first].is_done() {
            return Ok(());
        }
        if !self.activate_needed(context, first, frontier.as_deref_mut())? {
            return Ok(());
        }
        let mut stack = Vec::new();
        let mut memory = None;
        let subject = self.tasks[first].subject;
        for parent in &self.tasks[first].prerequisites {
            let parent = self.canonical(*parent);
            if !self.tasks[parent].needed && !self.tasks[parent].is_done() {
                push_funded(&mut stack, &mut memory, parent, context.execution, subject)?;
            }
        }
        while let Some(curr) = stack.pop() {
            let curr = self.canonical(curr);
            if self.tasks[curr].needed || self.tasks[curr].is_done() {
                continue;
            }
            if !self.activate_needed(context, curr, frontier.as_deref_mut())? {
                continue;
            }
            let subject = self.tasks[curr].subject;
            for parent in &self.tasks[curr].prerequisites {
                let parent = self.canonical(*parent);
                if !self.tasks[parent].needed && !self.tasks[parent].is_done() {
                    push_funded(&mut stack, &mut memory, parent, context.execution, subject)?;
                }
            }
        }
        Ok(())
    }

    /// Marks one undone canonical task needed and updates its queue. Returns
    /// `true` only if it is already `Closed` with pending prerequisites to walk.
    fn activate_needed(
        &mut self,
        context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        i: usize,
        frontier: Option<&mut Frontier>,
    ) -> Result<bool, SynchronousOutputFailure> {
        let subject = self.tasks[i].subject;
        checkpoint(context.execution, subject, 1)?;
        self.tasks[i].needed = true;
        self.needed_undone_count += 1;
        match self.tasks[i].state {
            TaskState::Identified => {
                if let Some(frontier) = frontier {
                    self.enqueue_prerequisite(frontier, i);
                } else {
                    map_insert(&mut self.pending_close, &i, &(), context.execution, subject)?;
                }
                Ok(false)
            }
            TaskState::Closed { waiting: 0, .. } => {
                let key = (self.tasks[i].rank, i);
                map_insert(&mut self.ready_queue, &key, &(), context.execution, subject)?;
                Ok(false)
            }
            TaskState::Closed { .. } => Ok(true),
            TaskState::Pending
            | TaskState::Alias { .. }
            | TaskState::Done { .. }
            | TaskState::Refused { .. } => Ok(false),
        }
    }

    pub(super) fn recompute_needed(
        &mut self,
        context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        allowed: Option<&[String]>,
    ) -> Result<(), SynchronousOutputFailure> {
        let subject = "required occurrence frontier";
        checkpoint(context.execution, subject, self.tasks.len() as u64 + 1)?;
        self.pending_close = ExecutionOrderedMap::default();
        self.ready_queue = ExecutionOrderedMap::default();
        self.needed_undone_count = 0;
        for task in &mut self.tasks {
            task.needed = false;
        }
        // Only already-identified tasks exist in `0..unidentified_from`, matching
        // base when `refresh_input_captures` runs before identifying new tasks.
        for i in 0..self.unidentified_from {
            // A real root alias remains a seed even when coalescing marked its
            // row done: its authentic canonical supplier may still be pending.
            if (self.tasks[i].root || self.tasks[i].reopened)
                && allowed_matches(self.tasks[i].tree.family(), allowed)
            {
                self.mark_needed(context, i, None)?;
            }
        }
        self.needed_dirty = false;
        Ok(())
    }

    /// Coalesce every real placement, including an unused alias's static tail.
    pub fn identify(
        &mut self,
        context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        candidate: &RelationalAuthoredCoupledCandidate<'_>,
        i: usize,
        allowed: Option<&[String]>,
    ) -> Result<(), SynchronousOutputFailure> {
        if !matches!(self.tasks[i].state, TaskState::Pending) {
            return Ok(());
        }
        let subject = self.tasks[i].subject;
        checkpoint(context.execution, subject, 1)?;
        let occurrence = self.tasks[i].tree.occurrence(context, candidate)?;
        self.tasks[i].component = occurrence.as_ref().map(|(_, component)| *component);
        self.tasks[i].key = occurrence.map(|(key, _)| key);
        let prior = if let Some(key) = self.tasks[i].key.as_ref() {
            let bucket_key = (key.family, key.scope, key.partition);
            checkpoint(
                context.execution,
                subject,
                self.by_key.len().max(1).ilog2() as u64 + 1,
            )?;
            let mut cursor = self.by_key.get(&bucket_key).copied();
            let mut found = None;
            while let Some(earlier) = cursor {
                checkpoint(context.execution, subject, 1)?;
                if self.tasks[earlier].key.as_ref() == Some(key) {
                    found = Some(earlier);
                    break;
                }
                cursor = self.tasks[earlier].next_bucket;
            }
            found
        } else {
            None
        };
        if let Some(prior) = prior {
            self.bind_alias(context, i, prior, allowed)
        } else {
            self.bind_canonical(context, i, allowed)
        }
    }

    fn bind_alias(
        &mut self,
        context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        i: usize,
        prior: usize,
        allowed: Option<&[String]>,
    ) -> Result<(), SynchronousOutputFailure> {
        let subject = self.tasks[i].subject;
        if self.tasks[i].component != self.tasks[prior].component {
            return Err(super::super::occurrences::refusal(
                subject,
                "shared occurrence has different complete component contexts",
            ));
        }
        // The shared route is owed by its earliest actual demand.
        let (earlier, later) = self.tasks.split_at(i);
        earlier[prior].owed.coalesce(&later[0].owed);
        self.tasks[i].state = TaskState::Alias { canonical: prior };
        let canonical = &mut self.tasks[prior];
        push_funded(
            &mut canonical.alias_rows,
            &mut canonical._alias_rows_memory,
            i,
            context.execution,
            subject,
        )?;
        if !self.tasks[i].continuation_done {
            if matches!(self.tasks[prior].state, TaskState::Done { .. }) {
                map_insert(
                    &mut self.ready_alias_continuations,
                    &i,
                    &(),
                    context.execution,
                    subject,
                )?;
            } else {
                let prev = self.tasks[prior].waiting_aliases_head.replace(i);
                self.tasks[i].next_waiting_alias = prev;
            }
        }
        if !self.needed_dirty
            && self.tasks[i].root
            && allowed_matches(self.tasks[i].tree.family(), allowed)
        {
            self.mark_needed(context, prior, None)?;
        }
        Ok(())
    }

    fn bind_canonical(
        &mut self,
        context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        i: usize,
        allowed: Option<&[String]>,
    ) -> Result<(), SynchronousOutputFailure> {
        let subject = self.tasks[i].subject;
        self.tasks[i].state = TaskState::Identified;
        if let Some(key) = self.tasks[i].key.as_ref() {
            let bucket_key = (key.family, key.scope, key.partition);
            self.tasks[i].next_bucket = self.by_key.get(&bucket_key).copied();
            map_insert(
                &mut self.by_key,
                &bucket_key,
                &i,
                context.execution,
                subject,
            )?;
        }
        if !self.needed_dirty
            && self.tasks[i].root
            && allowed_matches(self.tasks[i].tree.family(), allowed)
        {
            self.mark_needed(context, i, None)?;
        }
        Ok(())
    }
}
