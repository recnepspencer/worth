//! Every prepared parent precedes discovery and preparation of the next wave.
use super::*;
use worth_execution::{ExecutionMemoryReservation, ExecutionOrderedMap};
use worth_relational::facade::identity::EntityId;
mod completion;
use completion::Selection;
mod continuations;
mod inputs;
mod observation;
pub(in crate::domain_computation::primary_graph) use observation::{
    charged as observe_scheduling_work, visited_retained_reads,
};
mod occurrences;
mod outgoing;
mod prerequisites;
mod reopen;
mod state;
use state::{capacity_denied, push_funded, Scheduled, TaskState};

/// Every wave charge goes through here, so its observed units are the
/// wave's whole share of the request's work accounting.
fn checkpoint(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    subject: &str,
    work: u64,
) -> Result<(), SynchronousOutputFailure> {
    observation::charged(work);
    super::checkpoint(execution, subject, work)
}

type OccurrenceBucketKey = (&'static str, EntityId, [u8; 32]);

pub struct SynchronousOutputWave<S: ApplicationSchema> {
    tasks: Vec<Scheduled<S>>,
    storage: Option<ExecutionMemoryReservation>,
    unidentified_from: usize,
    by_key: ExecutionOrderedMap<OccurrenceBucketKey, usize>,
    pending_close: ExecutionOrderedMap<usize, ()>,
    ready_queue: ExecutionOrderedMap<(usize, usize), ()>,
    captured_active: ExecutionOrderedMap<usize, ()>,
    ready_alias_continuations: ExecutionOrderedMap<usize, ()>,
    done_components: ExecutionOrderedMap<usize, ()>,
    /// The task that retained or derived each parent output read (PR C R2).
    by_read: ExecutionOrderedMap<usize, usize>,
    /// Reopened rows that discovered again, to verify once identified.
    rediscovered: Vec<usize>,
    needed_undone_count: usize,
    needed_dirty: bool,
    last_allowed: Option<Option<Vec<String>>>,
}

impl<S: ApplicationSchema> SynchronousOutputWave<S> {
    pub(in crate::domain_computation::primary_graph::application_contribution) fn new() -> Self {
        Self {
            tasks: Vec::new(),
            storage: None,
            unidentified_from: 0,
            by_key: ExecutionOrderedMap::default(),
            pending_close: ExecutionOrderedMap::default(),
            ready_queue: ExecutionOrderedMap::default(),
            captured_active: ExecutionOrderedMap::default(),
            ready_alias_continuations: ExecutionOrderedMap::default(),
            done_components: ExecutionOrderedMap::default(),
            by_read: ExecutionOrderedMap::default(),
            rediscovered: Vec::new(),
            needed_undone_count: 0,
            needed_dirty: true,
            last_allowed: None,
        }
    }

    pub(super) fn push(
        &mut self,
        context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        subject: &'static str,
        position: &worth_query_declaration::facade::application_program::ApplicationConnectionDeclaration,
        tree_bytes: usize,
        tree: impl FnOnce() -> Box<dyn capture::RequiredTree<S>>,
    ) -> Result<(), SynchronousOutputFailure> {
        checkpoint(context.execution, subject, 1)?;
        let bytes = u64::try_from(tree_bytes)
            .ok()
            .ok_or_else(|| capacity_denied(subject))?;
        let memory = context.execution.allocation_policy().reserve_memory(bytes)
            .map_err(|cause| SynchronousOutputFailure(CoupledSourceReadStop::capture(
                crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial::allocation_denied(subject, cause))))?;
        self.reserve_slot(context, subject)?;
        let work = context.order.lookup_work(position).ok_or_else(|| {
            SynchronousOutputFailure(
                super::super::WorthQueryOutputDemandDenial::new(
                    super::super::WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
                    subject,
                )
                .into(),
            )
        })?;
        checkpoint(context.execution, subject, work)?;
        let rank = context.order.rank(position).ok_or_else(|| {
            SynchronousOutputFailure(
                super::super::WorthQueryOutputDemandDenial::new(
                    super::super::WorthQueryOutputDemandDenialKind::ForeignDemand,
                    subject,
                )
                .into(),
            )
        })?;
        let tree = tree();
        let continuation_done = !tree.has_static_continuation();
        self.tasks.push(Scheduled {
            root: true,
            reopened: false,
            needed: false,
            key: None,
            next_bucket: None,
            component: None,
            state: TaskState::Pending,
            close_epoch: 0,
            prerequisite_queued: false,
            next_prerequisite: None,
            continuation_done,
            waiting_aliases_head: None,
            next_waiting_alias: None,
            prerequisites: Vec::new(),
            _prerequisite_memory: None,
            dependents: Vec::new(),
            _dependents_memory: None,
            readers: Vec::new(),
            _readers_memory: None,
            alias_rows: Vec::new(),
            _alias_rows_memory: None,
            children: Vec::new(),
            _children_memory: None,
            reopen_check: None,
            subject,
            rank,
            owed: context.owner.owing(),
            tree,
            _memory: memory,
        });
        Ok(())
    }

    /// The parent outputs this task's derivation consumes: its admitted strict
    /// prerequisite suppliers' actual outputs. Discovery that selected the
    /// task is not consumption evidence; it is retained as discovery support.
    fn consumed_reads(
        &self,
        i: usize,
    ) -> Vec<crate::domain_computation::primary_graph::provider::ParentRead> {
        let task = &self.tasks[i];
        task.prerequisites
            .iter()
            .filter_map(|supplier| {
                self.tasks[self.canonical(*supplier)]
                    .parent()
                    .and_then(|parent| parent.read)
            })
            .collect()
    }

    fn reserve_slot(
        &mut self,
        context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        subject: &'static str,
    ) -> Result<(), SynchronousOutputFailure> {
        if self.tasks.len() < self.tasks.capacity() {
            return Ok(());
        }
        let next = self
            .tasks
            .capacity()
            .checked_mul(2)
            .ok_or_else(|| capacity_denied(subject))?
            .max(1);
        let bytes = next
            .checked_mul(size_of::<Scheduled<S>>())
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or_else(|| capacity_denied(subject))?;
        let memory = context.execution.allocation_policy().reserve_memory(bytes)
            .map_err(|cause| SynchronousOutputFailure(CoupledSourceReadStop::capture(
                crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial::allocation_denied(subject, cause))))?;
        let mut replacement = Vec::new();
        replacement
            .try_reserve_exact(next)
            .map_err(|_| capacity_denied(subject))?;
        if replacement.capacity() != next {
            return Err(capacity_denied(subject));
        }
        checkpoint(context.execution, subject, bytes)?;
        replacement.append(&mut self.tasks);
        self.tasks = replacement;
        self.storage = memory;
        Ok(())
    }

    pub(in crate::domain_computation::primary_graph::application_contribution) fn drive(
        &mut self,
        context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        candidate: &mut RelationalAuthoredCoupledCandidate<'_>,
        allowed: Option<&[String]>,
    ) -> Result<(), SynchronousOutputFailure> {
        let mut batch_id = 0usize;
        loop {
            // R2: no task runs on a retained parent a prior append made stale.
            self.reopen_stale(context, candidate)?;
            self.close_frontier(context, candidate, allowed)?;
            if self.discover_alias_continuations(context, candidate)? {
                continue;
            }
            let Some((rank, ready, _ready_memory)) = self.collect_rank_batch(context)? else {
                if self.handle_quiescence(context, candidate, allowed)? {
                    continue;
                }
                break;
            };
            observation::record(|| format!("batch id={batch_id} rank={rank} ready={ready:?}"));
            batch_id += 1;
            let completed = self.run_batch(context, candidate, allowed, &ready)?;
            // Nothing discovers from a parent the batch's own appends staled.
            self.reopen_stale(context, candidate)?;
            self.discover_completed(context, candidate, ready, completed)?;
        }
        Ok(())
    }

    fn collect_rank_batch(
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

    fn handle_quiescence(
        &mut self,
        context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        candidate: &RelationalAuthoredCoupledCandidate<'_>,
        allowed: Option<&[String]>,
    ) -> Result<bool, SynchronousOutputFailure> {
        observation::record(|| "quiescence".to_owned());
        self.refresh_outgoing(context, candidate, allowed)?;
        if !self.ready_queue.is_empty() {
            return Ok(true);
        }
        if self.needed_undone_count > 0 {
            observation::record(|| "blocked".to_owned());
            return Err(self.blocked_frontier());
        }
        Ok(false)
    }

    fn run_batch(
        &mut self,
        context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        candidate: &mut RelationalAuthoredCoupledCandidate<'_>,
        allowed: Option<&[String]>,
        ready: &[usize],
    ) -> Result<usize, SynchronousOutputFailure> {
        let mut completed = 0;
        for &i in ready {
            // Earlier real suppliers can change a still-unconsumed child's
            // desired parents. Reclose before computing any orphaned row.
            if self.refresh_input_captures(context, candidate, allowed)? {
                observation::record(|| format!("break input-captures before={i}"));
                break;
            }
            if self.reopen_stale(context, candidate)? {
                observation::record(|| format!("break reopen before={i}"));
                break;
            }
            context.owner.consume(self.consumed_reads(i));
            let inputs = self.tasks[i].take_inputs();
            let subject = self.tasks[i].subject;
            let component = self.tasks[i].component;
            checkpoint(context.execution, subject, 1)?;
            context.owner.owe(&self.tasks[i].owed);
            context.owner.supersede(self.tasks[i].reopened);
            let prepared = self.tasks[i]
                .tree
                .prepare(context, candidate, inputs, component);
            context.owner.supersede(false);
            context.owner.consume(Vec::new());
            observation::record_prepare(i, &self.tasks[i], &prepared);
            match prepared? {
                PlannedOutput::Prepared(parent) => {
                    self.complete_task(context, i, parent)?;
                    completed += 1;
                }
                PlannedOutput::Recapture => {
                    observation::record(|| format!("break recapture at={i}"));
                    self.clear_input_capture(context, i)?;
                    break;
                }
            }
            self.retain_parent_inputs(context, candidate, i)?;
        }
        Ok(completed)
    }

    fn discover_completed(
        &mut self,
        context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        candidate: &RelationalAuthoredCoupledCandidate<'_>,
        ready: Vec<usize>,
        completed: usize,
    ) -> Result<(), SynchronousOutputFailure> {
        let pin = candidate
            .pin_source(context.execution)
            .map_err(CoupledSourceReadStop::from)?;
        let mut next = Self::new();
        let mut selections = Vec::new();
        for i in ready.into_iter().take(completed) {
            let owed = self.tasks[i].owed.clone();
            let task = &mut self.tasks[i];
            if let TaskState::Done {
                parent: Some(parent),
            } = &mut task.state
            {
                parent.pin = pin.clone();
                // A discovered child is owed by the stage that owed its parent.
                context.owner.owe(&owed);
                context.owner.discover_for(Some(i));
                let start = next.tasks.len();
                let discovered = task.tree.discover(context, candidate, parent, &mut next);
                context.owner.discover_for(None);
                discovered?;
                task.continuation_done = true;
                selections.push((i, start..next.tasks.len()));
            } else if matches!(task.state, TaskState::Done { parent: None })
                && task.reopen_check.is_some()
            {
                // A reopened row that derived again as Retiring selects
                // nothing: what it selected before must be checked too.
                selections.push((i, 0..0));
            }
        }
        self.adopt_discovered(context, next, selections, Selection::Replace)
    }
}
