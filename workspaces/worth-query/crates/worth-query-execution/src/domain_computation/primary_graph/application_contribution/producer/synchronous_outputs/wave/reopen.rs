//! A done task whose retained parent a later append made stale derives again
//! (PR C §1.3 `Done -> Superseded`). The reopened task reads its source with
//! no reuse, so normal admission derives it; readers that have not run yet
//! close again and wait on it. Its earlier discovery and prerequisite
//! evidence stays live until its re-derivation proves every child and
//! parent it selected before is selected again; only then does that evidence
//! end at the invalidator (R4). A dropped selection refuses in the wave
//! itself (`verify_rediscovered`).
use super::*;
use state::{map_remove, TaskState};

/// What a reopened row selected before, and the append that reopened it.
/// Children are compared by route: a child no occurrence component
/// coalesces is a new row on re-discovery, yet the same selection.
pub(super) struct ReopenCheck {
    invalidator: usize,
    prior_children: Vec<SelectedRoute>,
    prior_prerequisites: Vec<usize>,
}

/// One selected child: the connection that selected it and its route.
/// Distinct connections selecting one route are distinct selections.
type SelectedRoute = (&'static str, super::super::occurrences::OccurrenceKey);

impl<S: ApplicationSchema> SynchronousOutputWave<S> {
    /// Reopen every retention qualified use superseded since the last call.
    /// Returns whether any task was reopened.
    pub(super) fn reopen_stale(
        &mut self,
        context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        candidate: &RelationalAuthoredCoupledCandidate<'_>,
    ) -> Result<bool, SynchronousOutputFailure> {
        let reopens = context.owner.qualify();
        let reopened = !reopens.is_empty();
        for reopen in reopens {
            let subject = "retained parent reopen";
            checkpoint(
                context.execution,
                subject,
                self.by_read.len().max(1).ilog2() as u64 + 1,
            )?;
            let Some(&i) = self.by_read.get(&reopen.read.0) else {
                unreachable!("only a retention a task claimed is superseded")
            };
            let owed = self.tasks[i].owed.position().max(reopen.invalidator_owed);
            if !self.tasks[i].is_done() {
                // An earlier reopen of this batch already reopened it with
                // its parent (R3): it, and every consumer it deferred, also
                // owes this append's invalidation.
                self.raise_owed(i, owed);
                observation::record(|| format!("reopen task={i} owed={owed} (already reopened)"));
                continue;
            }
            observation::record(|| format!("reopen task={i} owed={owed}"));
            self.reopen_task(context, candidate, i, owed, reopen.invalidator)?;
        }
        Ok(reopened)
    }

    pub(super) fn reopen_task(
        &mut self,
        context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        candidate: &RelationalAuthoredCoupledCandidate<'_>,
        i: usize,
        owed: usize,
        invalidator: usize,
    ) -> Result<(), SynchronousOutputFailure> {
        let subject = self.tasks[i].subject;
        // `by_read` keeps the task's read: a later reopen of the same read in
        // this batch finds the task already reopened.
        if self.done_components.contains_key(&i) {
            map_remove(&mut self.done_components, &i, context.execution, subject)?;
        }
        let prior_prerequisites = self.canonical_set(i);
        let prior_children = {
            let children = std::mem::take(&mut self.tasks[i].children);
            self.routes(context, candidate, children)?
        };
        let task = &mut self.tasks[i];
        let previous = match std::mem::replace(&mut task.state, TaskState::Identified) {
            TaskState::Done { parent } => parent,
            _ => None,
        };
        task.reopen_check = Some(ReopenCheck {
            invalidator,
            prior_children,
            prior_prerequisites,
        });
        task.close_epoch = task.close_epoch.wrapping_add(1);
        task.prerequisites = Vec::new();
        task._prerequisite_memory = None;
        task.state = TaskState::Identified;
        task.reopened = true;
        task.owed = crate::domain_computation::primary_graph::application_contribution::coupled_publication::OwedDemand::at(owed);
        self.reopen_aliases(context, candidate, i, invalidator)?;
        // Readers that have not run yet close again and wait on it; readers
        // that already consumed it wait for its new derivation, then settle
        // to it or reopen with it at the same invalidator (R3).
        let readers = std::mem::take(&mut self.tasks[i].readers);
        let mut consumers = Vec::new();
        for &(reader, epoch) in &readers {
            checkpoint(context.execution, subject, 1)?;
            if self.tasks[reader].close_epoch != epoch {
                continue;
            }
            if !self.tasks[reader].is_done() {
                self.clear_input_capture(context, reader)?;
            } else if let Some(consumed) =
                self.tasks[reader].parent().and_then(|parent| parent.read)
            {
                let owed = self.tasks[reader].owed.position().max(owed);
                consumers.push((reader, consumed, owed));
            }
        }
        self.tasks[i].readers = readers;
        self.needed_dirty = true;
        self.defer_consumers(context, i, previous, consumers, invalidator)
    }

    /// Rows coalesced onto a reopened task discover their own static intent
    /// again once it completes again, checked like the task itself.
    fn reopen_aliases(
        &mut self,
        context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        candidate: &RelationalAuthoredCoupledCandidate<'_>,
        i: usize,
        invalidator: usize,
    ) -> Result<(), SynchronousOutputFailure> {
        let aliases = std::mem::take(&mut self.tasks[i].alias_rows);
        for &alias in &aliases {
            let subject = self.tasks[alias].subject;
            checkpoint(context.execution, subject, 1)?;
            if self.ready_alias_continuations.contains_key(&alias) {
                map_remove(
                    &mut self.ready_alias_continuations,
                    &alias,
                    context.execution,
                    subject,
                )?;
            }
            if !self.tasks[alias].tree.has_static_continuation() {
                continue;
            }
            let prior_children = {
                let children = std::mem::take(&mut self.tasks[alias].children);
                self.routes(context, candidate, children)?
            };
            let row = &mut self.tasks[alias];
            row.reopen_check = Some(ReopenCheck {
                invalidator,
                prior_children,
                prior_prerequisites: Vec::new(),
            });
            row.continuation_done = false;
            let prev = self.tasks[i].waiting_aliases_head.replace(alias);
            self.tasks[alias].next_waiting_alias = prev;
        }
        self.tasks[i].alias_rows = aliases;
        Ok(())
    }

    /// The routes of `rows`: an occurrence's key, otherwise its route read.
    fn routes(
        &self,
        context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        candidate: &RelationalAuthoredCoupledCandidate<'_>,
        rows: Vec<usize>,
    ) -> Result<Vec<SelectedRoute>, SynchronousOutputFailure> {
        let mut routes = Vec::new();
        let mut memory = None;
        for row in rows {
            let subject = self.tasks[row].subject;
            checkpoint(context.execution, subject, 1)?;
            let route = match self.tasks[row].key.clone() {
                Some(key) => key,
                None => self.tasks[row].tree.route(context, candidate)?,
            };
            state::push_funded(
                &mut routes,
                &mut memory,
                (subject, route),
                context.execution,
                subject,
            )?;
        }
        Ok(routes)
    }

    /// The canonical prerequisites of `i`, sorted.
    fn canonical_set(&self, i: usize) -> Vec<usize> {
        let mut set = self.tasks[i]
            .prerequisites
            .iter()
            .map(|parent| self.canonical(*parent))
            .collect::<Vec<_>>();
        set.sort_unstable();
        set.dedup();
        set
    }

    /// Reopened rows that finished discovering again, once their new children
    /// are identified: earlier evidence ends at the invalidator only if every
    /// child and parent selected before is selected again.
    pub(super) fn verify_rediscovered(
        &mut self,
        context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        candidate: &RelationalAuthoredCoupledCandidate<'_>,
    ) -> Result<(), SynchronousOutputFailure> {
        for row in std::mem::take(&mut self.rediscovered) {
            let Some(check) = self.tasks[row].reopen_check.take() else {
                continue;
            };
            let subject = self.tasks[row].subject;
            let children = self.routes(context, candidate, self.tasks[row].children.clone())?;
            let prerequisites = self.canonical_set(row);
            checkpoint(
                context.execution,
                subject,
                (check.prior_children.len() * children.len().max(1)
                    + check.prior_prerequisites.len()) as u64
                    + 1,
            )?;
            let kept_children = check
                .prior_children
                .iter()
                .all(|child| children.contains(child));
            let kept_parents = check
                .prior_prerequisites
                .iter()
                .all(|parent| prerequisites.binary_search(parent).is_ok());
            if !(kept_children && kept_parents) {
                // Final support cannot be the guard: the quiescent outgoing
                // refresh replaces outgoing evidence. Refuse here.
                observation::record(|| format!("dropped selection task={row}"));
                // A static-edge row has no occurrence key: its route names it.
                let key = match self.tasks[row].key.clone() {
                    Some(key) => key,
                    None => self.tasks[row].tree.route(context, candidate)?,
                };
                return Err(SynchronousOutputFailure(CoupledSourceReadStop::from(
                    crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial::new(
                        crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind::SupersededSelectionDropped {
                            source: key.scope,
                        },
                        format!("{subject}: a reopened derivation dropped a selection it derived"),
                    ),
                )));
            }
            context
                .owner
                .supersede_discovery(context.execution, row, check.invalidator)
                .map_err(CoupledSourceReadStop::from)?;
        }
        Ok(())
    }
}
