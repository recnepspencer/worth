//! A completed task wakes the aliases and dependents that wait on it.
use super::*;
use state::{map_insert, map_remove, TaskState};

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
}
