//! A completed task wakes the aliases and dependents that wait on it.
use super::*;
use state::{map_insert, map_remove, push_funded, TaskState};

/// Whether a discovery pass replaces a row's recorded selection (its own
/// discovery) or adds to it (a quiescent outgoing refresh).
#[derive(Clone, Copy, Eq, PartialEq)]
pub(super) enum Selection {
    Replace,
    Accumulate,
}

impl<S: ApplicationSchema> SynchronousOutputWave<S> {
    pub(super) fn complete_task(
        &mut self,
        context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        i: usize,
        parent: Option<AppliedOutputParent>,
    ) -> Result<(), SynchronousOutputFailure> {
        let subject = self.tasks[i].subject;
        let rank = self.tasks[i].rank;
        let component = self.tasks[i].component;
        let has_parent = parent.is_some();
        if let Some(read) = parent.as_ref().and_then(|parent| parent.read) {
            map_insert(&mut self.by_read, &read.0, &i, context.execution, subject)?;
            context.owner.claim_retention(context.execution, read)?;
        }
        map_remove(
            &mut self.ready_queue,
            &(rank, i),
            context.execution,
            subject,
        )?;
        if self.captured_active.contains_key(&i) {
            map_remove(&mut self.captured_active, &i, context.execution, subject)?;
        }
        if self.tasks[i].needed {
            self.needed_undone_count -= 1;
        }
        self.tasks[i].state = TaskState::Done { parent };
        self.tasks[i].reopened = false;
        if component.is_some() {
            map_insert(
                &mut self.done_components,
                &i,
                &(),
                context.execution,
                subject,
            )?;
        }
        let mut alias = self.tasks[i].waiting_aliases_head.take();
        while let Some(a) = alias {
            alias = self.tasks[a].next_waiting_alias.take();
            map_insert(
                &mut self.ready_alias_continuations,
                &a,
                &(),
                context.execution,
                self.tasks[a].subject,
            )?;
        }
        let dependents = std::mem::take(&mut self.tasks[i].dependents);
        self.tasks[i]._dependents_memory = None;
        if has_parent {
            for (child, epoch) in dependents {
                checkpoint(context.execution, self.tasks[child].subject, 1)?;
                if self.tasks[child].close_epoch != epoch {
                    continue;
                }
                if let TaskState::Closed { waiting, .. } = &mut self.tasks[child].state {
                    *waiting -= 1;
                    if *waiting == 0 && self.tasks[child].needed {
                        let key = (self.tasks[child].rank, child);
                        map_insert(
                            &mut self.ready_queue,
                            &key,
                            &(),
                            context.execution,
                            self.tasks[child].subject,
                        )?;
                    }
                }
            }
        }
        Ok(())
    }

    /// Append the rows a discovery pass selected and record, per selecting
    /// row, which they are; a reopened row is then verified once identified.
    pub(super) fn adopt_discovered(
        &mut self,
        context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        next: Self,
        selections: Vec<(usize, std::ops::Range<usize>)>,
        selection: Selection,
    ) -> Result<(), SynchronousOutputFailure> {
        let base = self.tasks.len();
        for task in next.tasks {
            self.reserve_slot(context, task.subject)?;
            self.tasks.push(task);
        }
        for (row, range) in selections {
            let subject = self.tasks[row].subject;
            checkpoint(context.execution, subject, range.len() as u64 + 1)?;
            let task = &mut self.tasks[row];
            if selection == Selection::Replace {
                task.children = Vec::new();
                task._children_memory = None;
            }
            for child in range {
                push_funded(
                    &mut task.children,
                    &mut task._children_memory,
                    base + child,
                    context.execution,
                    subject,
                )?;
            }
            if selection == Selection::Replace && task.reopen_check.is_some() {
                self.rediscovered.push(row);
            }
        }
        Ok(())
    }
}
