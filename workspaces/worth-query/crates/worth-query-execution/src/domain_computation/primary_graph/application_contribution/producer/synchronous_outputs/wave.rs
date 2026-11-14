//! Every prepared parent precedes discovery and preparation of the next wave.
use super::*;
mod continuations;
mod occurrences;
struct Scheduled<S: ApplicationSchema> {
    key: Option<super::occurrences::OccurrenceKey>,
    component: Option<std::any::TypeId>,
    closed: bool,
    done: bool,
    alias: Option<usize>,
    continuation_done: bool,
    prerequisites: Vec<usize>,
    _prerequisite_memory: Option<worth_execution::ExecutionMemoryReservation>,
    subject: &'static str,
    rank: usize,
    tree: Box<dyn capture::RequiredTree<S>>,
    parent: Option<AppliedOutputParent>,
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
}
pub struct SynchronousOutputWave<S: ApplicationSchema> {
    tasks: Vec<Scheduled<S>>,
    storage: Option<worth_execution::ExecutionMemoryReservation>,
}
impl<S: ApplicationSchema> SynchronousOutputWave<S> {
    pub(in crate::domain_computation::primary_graph::application_contribution) fn new() -> Self {
        Self {
            tasks: Vec::new(),
            storage: None,
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
        context.checkpoint(subject)?;
        let capacity = || {
            SynchronousOutputFailure(
                super::super::WorthQueryOutputDemandDenial::new(
                    super::super::WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded,
                    subject,
                )
                .into(),
            )
        };
        let bytes = u64::try_from(tree_bytes).ok().ok_or_else(capacity)?;
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
            key: None,
            component: None,
            closed: false,
            done: false,
            alias: None,
            continuation_done,
            prerequisites: Vec::new(),
            _prerequisite_memory: None,
            subject,
            rank,
            tree,
            parent: None,
            _memory: memory,
        });
        Ok(())
    }
    fn reserve_slot(
        &mut self,
        context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        subject: &'static str,
    ) -> Result<(), SynchronousOutputFailure> {
        if self.tasks.len() < self.tasks.capacity() {
            return Ok(());
        }
        let capacity = || {
            SynchronousOutputFailure(
                super::super::WorthQueryOutputDemandDenial::new(
                    super::super::WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded,
                    subject,
                )
                .into(),
            )
        };
        let next = self
            .tasks
            .capacity()
            .checked_mul(2)
            .ok_or_else(capacity)?
            .max(1);
        let bytes = next
            .checked_mul(std::mem::size_of::<Scheduled<S>>())
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or_else(capacity)?;
        let memory = context.execution.allocation_policy().reserve_memory(bytes)
            .map_err(|cause| SynchronousOutputFailure(CoupledSourceReadStop::capture(
                crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial::allocation_denied(subject, cause))))?;
        let mut replacement = Vec::new();
        replacement
            .try_reserve_exact(next)
            .map_err(|_| capacity())?;
        if replacement.capacity() != next {
            return Err(capacity());
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
        loop {
            self.close_frontier(context, candidate)?;
            if self.discover_alias_continuations(context, candidate)? {
                continue;
            }
            let _planning_memory = self.quote_frontier(context, allowed.map_or(0, |v| v.len()))?;
            let needed = self.needed(allowed);
            let rank = self
                .tasks
                .iter()
                .enumerate()
                .filter(|(i, t)| needed[*i] && !t.done && self.ready(*i))
                .map(|(_, t)| t.rank)
                .min();
            let Some(rank) = rank else {
                if self
                    .tasks
                    .iter()
                    .enumerate()
                    .any(|(i, t)| needed[i] && !t.done)
                {
                    return Err(self.blocked_frontier(&needed));
                }
                break;
            };
            let mut ready = Vec::new();
            ready.try_reserve_exact(self.tasks.len()).map_err(|_| {
                SynchronousOutputFailure(CoupledSourceReadStop::from(
                    super::super::WorthQueryOutputDemandDenial::new(
                        super::super::WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded,
                        "required occurrence frontier",
                    ),
                ))
            })?;
            for (i, t) in self.tasks.iter().enumerate() {
                if needed[i] && !t.done && t.rank == rank && self.ready(i) {
                    ready.push(i);
                }
            }
            for i in &ready {
                let task = &mut self.tasks[*i];
                context.checkpoint(task.subject)?;
                task.parent = task.tree.prepare(context, candidate)?;
                task.done = true;
            }
            let pin = candidate
                .pin_source(context.execution)
                .map_err(CoupledSourceReadStop::from)?;
            let mut next = Self::new();
            for i in ready {
                let task = &mut self.tasks[i];
                if let Some(parent) = &mut task.parent {
                    parent.pin = pin.clone();
                    task.tree.discover(context, parent, &mut next)?;
                }
                task.continuation_done = true;
            }
            for task in next.tasks {
                self.reserve_slot(context, task.subject)?;
                self.tasks.push(task);
            }
        }
        Ok(())
    }
}
