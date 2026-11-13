//! Every prepared parent precedes discovery and preparation of the next wave.
use super::*;
struct Scheduled<S: ApplicationSchema> {
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
        self.tasks.push(Scheduled {
            subject,
            rank,
            tree: tree(),
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
        while !self.tasks.is_empty() {
            let mut rank = usize::MAX;
            for task in &self.tasks {
                context.checkpoint(task.subject)?;
                if allowed.is_none_or(|families| {
                    families.iter().any(|family| family == task.tree.family())
                }) {
                    rank = rank.min(task.rank);
                }
            }
            if rank == usize::MAX {
                break;
            }
            for task in &mut self.tasks {
                if task.rank == rank
                    && allowed.is_none_or(|families| {
                        families.iter().any(|family| family == task.tree.family())
                    })
                {
                    task.parent = task.tree.prepare(context, candidate)?;
                }
            }
            let pin = candidate
                .pin_source(context.execution)
                .map_err(CoupledSourceReadStop::from)?;
            let mut next = Self::new();
            for task in self.tasks.drain(..) {
                if task.rank != rank
                    || !allowed.is_none_or(|families| {
                        families.iter().any(|family| family == task.tree.family())
                    })
                {
                    next.reserve_slot(context, task.subject)?;
                    next.tasks.push(task);
                    continue;
                }
                if let Some(mut parent) = task.parent {
                    parent.pin = pin.clone();
                    task.tree.discover(context, &parent, &mut next)?;
                }
            }
            *self = next;
        }
        Ok(())
    }
}
