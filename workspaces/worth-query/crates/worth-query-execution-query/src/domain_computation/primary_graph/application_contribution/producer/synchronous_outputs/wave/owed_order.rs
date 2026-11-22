//! Owed-order gating for reopened derivations (PR C, R1; prototype). One
//! commit per stage runs every derivation an earlier stage owes before a
//! later one. The wave orders ready tasks by declared rank, so a reopened
//! task owed at `q` could derive before a task owed earlier has appended,
//! at an image one commit per stage never shows it. A ready reopened task
//! is therefore held while any undone task owed strictly before it remains,
//! and released once none does. Its blockers are never pulled forward.
//!
//! Progress: reopening a task raises every undone reader's owed position to
//! at least its own (`raise_readers_owed`), so a blocker never waits on the
//! task it holds, and the undone task with the least owed position is never
//! held. A drive that quiesces with held tasks (their blocker runs in a
//! later drive only) releases them, recorded, rather than blocking.
use super::*;
use state::{map_insert, map_remove, TaskState};

impl<S: ApplicationSchema> SynchronousOutputWave<S> {
    /// An undone task owed strictly before task `i`, if any: a task that is
    /// neither done, an alias, nor a pending refusal. Charged per task.
    fn earlier_owed_blocker(
        &self,
        context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        i: usize,
    ) -> Result<Option<usize>, SynchronousOutputFailure> {
        let owed = self.tasks[i].owed.position();
        checkpoint(context.execution, self.tasks[i].subject, self.tasks.len() as u64 + 1)?;
        Ok(self.tasks.iter().enumerate().position(|(j, task)| {
            j != i
                && task.owed.position() < owed
                && !task.is_done()
                && !matches!(task.state, TaskState::Alias { .. } | TaskState::Refused { .. })
        }))
    }

    /// Move ready reopened tasks with an earlier-owed blocker out of the
    /// ready queue, and back once none remains.
    pub(super) fn hold_reopened(
        &mut self,
        context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
    ) -> Result<(), SynchronousOutputFailure> {
        let held = self.held.iter().map(|(&i, ())| i).collect::<Vec<_>>();
        for i in held {
            if self.earlier_owed_blocker(context, i)?.is_none() {
                self.unhold(context, i)?;
            }
        }
        let ready = self
            .ready_queue
            .iter()
            .map(|(&(rank, i), ())| (rank, i))
            .filter(|&(_, i)| self.tasks[i].reopened)
            .collect::<Vec<_>>();
        for (rank, i) in ready {
            if let Some(blocker) = self.earlier_owed_blocker(context, i)? {
                let subject = self.tasks[i].subject;
                map_remove(&mut self.ready_queue, &(rank, i), context.execution, subject)?;
                map_insert(&mut self.held, &i, &(), context.execution, subject)?;
                observation::record(|| format!("hold task={i} blocker={blocker}"));
            }
        }
        Ok(())
    }

    /// Return a held task to the ready queue if it is still closed and ready.
    fn unhold(
        &mut self,
        context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        i: usize,
    ) -> Result<(), SynchronousOutputFailure> {
        let subject = self.tasks[i].subject;
        map_remove(&mut self.held, &i, context.execution, subject)?;
        if matches!(self.tasks[i].state, TaskState::Closed { waiting: 0, .. }) && self.tasks[i].needed {
            let key = (self.tasks[i].rank, i);
            map_insert(&mut self.ready_queue, &key, &(), context.execution, subject)?;
        }
        observation::record(|| format!("release task={i}"));
        Ok(())
    }

    /// At quiescence: release every held task (its blocker runs in a later
    /// drive only). Returns whether any was released.
    pub(super) fn release_held(
        &mut self,
        context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
    ) -> Result<bool, SynchronousOutputFailure> {
        let held = self.held.iter().map(|(&i, ())| i).collect::<Vec<_>>();
        for &i in &held {
            observation::record(|| format!("release held at quiescence task={i}"));
            self.unhold(context, i)?;
        }
        Ok(!held.is_empty() && !self.ready_queue.is_empty())
    }

    /// Task `i` was reopened owed at `owed`: every undone task that reads it,
    /// transitively, derives after it, so it owes at least as late (R1).
    pub(super) fn raise_readers_owed(
        &mut self,
        context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        i: usize,
        owed: usize,
    ) -> Result<(), SynchronousOutputFailure> {
        let mut stack = vec![i];
        while let Some(row) = stack.pop() {
            for &(reader, epoch) in &self.tasks[row].readers.clone() {
                checkpoint(context.execution, self.tasks[row].subject, 1)?;
                if self.tasks[reader].close_epoch != epoch || self.tasks[reader].is_done() {
                    continue;
                }
                if self.tasks[reader].owed.position() < owed {
                    self.raise_owed(reader, owed);
                    stack.push(reader);
                }
            }
        }
        Ok(())
    }

    /// The ready tasks of the lowest rank, in task order, once held tasks
    /// are out of the ready queue.
    pub(super) fn collect_rank_batch(
        &self,
        context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
    ) -> Result<
        Option<(usize, Vec<usize>, Option<ExecutionMemoryReservation>)>,
        SynchronousOutputFailure,
    > {
        let subject = "required occurrence frontier";
        checkpoint(
            context.execution,
            subject,
            self.ready_queue.len().max(1).ilog2() as u64 + 1,
        )?;
        let Some((&(rank, _), ())) = self.ready_queue.entry_at(0) else {
            return Ok(None);
        };
        let mut ready = Vec::new();
        let mut memory = None;
        for (&(r, i), ()) in self.ready_queue.iter() {
            if r != rank {
                break;
            }
            push_funded(&mut ready, &mut memory, i, context.execution, subject)?;
        }
        Ok(Some((rank, ready, memory)))
    }
}
